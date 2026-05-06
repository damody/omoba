use std::rc::Rc;

use glow::HasContext;

use crate::core::draw_command::*;
use crate::graphics::primitives::ImageFit;
use crate::graphics::transforms::Transform3D;
use crate::renderer::contracts::*;
use crate::renderer::opengl::font_renderer::FontAtlas;
use crate::renderer::opengl::image_cache::ImageCache;
use crate::renderer::opengl::shader;
use crate::renderer::opengl::vertex::*;
use crate::runtime::contracts::WindowMetrics;

/// 將所有頂點圍繞原點旋轉 angle_deg 度。
fn rotate_vertices(verts: &mut [Vertex], angle_deg: f32, origin_x: f32, origin_y: f32) {
    let rad = angle_deg * std::f32::consts::PI / 180.0;
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    for v in verts.iter_mut() {
        let dx = v.x - origin_x;
        let dy = v.y - origin_y;
        v.x = origin_x + dx * cos_a - dy * sin_a;
        v.y = origin_y + dx * sin_a + dy * cos_a;
    }
}

/// 從命令的變換中獲取旋轉訊息。返回 (angle_deg, origin_x, origin_y) 或無。
fn get_rotation(cmd: &DrawCommand, transforms: &[Transform3D]) -> Option<(f32, f32, f32)> {
    if cmd.transform_payload_index == K_INVALID_PAYLOAD_INDEX {
        return None;
    }
    let t = transforms.get(cmd.transform_payload_index as usize)?;
    if t.rotation_z_deg.abs() < 0.001 {
        return None;
    }
    // 原點是相對於矩形位置的
    Some((t.rotation_z_deg, cmd.rect.x + t.origin_x, cmd.rect.y + t.origin_y))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum TextureMode {
    None = 0,
    AlphaMask = 1,
    Rgba = 2,
}

struct BlurResources {
    program: glow::Program,
    texel_size_loc: glow::UniformLocation,
    offset_loc: glow::UniformLocation,
    quad_vbo: glow::Buffer,
    fbo: [glow::Framebuffer; 2],
    tex: [glow::Texture; 2],
    half_w: i32,
    half_h: i32,
    copy_tex: glow::Texture,
    copy_w: i32,
    copy_h: i32,
}

pub struct OpenGlRenderer {
    gl: Rc<glow::Context>,
    program: glow::Program,
    vbo: glow::Buffer,
    viewport_uniform: glow::UniformLocation,
    texture_uniform: glow::UniformLocation,
    texture_mode_uniform: glow::UniformLocation,
    font_atlas: Option<FontAtlas>,
    icon_font_atlas: Option<FontAtlas>,
    image_cache: ImageCache,
    blur: Option<BlurResources>,
}

/// 如果代碼點位於 Unicode 專用區域中，則傳回 true
/// （用於圖示字體，如 Font Awesome）。
fn is_private_use_codepoint(cp: u32) -> bool {
    cp >= 0xE000 && cp <= 0xF8FF
}

/// 計算給定影像擬合模式的繪製矩形和 UV 座標。
/// 返回（draw_x、draw_y、draw_w、draw_h、u0、v0、u1、v1）。
fn resolve_image_fit(
    fit: ImageFit, fx: f32, fy: f32, fw: f32, fh: f32, img_w: f32, img_h: f32,
) -> (f32, f32, f32, f32, f32, f32, f32, f32) {
    match fit {
        ImageFit::Fill | ImageFit::Stretch => {
            (fx, fy, fw, fh, 0.0, 0.0, 1.0, 1.0)
        }
        ImageFit::Center => {
            let dx = fx + (fw - img_w) * 0.5;
            let dy = fy + (fh - img_h) * 0.5;
            (dx, dy, img_w, img_h, 0.0, 0.0, 1.0, 1.0)
        }
        ImageFit::Contain => {
            let scale = (fw / img_w).min(fh / img_h);
            let dw = img_w * scale;
            let dh = img_h * scale;
            let dx = fx + (fw - dw) * 0.5;
            let dy = fy + (fh - dh) * 0.5;
            (dx, dy, dw, dh, 0.0, 0.0, 1.0, 1.0)
        }
        ImageFit::Cover => {
            let scale = (fw / img_w).max(fh / img_h);
            let dw = img_w * scale;
            let dh = img_h * scale;
            let dx = fx + (fw - dw) * 0.5;
            let dy = fy + (fh - dh) * 0.5;
            (dx, dy, dw, dh, 0.0, 0.0, 1.0, 1.0)
        }
    }
}

impl OpenGlRenderer {
    pub fn gl(&self) -> &Rc<glow::Context> {
        &self.gl
    }

    pub unsafe fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let program = shader::create_program(&gl)?;
        let vbo = gl.create_buffer().map_err(|e| e.to_string())?;

        let viewport_uniform = gl.get_uniform_location(program, "u_viewport")
            .ok_or("u_viewport not found")?;
        let texture_uniform = gl.get_uniform_location(program, "u_texture")
            .ok_or("u_texture not found")?;
        let texture_mode_uniform = gl.get_uniform_location(program, "u_texture_mode")
            .ok_or("u_texture_mode not found")?;

        Ok(Self {
            gl,
            program,
            vbo,
            viewport_uniform,
            texture_uniform,
            texture_mode_uniform,
            font_atlas: None,
            icon_font_atlas: None,
            image_cache: ImageCache::new(),
            blur: None,
        })
    }

    pub unsafe fn set_font(&mut self, font: fontdue::Font, stb_to_fontdue_ratio: f32) {
        if let Some(ref atlas) = self.font_atlas {
            atlas.destroy(&self.gl);
        }
        self.font_atlas = Some(FontAtlas::new(&self.gl, font, stb_to_fontdue_ratio));
    }

    pub unsafe fn set_icon_font(&mut self, font: fontdue::Font, stb_to_fontdue_ratio: f32) {
        if let Some(ref atlas) = self.icon_font_atlas {
            atlas.destroy(&self.gl);
        }
        self.icon_font_atlas = Some(FontAtlas::new(&self.gl, font, stb_to_fontdue_ratio));
    }

    unsafe fn flush_vertices(&self, vertices: &[Vertex], tex_mode: TextureMode, texture: Option<glow::Texture>) {
        if vertices.is_empty() {
            return;
        }

        let gl = &self.gl;
        gl.use_program(Some(self.program));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        let data: &[u8] = std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * VERTEX_SIZE,
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STREAM_DRAW);

        let stride = VERTEX_SIZE as i32;
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, 8);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, 24);

        gl.uniform_1_i32(Some(&self.texture_mode_uniform), tex_mode as i32);

        if let Some(tex) = texture {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.uniform_1_i32(Some(&self.texture_uniform), 0);
        }

        gl.draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);

        gl.disable_vertex_attrib_array(0);
        gl.disable_vertex_attrib_array(1);
        gl.disable_vertex_attrib_array(2);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
    }

    unsafe fn set_scissor(&self, cmd: &DrawCommand, fb_w: i32, fb_h: i32) {
        if cmd.has_clip {
            let cx = cmd.clip_rect.x as i32;
            let cy = fb_h - (cmd.clip_rect.y + cmd.clip_rect.h) as i32;
            let cw = cmd.clip_rect.w as i32;
            let ch = cmd.clip_rect.h as i32;
            self.gl.scissor(cx.max(0), cy.max(0), cw.max(1), ch.max(1));
        } else {
            self.gl.scissor(0, 0, fb_w, fb_h);
        }
    }

    unsafe fn create_blur_texture(gl: &glow::Context, w: i32, h: i32) -> glow::Texture {
        let tex = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, w, h, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
        gl.bind_texture(glow::TEXTURE_2D, None);
        tex
    }

    unsafe fn create_blur_fbo(gl: &glow::Context, tex: glow::Texture) -> glow::Framebuffer {
        let fbo = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(tex), 0);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        fbo
    }

    unsafe fn ensure_blur(&mut self, fb_w: i32, fb_h: i32) {
        let half_w = (fb_w / 2).max(1);
        let half_h = (fb_h / 2).max(1);

        if let Some(ref b) = self.blur {
            if b.copy_w == fb_w && b.copy_h == fb_h {
                return;
            }
            // 大小已更改 - 銷毀舊資源並重新創建
            let gl = &self.gl;
            gl.delete_texture(b.copy_tex);
            gl.delete_texture(b.tex[0]);
            gl.delete_texture(b.tex[1]);
            gl.delete_framebuffer(b.fbo[0]);
            gl.delete_framebuffer(b.fbo[1]);
            gl.delete_buffer(b.quad_vbo);
            gl.delete_program(b.program);
            self.blur = None;
        }

        let gl = &self.gl;
        let program = shader::create_blur_program(gl).expect("blur shader failed");
        let texel_size_loc = gl.get_uniform_location(program, "u_texel_size").expect("u_texel_size");
        let offset_loc = gl.get_uniform_location(program, "u_offset").expect("u_offset");

        let quad_vbo = gl.create_buffer().unwrap();
        let quad_data: [f32; 12] = [
            0.0, 0.0,  1.0, 0.0,  1.0, 1.0,
            0.0, 0.0,  1.0, 1.0,  0.0, 1.0,
        ];
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
        let bytes: &[u8] = std::slice::from_raw_parts(quad_data.as_ptr() as *const u8, 48);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        let copy_tex = Self::create_blur_texture(gl, fb_w, fb_h);
        let tex0 = Self::create_blur_texture(gl, half_w, half_h);
        let tex1 = Self::create_blur_texture(gl, half_w, half_h);
        let fbo0 = Self::create_blur_fbo(gl, tex0);
        let fbo1 = Self::create_blur_fbo(gl, tex1);

        self.blur = Some(BlurResources {
            program, texel_size_loc, offset_loc, quad_vbo,
            fbo: [fbo0, fbo1], tex: [tex0, tex1],
            half_w, half_h,
            copy_tex, copy_w: fb_w, copy_h: fb_h,
        });
    }
}

impl RendererBackend for OpenGlRenderer {
    fn begin_frame(&mut self, metrics: &WindowMetrics, clear_state: &ClearState) {
        unsafe {
            let gl = &self.gl;
            gl.viewport(0, 0, metrics.framebuffer_w, metrics.framebuffer_h);
            if clear_state.clear_color {
                gl.clear_color(clear_state.r, clear_state.g, clear_state.b, clear_state.a);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
            }
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(0, 0, metrics.framebuffer_w, metrics.framebuffer_h);

            gl.use_program(Some(self.program));
            gl.uniform_2_f32(
                Some(&self.viewport_uniform),
                metrics.framebuffer_w as f32,
                metrics.framebuffer_h as f32,
            );
        }
    }

    fn render(&mut self, draw_data: &DrawDataView, metrics: &WindowMetrics) {
        unsafe {
            let fb_w = metrics.framebuffer_w;
            let fb_h = metrics.framebuffer_h;

            for cmd in draw_data.commands {
                self.set_scissor(cmd, fb_w, fb_h);

                match cmd.command_type {
                    CommandType::FilledRect => {
                        let mut verts = Vec::new();
                        push_rounded_quad(
                            &mut verts,
                            cmd.rect.x, cmd.rect.y, cmd.rect.w, cmd.rect.h,
                            cmd.color.r, cmd.color.g, cmd.color.b, cmd.color.a,
                            cmd.radius,
                        );
                        if let Some((angle, ox, oy)) = get_rotation(cmd, draw_data.transform_payloads) {
                            rotate_vertices(&mut verts, angle, ox, oy);
                        }
                        self.flush_vertices(&verts, TextureMode::None, None);
                    }
                    CommandType::RectOutline => {
                        let mut verts = Vec::new();
                        push_rounded_outline(
                            &mut verts,
                            cmd.rect.x, cmd.rect.y, cmd.rect.w, cmd.rect.h,
                            cmd.color.r, cmd.color.g, cmd.color.b, cmd.color.a,
                            cmd.radius, cmd.thickness,
                        );
                        if let Some((angle, ox, oy)) = get_rotation(cmd, draw_data.transform_payloads) {
                            rotate_vertices(&mut verts, angle, ox, oy);
                        }
                        self.flush_vertices(&verts, TextureMode::None, None);
                    }
                    CommandType::Text => {
                        let start = cmd.text_offset as usize;
                        let end = start + cmd.text_length as usize;
                        if end <= draw_data.text_arena.len() {
                            let text = std::str::from_utf8(&draw_data.text_arena[start..end]).unwrap_or("");
                            if !text.is_empty() && self.font_atlas.is_some() {
                                let gl_ref = Rc::clone(&self.gl);
                                let has_icon_font = self.icon_font_atlas.is_some();
                                let font_atlas = self.font_atlas.as_mut().unwrap();
                                let render_fs = font_atlas.render_font_size(cmd.font_size);
                                let icon_render_fs = if has_icon_font {
                                    self.icon_font_atlas.as_ref().unwrap().render_font_size(cmd.font_size)
                                } else {
                                    render_fs
                                };

                                // 第 1 遍：計算指標並柵格化所有字形。
                                // 收集每個字形的資料以避免在刷新期間持有可變借用。
                                struct GlyphInfo {
                                    is_icon: bool,
                                    advance: f32,
                                    // 四邊形資料（如果字形大小為零則無）
                                    quad: Option<(f32, f32, f32, f32, f32, f32, f32, f32)>,
                                    // u0、v0、u1、v1
                                }
                                let mut max_above: f32 = 0.0;
                                let mut max_below: f32 = 0.0;
                                let mut total_width: f32 = 0.0;
                                let mut glyph_infos: Vec<GlyphInfo> = Vec::new();

                                for ch in text.chars() {
                                    let use_icon = has_icon_font && is_private_use_codepoint(ch as u32);
                                    let entry = if use_icon {
                                        let icon_atlas = self.icon_font_atlas.as_mut().unwrap();
                                        icon_atlas.get_or_rasterize(&gl_ref, ch, icon_render_fs)
                                    } else {
                                        font_atlas.get_or_rasterize(&gl_ref, ch, render_fs)
                                    };
                                    let above = (entry.offset_y + entry.height).max(0.0);
                                    let below = (-entry.offset_y).max(0.0);
                                    max_above = max_above.max(above);
                                    max_below = max_below.max(below);
                                    total_width += entry.advance_width;
                                    let quad = if entry.width > 0.0 && entry.height > 0.0 {
                                        Some((entry.offset_x, entry.offset_y, entry.width, entry.height,
                                              entry.u0, entry.v0, entry.u1, entry.v1))
                                    } else {
                                        None
                                    };
                                    glyph_infos.push(GlyphInfo {
                                        is_icon: use_icon,
                                        advance: entry.advance_width,
                                        quad,
                                    });
                                }

                                if max_above + max_below < 1.0 {
                                    if let Some(lm) = font_atlas.font.horizontal_line_metrics(render_fs) {
                                        max_above = lm.ascent;
                                        max_below = -lm.descent;
                                    } else {
                                        max_above = render_fs * 0.72;
                                        max_below = render_fs * 0.28;
                                    }
                                }

                                let text_h = (max_above + max_below).max(1.0);
                                let baseline_y = (cmd.rect.y + (cmd.rect.h - text_h).max(0.0) * 0.5 + max_above).round();
                                let start_x = match cmd.align {
                                    TextAlign::Left => cmd.rect.x,
                                    TextAlign::Center => cmd.rect.x + (cmd.rect.w - total_width) * 0.5,
                                    TextAlign::Right => cmd.rect.x + cmd.rect.w - total_width,
                                };

                                // 第 2 遍：建立頂點批次並刷新。
                                // 不需要藉用可變的圖集－所有字形都已經光柵化。
                                let text_tex = font_atlas.texture;
                                let icon_tex = self.icon_font_atlas.as_ref().map(|a| a.texture);

                                let mut verts = Vec::new();
                                let mut pen_x = start_x;
                                let mut current_is_icon = false;
                                for gi in &glyph_infos {
                                    if !verts.is_empty() && gi.is_icon != current_is_icon {
                                        let tex = if current_is_icon { icon_tex.unwrap() } else { text_tex };
                                        self.flush_vertices(&verts, TextureMode::AlphaMask, Some(tex));
                                        verts.clear();
                                    }
                                    current_is_icon = gi.is_icon;
                                    if let Some((ox, oy, w, h, u0, v0, u1, v1)) = gi.quad {
                                        let gx = (pen_x + ox).round();
                                        let gy = (baseline_y - oy - h).round();
                                        push_textured_quad(
                                            &mut verts,
                                            gx, gy, w, h,
                                            cmd.color.r, cmd.color.g, cmd.color.b, cmd.color.a,
                                            u0, v0, u1, v1,
                                        );
                                    }
                                    pen_x += gi.advance;
                                }
                                if !verts.is_empty() {
                                    let tex = if current_is_icon { icon_tex.unwrap() } else { text_tex };
                                    self.flush_vertices(&verts, TextureMode::AlphaMask, Some(tex));
                                }
                            }
                        }
                    }
                    CommandType::ImageRect => {
                        let start = cmd.text_offset as usize;
                        let end = start + cmd.text_length as usize;
                        if end <= draw_data.text_arena.len() {
                            let path = std::str::from_utf8(&draw_data.text_arena[start..end]).unwrap_or("");
                            let gl_ref = Rc::clone(&self.gl);
                            if let Some(cached) = self.image_cache.get_or_load(&gl_ref, path) {
                                let tex = cached.texture;
                                let img_w = cached.width as f32;
                                let img_h = cached.height as f32;
                                let frame = &cmd.rect;
                                let fit = cmd.image_fit;

                                // 根據擬合模式計算繪製矩形和UV
                                let (draw_x, draw_y, draw_w, draw_h, u0, v0, u1, v1) =
                                    resolve_image_fit(fit, frame.x, frame.y, frame.w, frame.h, img_w, img_h);

                                let gl = &self.gl;
                                let radius = cmd.radius;

                                // 將剪刀設定為框架矩形（剪輯覆蓋/中心溢出）
                                let sx = frame.x as i32;
                                let sy = fb_h - (frame.y + frame.h) as i32;
                                let sw = frame.w as i32;
                                let sh = frame.h as i32;
                                gl.scissor(sx.max(0), sy.max(0), sw.max(1), sh.max(1));

                                if radius > 0.0 {
                                    // 基於模板的圓角剪切
                                    gl.enable(glow::STENCIL_TEST);
                                    gl.clear(glow::STENCIL_BUFFER_BIT);
                                    gl.stencil_func(glow::ALWAYS, 1, 0xFF);
                                    gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
                                    gl.color_mask(false, false, false, false);

                                    // 將圓角矩形繪製到模板中
                                    let mut stencil_verts = Vec::new();
                                    push_rounded_quad(
                                        &mut stencil_verts,
                                        frame.x, frame.y, frame.w, frame.h,
                                        1.0, 1.0, 1.0, 1.0,
                                        radius,
                                    );
                                    self.flush_vertices(&stencil_verts, TextureMode::None, None);

                                    // 現在只在 stencil == 1 的地方繪製影像
                                    gl.color_mask(true, true, true, true);
                                    gl.stencil_func(glow::EQUAL, 1, 0xFF);
                                    gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);

                                    let mut verts = Vec::new();
                                    push_textured_quad(
                                        &mut verts,
                                        draw_x, draw_y, draw_w, draw_h,
                                        1.0, 1.0, 1.0, 1.0,
                                        u0, v0, u1, v1,
                                    );
                                    self.flush_vertices(&verts, TextureMode::Rgba, Some(tex));

                                    gl.disable(glow::STENCIL_TEST);
                                } else {
                                    // 沒有半徑——只需用剪刀剪裁繪製
                                    let mut verts = Vec::new();
                                    push_textured_quad(
                                        &mut verts,
                                        draw_x, draw_y, draw_w, draw_h,
                                        1.0, 1.0, 1.0, 1.0,
                                        u0, v0, u1, v1,
                                    );
                                    self.flush_vertices(&verts, TextureMode::Rgba, Some(tex));
                                }

                                // 恢復剪刀
                                self.set_scissor(cmd, fb_w, fb_h);
                            }
                        }
                    }
                    CommandType::BackdropBlur => {
                        let blur_radius = cmd.blur_radius.max(0.0);
                        let corner_radius = cmd.radius;
                        let rect = cmd.rect;

                        if blur_radius < 1.0 {
                            // 無模糊 - 只需繪製半透明疊加層
                            let mut verts = Vec::new();
                            push_quad(&mut verts, rect.x, rect.y, rect.w, rect.h, 0.0, 0.0, 0.0, 0.15);
                            self.flush_vertices(&verts, TextureMode::None, None);
                        } else {
                            // 真正的河瀨背景模糊
                            self.ensure_blur(fb_w, fb_h);
                            let b = self.blur.as_ref().unwrap();
                            let bp = b.program;
                            let btsl = b.texel_size_loc;
                            let bol = b.offset_loc;
                            let bqv = b.quad_vbo;
                            let bfbo = b.fbo;
                            let btex = b.tex;
                            let bhw = b.half_w;
                            let bhh = b.half_h;
                            let bct = b.copy_tex;

                            let gl = &self.gl;

                            // 1.複製目前幀緩衝區→copy_tex
                            gl.bind_texture(glow::TEXTURE_2D, Some(bct));
                            gl.copy_tex_sub_image_2d(glow::TEXTURE_2D, 0, 0, 0, 0, 0, fb_w, fb_h);

                            // 2.禁用剪刀模糊頻道
                            gl.disable(glow::SCISSOR_TEST);

                            let passes = ((blur_radius / 3.0) as i32).clamp(2, 6);

                            // 3.設定模糊著色器+全螢幕四元VBO
                            gl.use_program(Some(bp));
                            gl.bind_buffer(glow::ARRAY_BUFFER, Some(bqv));
                            gl.enable_vertex_attrib_array(0);
                            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
                            gl.disable_vertex_attrib_array(1);
                            gl.disable_vertex_attrib_array(2);
                            gl.active_texture(glow::TEXTURE0);

                            // 4. Pass 0：以一半解析度下取樣 copy_tex → fbo[0]
                            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(bfbo[0]));
                            gl.viewport(0, 0, bhw, bhh);
                            gl.bind_texture(glow::TEXTURE_2D, Some(bct));
                            gl.uniform_2_f32(Some(&btsl), 1.0 / fb_w as f32, 1.0 / fb_h as f32);
                            gl.uniform_1_f32(Some(&bol), 0.0);
                            gl.draw_arrays(glow::TRIANGLES, 0, 6);

                            // 5. Kawase 模糊通道（半解析度乒乓球）
                            for i in 1..passes {
                                let src = ((i - 1) % 2) as usize;
                                let dst = (i % 2) as usize;
                                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(bfbo[dst]));
                                gl.bind_texture(glow::TEXTURE_2D, Some(btex[src]));
                                gl.uniform_2_f32(Some(&btsl), 1.0 / bhw as f32, 1.0 / bhh as f32);
                                gl.uniform_1_f32(Some(&bol), i as f32);
                                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                            }

                            let final_tex = btex[((passes - 1) % 2) as usize];

                            // 6.恢復主幀緩衝區+視窗+剪刀+主著色器
                            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                            gl.viewport(0, 0, fb_w, fb_h);
                            gl.enable(glow::SCISSOR_TEST);
                            self.set_scissor(cmd, fb_w, fb_h);
                            gl.use_program(Some(self.program));
                            gl.uniform_2_f32(Some(&self.viewport_uniform), fb_w as f32, fb_h as f32);

                            // 7. Composite：將模糊紋理繪製到模糊矩形中
                            let fb_wf = fb_w as f32;
                            let fb_hf = fb_h as f32;
                            let u0 = rect.x / fb_wf;
                            let u1 = (rect.x + rect.w) / fb_wf;
                            let v_top = 1.0 - rect.y / fb_hf;
                            let v_bot = 1.0 - (rect.y + rect.h) / fb_hf;

                            if corner_radius > 0.0 {
                                // 圓角模板
                                gl.enable(glow::STENCIL_TEST);
                                gl.clear(glow::STENCIL_BUFFER_BIT);
                                gl.stencil_func(glow::ALWAYS, 1, 0xFF);
                                gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
                                gl.color_mask(false, false, false, false);

                                let mut sv = Vec::new();
                                push_rounded_quad(&mut sv, rect.x, rect.y, rect.w, rect.h, 1.0, 1.0, 1.0, 1.0, corner_radius);
                                self.flush_vertices(&sv, TextureMode::None, None);

                                gl.color_mask(true, true, true, true);
                                gl.stencil_func(glow::EQUAL, 1, 0xFF);
                                gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
                            }

                            let mut verts = Vec::new();
                            push_textured_quad(&mut verts, rect.x, rect.y, rect.w, rect.h,
                                1.0, 1.0, 1.0, 1.0, u0, v_top, u1, v_bot);
                            self.flush_vertices(&verts, TextureMode::Rgba, Some(final_tex));

                            if corner_radius > 0.0 {
                                gl.disable(glow::STENCIL_TEST);
                            }
                        }
                    }
                    CommandType::Chevron => {
                        let mut verts = Vec::new();
                        let cx = cmd.rect.x + cmd.rect.w * 0.5;
                        let cy = cmd.rect.y + cmd.rect.h * 0.5;
                        let sz = cmd.rect.w.min(cmd.rect.h) * 0.35;
                        let c = &cmd.color;
                        let half_t = cmd.thickness.max(0.8) * 0.5;
                        let cos_r = cmd.rotation.cos();
                        let sin_r = cmd.rotation.sin();
                        let rotate = |dx: f32, dy: f32| -> (f32, f32) {
                            (cx + dx * cos_r - dy * sin_r, cy + dx * sin_r + dy * cos_r)
                        };
                        // ">" 形狀（旋轉 = 0）：尖端位於右側，手臂位於左上角和左下角
                        // 上臂：（左，上）->（右，中）
                        let ax0 = -sz * 0.4;
                        let ay0 = -sz * 0.5;
                        let ax1 = sz * 0.4;
                        let ay1 = 0.0_f32;
                        // 下臂：（右，中）->（左，下）
                        let bx0 = sz * 0.4;
                        let by0 = 0.0_f32;
                        let bx1 = -sz * 0.4;
                        let by1 = sz * 0.5;

                        // 將每個手臂繪製為旋轉的細四邊形（具有厚度的線段）
                        let draw_line = |verts: &mut Vec<Vertex>, lx0: f32, ly0: f32, lx1: f32, ly1: f32| {
                            let dx = lx1 - lx0;
                            let dy = ly1 - ly0;
                            let len = (dx * dx + dy * dy).sqrt().max(0.001);
                            // 厚度垂直方向
                            let px = -dy / len * half_t;
                            let py = dx / len * half_t;
                            let (p0x, p0y) = rotate(lx0 - px, ly0 - py);
                            let (p1x, p1y) = rotate(lx0 + px, ly0 + py);
                            let (p2x, p2y) = rotate(lx1 + px, ly1 + py);
                            let (p3x, p3y) = rotate(lx1 - px, ly1 - py);
                            // 四邊形的兩個三角形
                            let v = |x: f32, y: f32| -> Vertex {
                                Vertex { x, y, u: 0.0, v: 0.0, r: c.r, g: c.g, b: c.b, a: c.a }
                            };
                            verts.push(v(p0x, p0y));
                            verts.push(v(p1x, p1y));
                            verts.push(v(p2x, p2y));
                            verts.push(v(p0x, p0y));
                            verts.push(v(p2x, p2y));
                            verts.push(v(p3x, p3y));
                        };
                        draw_line(&mut verts, ax0, ay0, ax1, ay1);
                        draw_line(&mut verts, bx0, by0, bx1, by1);
                        self.flush_vertices(&verts, TextureMode::None, None);
                    }
                    CommandType::Line => {
                        // rect.x/y = 開始，rect.w/h = 結束（重新調整用途）
                        let x0 = cmd.rect.x;
                        let y0 = cmd.rect.y;
                        let x1 = cmd.rect.w;
                        let y1 = cmd.rect.h;
                        let half_t = cmd.thickness.max(0.5) * 0.5;
                        let c = &cmd.color;

                        let dx = x1 - x0;
                        let dy = y1 - y0;
                        let len = (dx * dx + dy * dy).sqrt().max(0.001);
                        let px = -dy / len * half_t;
                        let py = dx / len * half_t;

                        let v = |x: f32, y: f32| -> Vertex {
                            Vertex { x, y, u: 0.0, v: 0.0, r: c.r, g: c.g, b: c.b, a: c.a }
                        };
                        let verts = vec![
                            v(x0 - px, y0 - py),
                            v(x0 + px, y0 + py),
                            v(x1 + px, y1 + py),
                            v(x0 - px, y0 - py),
                            v(x1 + px, y1 + py),
                            v(x1 - px, y1 - py),
                        ];
                        self.flush_vertices(&verts, TextureMode::None, None);
                    }
                }
            }
        }
    }

    fn end_frame(&mut self) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.disable(glow::BLEND);
        }
    }

    fn release_resources(&mut self) {
        unsafe {
            if let Some(ref atlas) = self.font_atlas {
                atlas.destroy(&self.gl);
            }
            if let Some(ref atlas) = self.icon_font_atlas {
                atlas.destroy(&self.gl);
            }
            self.image_cache.destroy(&self.gl);
            if let Some(ref b) = self.blur {
                self.gl.delete_texture(b.copy_tex);
                self.gl.delete_texture(b.tex[0]);
                self.gl.delete_texture(b.tex[1]);
                self.gl.delete_framebuffer(b.fbo[0]);
                self.gl.delete_framebuffer(b.fbo[1]);
                self.gl.delete_buffer(b.quad_vbo);
                self.gl.delete_program(b.program);
            }
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_program(self.program);
        }
    }
}
