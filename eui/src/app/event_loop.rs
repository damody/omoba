use std::num::NonZeroU32;
use std::rc::Rc;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes};

use crate::app::options::AppOptions;
use crate::core::context::Context;
use crate::core::foundation::InputState;
use crate::renderer::contracts::{ClearState, DrawDataView, RendererBackend};
use crate::renderer::opengl::renderer::OpenGlRenderer;
use crate::runtime::contracts::WindowMetrics;
use crate::text::measurement::TextMeasurer;

struct AppState {
    window: Window,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    gl_context: glutin::context::PossiblyCurrentContext,
    renderer: OpenGlRenderer,
    ctx: Context,
    input: InputState,
    prev_mouse_down: bool,
    prev_right_down: bool,
    prev_middle_down: bool,
    start_time: std::time::Instant,
    frame_index: u64,
    _redraw_needed: bool,
    clipboard: Option<arboard::Clipboard>,
    key_ctrl: bool,
}

pub fn run_app<F>(build_ui: F, options: AppOptions)
where
    F: FnMut(&mut Context, &mut crate::quick::ui::UI<'_>) + 'static,
{
    let event_loop = EventLoop::new().expect("failed to create event loop");

    let mut app = AppHandler {
        build_ui: Box::new(build_ui),
        options,
        state: None,
    };

    event_loop.run_app(&mut app).expect("event loop error");
}

struct AppHandler {
    build_ui: Box<dyn FnMut(&mut Context, &mut crate::quick::ui::UI<'_>)>,
    options: AppOptions,
    state: Option<AppState>,
}

/// 使用上下文呼叫 build_ui 的幫助程序，圍繞借用檢查器進行工作。
/// 這是安全的，因為 UI 借用了 ctx，而 build_ui 接收了兩者
/// 但 UI 是在每一幀中重新建立的，並且不會比此呼叫更長久。
fn call_build_ui(
    build_ui: &mut dyn FnMut(&mut Context, &mut crate::quick::ui::UI<'_>),
    ctx: &mut Context,
) {
    let mut ui = crate::quick::ui::UI::new(ctx);
    // 我們需要將 ctx 傳遞給 build_ui，但 ui 已經借用了它。
    // 相反，build_ui 應該只使用 &mut UI 來存取 ctx。
    // 然而，簽名期望兩者兼而有之。我們使用原始指標來破壞別名。
    let ctx_ptr = ui.ctx() as *mut Context;
    unsafe {
        build_ui(&mut *ctx_ptr, &mut ui);
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title(self.options.title.clone())
            .with_inner_size(LogicalSize::new(self.options.width as f64, self.options.height as f64));

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_stencil_size(8)
            .with_multisampling(4);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs.reduce(|a, b| {
                    if a.num_samples() > b.num_samples() { a } else { b }
                }).unwrap()
            })
            .expect("failed to build display");

        let window = window.expect("window not created");
        let raw_handle = window.window_handle().ok()
            .map(|h| h.as_raw());

        let gl_display = gl_config.display();
        let context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 2))))
            .build(raw_handle);

        let not_current_ctx = unsafe {
            gl_display.create_context(&gl_config, &context_attrs)
                .expect("failed to create GL context")
        };

        let size = window.inner_size();
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.window_handle().unwrap().as_raw(),
            NonZeroU32::new(size.width.max(1)).unwrap(),
            NonZeroU32::new(size.height.max(1)).unwrap(),
        );

        let gl_surface = unsafe {
            gl_display.create_window_surface(&gl_config, &surface_attrs)
                .expect("failed to create window surface")
        };

        let gl_context = not_current_ctx
            .make_current(&gl_surface)
            .expect("failed to make GL context current");

        if self.options.vsync {
            let _ = gl_surface.set_swap_interval(&gl_context, glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap()));
        }

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name| {
                gl_display.get_proc_address(name)
            })
        };
        let gl = Rc::new(gl);

        let mut renderer = unsafe { OpenGlRenderer::new(gl).expect("failed to create OpenGL renderer") };

        // 載入字體：使用者指定的文件，或回退到系統預設值
        let font_data = if let Some(ref font_file) = self.options.text_font_file {
            std::fs::read(font_file).ok()
        } else {
            load_system_default_font()
        };
        let mut ctx = Context::new();
        if let Some(data) = font_data {
            if let Some(measurer) = TextMeasurer::new(&data) {
                let font = measurer.font().clone();
                let ratio = measurer.stb_to_fontdue_ratio();
                unsafe { renderer.set_font(font, ratio); }
                ctx.set_text_measurer(measurer);
            }
        }

        // 載入圖示字體（Font Awesome）以進行 PUA 程式碼點渲染。
        // 圖示字體可能是stb_truetype無法解析的OTF（基於CFF），
        // 所以我們只用 fontdue 載入它——圖示不需要 STB 測量。
        if self.options.enable_icon_font_fallback {
            if let Some(icon_data) = load_icon_font(&self.options.icon_font_file) {
                let settings = fontdue::FontSettings {
                    collection_index: 0,
                    scale: 40.0,
                    load_substitutions: true,
                };
                if let Ok(icon_font) = fontdue::Font::from_bytes(&icon_data[..], settings) {
                    // 圖示字體直接使用fontdue（ratio=0表示不進行STB校正）。
                    // 圖示大小由矩形決定，而不是文字度量。
                    unsafe { renderer.set_icon_font(icon_font, 0.0); }
                }
            }
        }

        let input = InputState::default();

        self.state = Some(AppState {
            window,
            gl_surface,
            gl_context,
            renderer,
            ctx,
            input,
            prev_mouse_down: false,
            prev_right_down: false,
            prev_middle_down: false,
            start_time: std::time::Instant::now(),
            frame_index: 0,
            _redraw_needed: true,
            clipboard: arboard::Clipboard::new().ok(),
            key_ctrl: false,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let size = state.window.inner_size();
                let scale = state.window.scale_factor() as f32;
                let fb_w = size.width as i32;
                let fb_h = size.height as i32;
                let win_w = (size.width as f32 / scale) as i32;
                let win_h = (size.height as f32 / scale) as i32;

                let metrics = WindowMetrics {
                    window_w: win_w,
                    window_h: win_h,
                    framebuffer_w: fb_w,
                    framebuffer_h: fb_h,
                    dpi_scale_x: scale,
                    dpi_scale_y: scale,
                    dpi_scale: scale,
                };

                state.input.time_seconds = state.start_time.elapsed().as_secs_f64();

                // 開始幀
                state.ctx.begin_frame(fb_w as f32, fb_h as f32, scale, state.input.clone());

                // 建構使用者介面
                call_build_ui(&mut *self.build_ui, &mut state.ctx);

                state.ctx.end_frame();

                // 將 Clipboard_out 從上下文同步回狀態
                // （begin_frame 複製輸入，因此 ctx 修改副本）
                if !state.ctx.input().clipboard_out.is_empty() {
                    state.input.clipboard_out = state.ctx.input().clipboard_out.clone();
                }

                // 使成為
                let bg = state.ctx.theme().background;
                let clear = ClearState { r: bg.r, g: bg.g, b: bg.b, a: bg.a, clear_color: true };
                state.renderer.begin_frame(&metrics, &clear);

                // 自訂預渲染回呼（GPU切片矩形等）
                if let Some(ref mut cb) = self.options.pre_render {
                    let gl = state.renderer.gl();
                    cb(gl, fb_w as f32, fb_h as f32);
                }

                let draw_data = DrawDataView {
                    commands: state.ctx.commands(),
                    text_arena: state.ctx.text_arena(),
                    transform_payloads: state.ctx.transform_payloads(),
                };
                state.renderer.render(&draw_data, &metrics);
                state.renderer.end_frame();

                // 自訂渲染後回調（幾何著色器流程等）
                if let Some(ref mut cb) = self.options.post_render {
                    let gl = state.renderer.gl();
                    cb(gl, fb_w as f32, fb_h as f32);
                }

                // 如果需要，請應用標題更改
                if let Some(ref new_title) = state.input.title_request {
                    state.window.set_title(new_title);
                }

                // 應用遊標圖標
                {
                    use crate::core::foundation::CursorIcon as CI;
                    let winit_cursor = match state.input.cursor_icon {
                        CI::Default => winit::window::CursorIcon::Default,
                        CI::EWResize => winit::window::CursorIcon::EwResize,
                        CI::NSResize => winit::window::CursorIcon::NsResize,
                        CI::Pointer => winit::window::CursorIcon::Pointer,
                        CI::Crosshair => winit::window::CursorIcon::Crosshair,
                    };
                    state.window.set_cursor(winit_cursor);
                    state.input.cursor_icon = CI::Default;
                }

                // 如果非空白則將clipboard_out寫入系統剪貼簿
                if !state.input.clipboard_out.is_empty() {
                    if let Some(cb) = state.clipboard.as_mut() {
                        let _ = cb.set_text(state.input.clipboard_out.clone());
                    }
                    state.input.clipboard_out.clear();
                }

                state.gl_surface.swap_buffers(&state.gl_context).expect("swap buffers");
                state.frame_index += 1;

                // 重置每幀輸入
                state.input.mouse_pressed = false;
                state.input.mouse_released = false;
                state.input.mouse_right_pressed = false;
                state.input.mouse_right_released = false;
                state.input.mouse_middle_pressed = false;
                state.input.mouse_middle_released = false;
                state.input.mouse_wheel_y = 0.0;
                state.input.text_input.clear();
                state.input.key_backspace = false;
                state.input.key_delete = false;
                state.input.key_enter = false;
                state.input.key_escape = false;
                state.input.key_left = false;
                state.input.key_right = false;
                state.input.key_up = false;
                state.input.key_down = false;
                state.input.key_home = false;
                state.input.key_end = false;
                state.input.key_select_all = false;
                state.input.key_copy = false;
                state.input.key_cut = false;
                state.input.key_paste = false;
                state.input.key_undo = false;
                state.input.key_redo = false;
                state.input.clipboard_text.clear();
                state.input.dropped_files.clear();
                state.input.title_request = None;

                // 請求連續重畫
                state.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.input.mouse_x = position.x as f32;
                state.input.mouse_y = position.y as f32;
                state.window.request_redraw();
            }
            WindowEvent::MouseInput { state: btn_state, button, .. } => {
                let pressed = btn_state == ElementState::Pressed;
                match button {
                    MouseButton::Left => {
                        state.input.mouse_pressed = pressed && !state.prev_mouse_down;
                        state.input.mouse_released = !pressed && state.prev_mouse_down;
                        state.input.mouse_down = pressed;
                        state.prev_mouse_down = pressed;
                    }
                    MouseButton::Right => {
                        state.input.mouse_right_pressed = pressed && !state.prev_right_down;
                        state.input.mouse_right_released = !pressed && state.prev_right_down;
                        state.input.mouse_right_down = pressed;
                        state.prev_right_down = pressed;
                    }
                    MouseButton::Middle => {
                        state.input.mouse_middle_pressed = pressed && !state.prev_middle_down;
                        state.input.mouse_middle_released = !pressed && state.prev_middle_down;
                        state.input.mouse_middle_down = pressed;
                        state.prev_middle_down = pressed;
                    }
                    _ => {}
                }
                state.window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        state.input.mouse_wheel_y = y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        state.input.mouse_wheel_y = pos.y as f32 / 30.0;
                    }
                }
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if pressed {
                    match event.logical_key {
                        Key::Named(NamedKey::Backspace) => state.input.key_backspace = true,
                        Key::Named(NamedKey::Delete) => state.input.key_delete = true,
                        Key::Named(NamedKey::Enter) => state.input.key_enter = true,
                        Key::Named(NamedKey::Escape) => state.input.key_escape = true,
                        Key::Named(NamedKey::ArrowLeft) => state.input.key_left = true,
                        Key::Named(NamedKey::ArrowRight) => state.input.key_right = true,
                        Key::Named(NamedKey::ArrowUp) => state.input.key_up = true,
                        Key::Named(NamedKey::ArrowDown) => state.input.key_down = true,
                        Key::Named(NamedKey::Home) => state.input.key_home = true,
                        Key::Named(NamedKey::End) => state.input.key_end = true,
                        Key::Named(NamedKey::Shift) => state.input.key_shift = true,
                        Key::Named(NamedKey::Control) => { state.key_ctrl = true; state.input.key_ctrl = true; }
                        Key::Character(ref c) if !state.key_ctrl && matches!(c.as_str(), "w" | "W") => state.input.key_w = true,
                        Key::Character(ref c) if !state.key_ctrl && matches!(c.as_str(), "a" | "A") => state.input.key_a = true,
                        Key::Character(ref c) if !state.key_ctrl && matches!(c.as_str(), "s" | "S") => state.input.key_s = true,
                        Key::Character(ref c) if !state.key_ctrl && matches!(c.as_str(), "d" | "D") => state.input.key_d = true,
                        Key::Character(ref c) if state.key_ctrl => {
                            match c.as_str() {
                                "a" | "A" => state.input.key_select_all = true,
                                "c" | "C" => state.input.key_copy = true,
                                "x" | "X" => state.input.key_cut = true,
                                "v" | "V" => {
                                    state.input.key_paste = true;
                                    state.input.clipboard_text = state.clipboard
                                        .as_mut()
                                        .and_then(|cb| cb.get_text().ok())
                                        .unwrap_or_default();
                                }
                                "z" | "Z" => {
                                    if state.input.key_shift {
                                        state.input.key_redo = true;
                                    } else {
                                        state.input.key_undo = true;
                                    }
                                }
                                "y" | "Y" => state.input.key_redo = true,
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    // 將可列印文字轉送至text_input（winit 0.30：event.text）
                    // 按住 Ctrl 時跳過以避免插入控製字符
                    if !state.key_ctrl {
                        if let Some(ref text) = event.text {
                            let s = text.as_str();
                            if !s.is_empty() && s.chars().all(|c| !c.is_control()) {
                                state.input.text_input.push_str(s);
                            }
                        }
                    }
                }
                if !pressed {
                    match event.logical_key {
                        Key::Named(NamedKey::Shift) => state.input.key_shift = false,
                        Key::Named(NamedKey::Control) => { state.key_ctrl = false; state.input.key_ctrl = false; }
                        Key::Character(ref c) if matches!(c.as_str(), "w" | "W") => state.input.key_w = false,
                        Key::Character(ref c) if matches!(c.as_str(), "a" | "A") => state.input.key_a = false,
                        Key::Character(ref c) if matches!(c.as_str(), "s" | "S") => state.input.key_s = false,
                        Key::Character(ref c) if matches!(c.as_str(), "d" | "D") => state.input.key_d = false,
                        _ => {}
                    }
                }
                state.window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    state.gl_surface.resize(
                        &state.gl_context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
                state.window.request_redraw();
            }
            WindowEvent::DroppedFile(path) => {
                state.input.dropped_files.push(path);
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

/// 嘗試載入預設的系統字體。
/// Windows：Segoe UI → Arial → Tahoma
/// macOS：SF Pro / Helvetica Neue
/// Linux：DejaVu Sans / Noto Sans / Liberation Sans
fn load_system_default_font() -> Option<Vec<u8>> {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\tahoma.ttf",
            "C:\\Windows\\Fonts\\calibri.ttf",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/SFPro.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
            "/Library/Fonts/Arial.ttf",
        ]
    } else {
        // Linux/其他
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
        ]
    };

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }
    None
}

/// 解析並載入圖標字體文件，嘗試多個候選路徑
/// 符合 C++resolve_bundled_icon_font_file() 行為。
fn load_icon_font(explicit_file: &Option<String>) -> Option<Vec<u8>> {
    const BUNDLED_NAME: &str = "Font Awesome 7 Free-Solid-900.otf";

    // 首先嘗試顯式路徑
    if let Some(ref path) = explicit_file {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }

    // 嘗試候選位置（匹配 C++）
    let candidates = [
        format!("include/{BUNDLED_NAME}"),
        format!("./include/{BUNDLED_NAME}"),
        format!("../include/{BUNDLED_NAME}"),
        format!("../../include/{BUNDLED_NAME}"),
    ];
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }

    // 嘗試相對於可執行文件
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for depth in &["", "..", "../..", "../../.."] {
                let p = exe_dir.join(depth).join("include").join(BUNDLED_NAME);
                if let Ok(data) = std::fs::read(&p) {
                    return Some(data);
                }
            }
        }
    }

    None
}
