// 階段 5：完整情境實施

use std::collections::HashMap;
use std::sync::Arc;

use crate::color::{mix, rgba, Color};
use crate::core::context_state::*;
use crate::core::context_utils::*;
use crate::core::draw_command::*;
use crate::core::foundation::*;
use crate::graphics::effects::*;
use crate::graphics::primitives::*;
use crate::graphics::transforms::*;
use crate::rect::Rect;
use crate::text::measurement::TextMeasurer;

/// 在位元組位置“pos”處解碼單一 UTF-8 代碼點。返回（代碼點，next_byte_index）。
fn decode_utf8_at(bytes: &[u8], pos: usize) -> (u32, usize) {
    let b0 = bytes[pos] as u32;
    if b0 < 0x80 {
        return (b0, pos + 1);
    }
    if b0 < 0xC0 || pos + 1 >= bytes.len() {
        return (0xFFFD, pos + 1);
    }
    let b1 = (bytes[pos + 1] & 0x3F) as u32;
    if b0 < 0xE0 {
        return ((b0 & 0x1F) << 6 | b1, pos + 2);
    }
    if pos + 2 >= bytes.len() {
        return (0xFFFD, pos + 2);
    }
    let b2 = (bytes[pos + 2] & 0x3F) as u32;
    if b0 < 0xF0 {
        return ((b0 & 0x0F) << 12 | b1 << 6 | b2, pos + 3);
    }
    if pos + 3 >= bytes.len() {
        return (0xFFFD, pos + 3);
    }
    let b3 = (bytes[pos + 3] & 0x3F) as u32;
    ((b0 & 0x07) << 18 | b1 << 12 | b2 << 6 | b3, pos + 4)
}

/// 尋找位元組偏移量落入哪個換行行。
fn find_line_for_offset(lines: &[(usize, usize)], offset: usize) -> usize {
    for (i, (start, len)) in lines.iter().enumerate() {
        if offset <= start + len {
            // 如果偏移量恰好位於行邊界，則優先選擇前一行末尾
            // 除非它是第一行
            if offset == *start + *len && i + 1 < lines.len() && offset == lines[i + 1].0 {
                // 偏移量位於邊界處；如果它符合下一行的開頭，
                // 只有當我們處於中弦時它才屬於下一行
                continue;
            }
            return i;
        }
    }
    lines.len().saturating_sub(1)
}

/// 用於文字輸入欄位的字元輸入過濾器。
///
/// - `Any`：不過濾
/// - `Int`：只允許數字與負號（`-` 可多次出現；最終語意由 parse 判斷）
/// - `Float`：允許數字、負號、小數點（允許中間狀態，由 parse 判斷合法）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharFilter {
    Any,
    Int,
    Float,
}

impl CharFilter {
    pub(crate) fn accept(self, c: char) -> bool {
        match self {
            CharFilter::Any => true,
            CharFilter::Int => c.is_ascii_digit() || c == '-',
            CharFilter::Float => c.is_ascii_digit() || c == '-' || c == '.',
        }
    }

    pub(crate) fn filter_str(self, s: &str) -> String {
        match self {
            CharFilter::Any => s.to_string(),
            _ => s.chars().filter(|c| self.accept(*c)).collect(),
        }
    }
}

#[allow(dead_code)]
pub struct Context {
    // 繪畫
    pub(crate) commands: Vec<DrawCommand>,
    pub(crate) text_arena: Vec<u8>,
    pub(crate) brush_payloads: Vec<Brush>,
    pub(crate) transform_payloads: Vec<Transform3D>,

    // 佈局
    pub(crate) layout_stack: Vec<ContextLayoutRectState>,
    pub(crate) scope_stack: Vec<ContextScopeState>,
    pub(crate) clip_stack: Vec<Rect>,

    // 狀態
    pub(crate) motion_states: HashMap<u64, ContextMotionState>,
    pub(crate) scroll_states: HashMap<u64, ContextScrollAreaState>,
    pub(crate) text_area_states: HashMap<u64, ContextTextAreaState>,

    // 輸入
    pub(crate) input: InputState,
    pub(crate) theme: Theme,
    pub(crate) frame_index: u64,
    pub(crate) prev_time_seconds: f64,
    pub(crate) has_prev_time: bool,
    pub(crate) ui_dt: f32,

    // 視窗
    pub(crate) viewport_w: f32,
    pub(crate) viewport_h: f32,
    pub(crate) dpi_scale: f32,

    // 文字
    pub(crate) text_measurer: Option<TextMeasurer>,

    // 熱/主動跟踪
    pub(crate) hot_id: u64,
    pub(crate) active_id: u64,
    pub(crate) focus_id: u64,

    // 記憶體資產註冊表
    pub(crate) memory_assets: HashMap<String, Arc<Vec<u8>>>,

    // 全球阿爾法
    pub(crate) global_alpha: f32,

    // 當前變換（應用於所有推送的命令）
    pub(crate) current_transform_index: u32,

    // 主動滑桿跟踪
    pub(crate) active_slider_id: u64,

    // 為 slider 右側的可編輯數字框保留的編輯中字串緩衝
    pub(crate) slider_edit_buffers: HashMap<u64, String>,

    // 磁滯計數器
    pub(crate) cmd_underuse: u32,
    pub(crate) text_underuse: u32,
}

impl Context {
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(512),
            text_arena: Vec::with_capacity(4096),
            brush_payloads: Vec::with_capacity(64),
            transform_payloads: Vec::with_capacity(16),
            layout_stack: Vec::with_capacity(16),
            scope_stack: Vec::with_capacity(8),
            clip_stack: Vec::with_capacity(8),
            motion_states: HashMap::new(),
            scroll_states: HashMap::new(),
            text_area_states: HashMap::new(),
            input: InputState::default(),
            theme: Theme::default(),
            frame_index: 0,
            prev_time_seconds: 0.0,
            has_prev_time: false,
            ui_dt: 1.0 / 60.0,
            viewport_w: 800.0,
            viewport_h: 600.0,
            dpi_scale: 1.0,
            text_measurer: None,
            hot_id: 0,
            active_id: 0,
            focus_id: 0,
            memory_assets: HashMap::new(),
            global_alpha: 1.0,
            current_transform_index: K_INVALID_PAYLOAD_INDEX,
            active_slider_id: 0,
            slider_edit_buffers: HashMap::new(),
            cmd_underuse: 0,
            text_underuse: 0,
        }
    }

    pub fn set_text_measurer(&mut self, measurer: TextMeasurer) {
        self.text_measurer = Some(measurer);
    }

    pub fn begin_frame(&mut self, viewport_w: f32, viewport_h: f32, dpi_scale: f32, input: InputState) {
        self.viewport_w = viewport_w;
        self.viewport_h = viewport_h;
        self.dpi_scale = dpi_scale;
        // 計算幀增量時間匹配 C++ ui_dt_ = Clip(dt, 1/240, 0.10)
        if self.has_prev_time {
            let dt = input.time_seconds - self.prev_time_seconds;
            self.ui_dt = (dt as f32).clamp(1.0 / 240.0, 0.10);
        } else {
            self.ui_dt = 1.0 / 60.0;
            self.has_prev_time = true;
        }
        self.prev_time_seconds = input.time_seconds;
        self.input = input;
        self.frame_index += 1;

        self.commands.clear();
        self.text_arena.clear();
        self.brush_payloads.clear();
        self.transform_payloads.clear();
        self.current_transform_index = K_INVALID_PAYLOAD_INDEX;
        self.layout_stack.clear();
        self.scope_stack.clear();
        self.clip_stack.clear();

        // 推根佈局
        self.layout_stack.push(ContextLayoutRectState {
            layout_rect: Rect::new(0.0, 0.0, viewport_w, viewport_h),
            cursor_y: 0.0,
            ..Default::default()
        });
    }

    pub fn end_frame(&mut self) {
        // 計算髒檢查的哈希值
        for cmd in &mut self.commands {
            cmd.hash = context_hash_command_base(cmd);
        }

        // GC 陳舊運動狀態
        let frame = self.frame_index;
        self.motion_states.retain(|_, state| frame - state.last_touched_frame < 120);
        self.scroll_states.retain(|_, state| frame - state.last_touched_frame < 120);
        self.text_area_states.retain(|_, state| frame - state.last_touched_frame < 120);
    }

    // ── 佈局矩形 ──

    pub fn layout_rect(&self) -> Rect {
        self.layout_stack.last()
            .map(|s| s.layout_rect)
            .unwrap_or(Rect::new(0.0, 0.0, self.viewport_w, self.viewport_h))
    }

    pub fn cursor_y(&self) -> f32 {
        self.layout_stack.last().map(|s| s.cursor_y).unwrap_or(0.0)
    }

    pub fn advance_cursor(&mut self, height: f32, gap: f32) {
        if let Some(state) = self.layout_stack.last_mut() {
            state.cursor_y += height + gap;
        }
    }

    pub fn push_layout_rect(&mut self, rect: Rect) {
        self.layout_stack.push(ContextLayoutRectState {
            layout_rect: rect,
            cursor_y: rect.y,
            ..Default::default()
        });
    }

    pub fn pop_layout_rect(&mut self) {
        if self.layout_stack.len() > 1 {
            self.layout_stack.pop();
        }
    }

    // ── 剪輯 ──

    pub fn push_clip(&mut self, rect: Rect) {
        let clipped = if let Some(top) = self.clip_stack.last() {
            context_intersect_rects(top, &rect).unwrap_or(Rect::ZERO)
        } else {
            rect
        };
        self.clip_stack.push(clipped);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.last().copied()
    }

    // ── 繪畫指令 ──

    /// 將請求的剪輯與剪輯堆疊合併，並返回有效剪輯。
    fn resolve_effective_clip(&self, requested_clip: Option<&Rect>) -> (Option<Rect>, bool) {
        let mut has_clip = false;
        let mut effective = Rect::ZERO;
        if let Some(req) = requested_clip {
            effective = *req;
            has_clip = true;
        }
        if let Some(stack_clip) = self.current_clip() {
            if has_clip {
                if let Some(merged) = context_intersect_rects(&effective, &stack_clip) {
                    effective = merged;
                } else {
                    return (None, false); // fully clipped
                }
            } else {
                effective = stack_clip;
                has_clip = true;
            }
        }
        if has_clip { (Some(effective), true) } else { (None, false) }
    }

    /// 將有效剪輯應用於命令。如果命令被完全裁剪，則傳回 false。
    fn apply_clip_to_command(&self, cmd: &mut DrawCommand, requested_clip: Option<&Rect>) -> bool {
        // 如果可用，請使用預先計算的visible_rect（來自旋轉變換），否則使用矩形
        let base_vis = if cmd.visible_rect.w > 0.0 && cmd.visible_rect.h > 0.0 {
            cmd.visible_rect
        } else {
            cmd.rect
        };
        let (effective, has_clip) = self.resolve_effective_clip(requested_clip);
        if !has_clip {
            cmd.visible_rect = base_vis;
            return true;
        }
        let effective = effective.unwrap();
        if let Some(vis) = context_intersect_rects(&base_vis, &effective) {
            cmd.has_clip = true;
            cmd.clip_rect = effective;
            cmd.visible_rect = vis;
            true
        } else {
            false
        }
    }

    fn push_command_with_clip(&mut self, mut cmd: DrawCommand, requested_clip: Option<&Rect>) -> usize {
        if self.global_alpha < 1.0 {
            cmd.color.a *= self.global_alpha;
        }
        if self.current_transform_index != K_INVALID_PAYLOAD_INDEX {
            cmd.transform_payload_index = self.current_transform_index;
            // 計算visible_rect作為旋轉矩形的軸對齊邊界框
            if let Some(t) = self.transform_payloads.get(self.current_transform_index as usize) {
                if t.rotation_z_deg.abs() > 0.001 {
                    let cx = cmd.rect.x + t.origin_x;
                    let cy = cmd.rect.y + t.origin_y;
                    let rad = t.rotation_z_deg * std::f32::consts::PI / 180.0;
                    let cos_a = rad.cos();
                    let sin_a = rad.sin();
                    let corners = [
                        (cmd.rect.x, cmd.rect.y),
                        (cmd.rect.x + cmd.rect.w, cmd.rect.y),
                        (cmd.rect.x + cmd.rect.w, cmd.rect.y + cmd.rect.h),
                        (cmd.rect.x, cmd.rect.y + cmd.rect.h),
                    ];
                    let mut min_x = f32::MAX;
                    let mut min_y = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut max_y = f32::MIN;
                    for (px, py) in corners {
                        let dx = px - cx;
                        let dy = py - cy;
                        let rx = cx + dx * cos_a - dy * sin_a;
                        let ry = cy + dx * sin_a + dy * cos_a;
                        min_x = min_x.min(rx);
                        min_y = min_y.min(ry);
                        max_x = max_x.max(rx);
                        max_y = max_y.max(ry);
                    }
                    cmd.visible_rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                }
            }
        }
        if !self.apply_clip_to_command(&mut cmd, requested_clip) {
            return usize::MAX;
        }
        let idx = self.commands.len();
        self.commands.push(cmd);
        idx
    }

    fn push_command(&mut self, cmd: DrawCommand) -> usize {
        self.push_command_with_clip(cmd, None)
    }

    /// 使用給定的顏色和粗細繪製從 (x0,y0) 到 (x1,y1) 的線段。
    pub fn paint_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, thickness: f32) -> usize {
        let half = thickness * 0.5;
        let min_x = x0.min(x1) - half;
        let min_y = y0.min(y1) - half;
        let max_x = x0.max(x1) + half;
        let max_y = y0.max(y1) + half;
        self.push_command(DrawCommand {
            command_type: CommandType::Line,
            rect: Rect::new(x0, y0, x1, y1), // start=(x,y), end=(w,h) repurposed
            visible_rect: Rect::new(min_x, min_y, max_x - min_x, max_y - min_y),
            color,
            thickness,
            ..Default::default()
        })
    }

    pub fn paint_filled_rect(&mut self, rect: Rect, color: Color, radius: f32) -> usize {
        self.push_command(DrawCommand {
            command_type: CommandType::FilledRect,
            rect,
            color,
            radius,
            ..Default::default()
        })
    }

    pub fn paint_filled_rect_clipped(&mut self, rect: Rect, color: Color, radius: f32, clip_rect: Option<&Rect>) -> usize {
        self.push_command_with_clip(DrawCommand {
            command_type: CommandType::FilledRect,
            rect,
            color,
            radius,
            ..Default::default()
        }, clip_rect)
    }

    pub fn paint_filled_rect_with_brush(&mut self, rect: Rect, brush: Brush, radius: f32) -> usize {
        let payload_index = self.brush_payloads.len() as u32;
        self.brush_payloads.push(brush);
        let color = match brush.kind {
            BrushKind::Solid => Color::new(brush.solid.r, brush.solid.g, brush.solid.b, brush.solid.a),
            BrushKind::LinearGradient if brush.linear.stop_count > 0 => {
                let c = &brush.linear.stops[0].color;
                Color::new(c.r, c.g, c.b, c.a)
            }
            BrushKind::RadialGradient if brush.radial.stop_count > 0 => {
                let c = &brush.radial.stops[0].color;
                Color::new(c.r, c.g, c.b, c.a)
            }
            _ => Color::WHITE,
        };
        self.push_command(DrawCommand {
            command_type: CommandType::FilledRect,
            rect,
            color,
            radius,
            brush_payload_index: payload_index,
            payload_hash: context_hash_brush(&brush),
            ..Default::default()
        })
    }

    pub fn paint_outline_rect(&mut self, rect: Rect, color: Color, radius: f32, thickness: f32) -> usize {
        self.push_command(DrawCommand {
            command_type: CommandType::RectOutline,
            rect,
            color,
            radius,
            thickness,
            ..Default::default()
        })
    }

    pub fn paint_text_clipped(&mut self, rect: Rect, text: &str, font_size: f32,
                              color: Color, align: TextAlign, clip_rect: Option<&Rect>) -> usize {
        let offset = self.text_arena.len() as u32;
        self.text_arena.extend_from_slice(text.as_bytes());
        let length = text.len() as u32;
        self.push_command_with_clip(DrawCommand {
            command_type: CommandType::Text,
            rect,
            color,
            font_size,
            align,
            text_offset: offset,
            text_length: length,
            ..Default::default()
        }, clip_rect)
    }

    pub fn paint_text(&mut self, rect: Rect, text: &str, font_size: f32, color: Color, align: TextAlign) -> usize {
        self.paint_text_clipped(rect, text, font_size, color, align, None)
    }

    /// 使用字元級換行繪製文本，每行發出一個文本命令。
    /// 符合 C++ `context_text_area` 換行演算法：逐字元寬度
    /// 測量時，「\n」強制換行，超過「rect.w」則換行到下一行。    pub fn paint_text_wrapped(&mut self, rect: Rect, text: &str, font_size: f32, color: Color, align: TextAlign) {
        let content_w = rect.w;
        if content_w <= 0.0 || text.is_empty() {
            return;
        }

        // 符合C++文字區域：line_h = text_font + 5.0
        let line_h = font_size + 5.0;

        let mut y = rect.y;
        let bytes = text.as_bytes();
        let mut line_start: usize = 0;
        let mut index: usize = 0;
        let mut line_width: f32 = 0.0;

        while index < bytes.len() {
            let ch = bytes[index];

            // 跳過\r
            if ch == b'\r' {
                index += 1;
                continue;
            }

            // 硬斷線
            if ch == b'\n' {
                let line_text = &text[line_start..index];
                let line_rect = Rect::new(rect.x, y, content_w, line_h);
                if y + line_h > rect.y && y < rect.y + rect.h {
                    self.paint_text(line_rect, line_text, font_size, color, align);
                }
                y += line_h;
                index += 1;
                line_start = index;
                line_width = 0.0;
                continue;
            }

            // 解碼 UTF-8 碼點並測量提前量
            let (cp, next) = decode_utf8_at(bytes, index);
            let advance = if let Some(ref measurer) = self.text_measurer {
                let ch_char = char::from_u32(cp).unwrap_or('\u{FFFD}');
                measurer.measure_char_advance(ch_char, font_size)
            } else {
                font_size * 0.5
            };

            // 如果超出內容寬度則換行
            if line_width > 0.0 && line_width + advance > content_w {
                let line_text = &text[line_start..index];
                let line_rect = Rect::new(rect.x, y, content_w, line_h);
                if y + line_h > rect.y && y < rect.y + rect.h {
                    self.paint_text(line_rect, line_text, font_size, color, align);
                }
                y += line_h;
                line_start = index;
                line_width = 0.0;
                continue;
            }

            index = next;
            line_width += advance;
        }

        // 發出最後一行
        if line_start <= bytes.len() {
            let line_text = &text[line_start..];
            let line_rect = Rect::new(rect.x, y, content_w, line_h);
            if y + line_h > rect.y && y < rect.y + rect.h {
                self.paint_text(line_rect, line_text, font_size, color, align);
            }
        }
    }

    /// 計算與 C++ 字元換行演算法相符的換行文字的換行符。
    /// 傳回每行的 (start_byte, length_bytes) Vec。
    fn compute_wrapped_lines(&self, text: &str, font_size: f32, content_w: f32) -> Vec<(usize, usize)> {
        let bytes = text.as_bytes();
        let mut lines = Vec::with_capacity(64);
        let mut line_start: usize = 0;
        let mut index: usize = 0;
        let mut line_width: f32 = 0.0;

        while index < bytes.len() {
            let ch = bytes[index];
            if ch == b'\r' { index += 1; continue; }
            if ch == b'\n' {
                lines.push((line_start, index - line_start));
                index += 1;
                line_start = index;
                line_width = 0.0;
                continue;
            }
            let (cp, next) = decode_utf8_at(bytes, index);
            let advance = if let Some(ref measurer) = self.text_measurer {
                let ch_char = char::from_u32(cp).unwrap_or('\u{FFFD}');
                measurer.measure_char_advance(ch_char, font_size)
            } else {
                font_size * 0.5
            };
            if line_width > 0.0 && line_width + advance > content_w {
                lines.push((line_start, index - line_start));
                line_start = index;
                line_width = 0.0;
                continue;
            }
            index = next;
            line_width += advance;
        }
        lines.push((line_start, bytes.len() - line_start));
        if lines.is_empty() {
            lines.push((0, 0));
        }
        lines
    }

    /// 與paint_text_wrapped類似，但將clip_rect傳遞給每一行（符合C++ text_area）。
    pub fn paint_text_wrapped_clipped(&mut self, rect: Rect, text: &str, font_size: f32,
                                       color: Color, align: TextAlign, clip: &Rect) {
        let content_w = rect.w;
        if content_w <= 0.0 || text.is_empty() {
            return;
        }

        let line_h = font_size + 5.0;
        let mut y = rect.y;
        let bytes = text.as_bytes();
        let mut line_start: usize = 0;
        let mut index: usize = 0;
        let mut line_width: f32 = 0.0;

        while index < bytes.len() {
            let ch = bytes[index];
            if ch == b'\r' { index += 1; continue; }
            if ch == b'\n' {
                let line_text = &text[line_start..index];
                let line_rect = Rect::new(rect.x, y, content_w, line_h);
                if y + line_h > rect.y && y < rect.y + rect.h {
                    self.paint_text_clipped(line_rect, line_text, font_size, color, align, Some(clip));
                }
                y += line_h;
                index += 1;
                line_start = index;
                line_width = 0.0;
                continue;
            }
            let (cp, next) = decode_utf8_at(bytes, index);
            let advance = if let Some(ref measurer) = self.text_measurer {
                let ch_char = char::from_u32(cp).unwrap_or('\u{FFFD}');
                measurer.measure_char_advance(ch_char, font_size)
            } else {
                font_size * 0.5
            };
            if line_width > 0.0 && line_width + advance > content_w {
                let line_text = &text[line_start..index];
                let line_rect = Rect::new(rect.x, y, content_w, line_h);
                if y + line_h > rect.y && y < rect.y + rect.h {
                    self.paint_text_clipped(line_rect, line_text, font_size, color, align, Some(clip));
                }
                y += line_h;
                line_start = index;
                line_width = 0.0;
                continue;
            }
            index = next;
            line_width += advance;
        }
        if line_start <= bytes.len() {
            let line_text = &text[line_start..];
            let line_rect = Rect::new(rect.x, y, content_w, line_h);
            if y + line_h > rect.y && y < rect.y + rect.h {
                self.paint_text_clipped(line_rect, line_text, font_size, color, align, Some(clip));
            }
        }
    }

    pub fn paint_image_rect(&mut self, rect: Rect, image_path: &str, fit: ImageFit, radius: f32) -> usize {
        let hash = context_hash_sv(image_path);
        let offset = self.text_arena.len() as u32;
        self.text_arena.extend_from_slice(image_path.as_bytes());
        let length = image_path.len() as u32;
        self.push_command(DrawCommand {
            command_type: CommandType::ImageRect,
            rect,
            radius,
            image_fit: fit,
            text_offset: offset,
            text_length: length,
            payload_hash: hash,
            ..Default::default()
        })
    }

    pub fn paint_backdrop_blur(&mut self, rect: Rect, blur_radius: f32, radius: f32) -> usize {
        self.push_command(DrawCommand {
            command_type: CommandType::BackdropBlur,
            rect,
            blur_radius,
            radius,
            ..Default::default()
        })
    }

    pub fn paint_chevron(&mut self, rect: Rect, color: Color, rotation: f32) -> usize {
        self.paint_chevron_ex(rect, color, rotation, 1.8)
    }

    pub fn paint_chevron_ex(&mut self, rect: Rect, color: Color, rotation: f32, thickness: f32) -> usize {
        self.push_command(DrawCommand {
            command_type: CommandType::Chevron,
            rect,
            color,
            rotation,
            thickness,
            ..Default::default()
        })
    }

    // ── 停靠/分離 ──

    pub fn dock_top(&mut self, height: f32) -> Rect {
        let lr = self.layout_rect();
        let r = Rect::new(lr.x, lr.y, lr.w, height);
        if let Some(state) = self.layout_stack.last_mut() {
            state.layout_rect.y += height;
            state.layout_rect.h = (state.layout_rect.h - height).max(0.0);
            state.cursor_y = state.layout_rect.y;
        }
        r
    }

    pub fn dock_bottom(&mut self, height: f32) -> Rect {
        let lr = self.layout_rect();
        let r = Rect::new(lr.x, lr.y + lr.h - height, lr.w, height);
        if let Some(state) = self.layout_stack.last_mut() {
            state.layout_rect.h = (state.layout_rect.h - height).max(0.0);
        }
        r
    }

    pub fn dock_left(&mut self, width: f32) -> Rect {
        let lr = self.layout_rect();
        let r = Rect::new(lr.x, lr.y, width, lr.h);
        if let Some(state) = self.layout_stack.last_mut() {
            state.layout_rect.x += width;
            state.layout_rect.w = (state.layout_rect.w - width).max(0.0);
        }
        r
    }

    pub fn dock_right(&mut self, width: f32) -> Rect {
        let lr = self.layout_rect();
        let r = Rect::new(lr.x + lr.w - width, lr.y, width, lr.h);
        if let Some(state) = self.layout_stack.last_mut() {
            state.layout_rect.w = (state.layout_rect.w - width).max(0.0);
        }
        r
    }

    pub fn split_h(&self, left_width: f32) -> (Rect, Rect) {
        let lr = self.layout_rect();
        let left = Rect::new(lr.x, lr.y, left_width, lr.h);
        let right = Rect::new(lr.x + left_width, lr.y, (lr.w - left_width).max(0.0), lr.h);
        (left, right)
    }

    pub fn split_v(&self, top_height: f32) -> (Rect, Rect) {
        let lr = self.layout_rect();
        let top = Rect::new(lr.x, lr.y, lr.w, top_height);
        let bottom = Rect::new(lr.x, lr.y + top_height, lr.w, (lr.h - top_height).max(0.0));
        (top, bottom)
    }

    // ── 屈伸划船 ──

    pub fn begin_flex_row(&mut self, items: &[FlexLength], gap: f32, align: FlexAlign) {
        let lr = self.layout_rect();
        let n = items.len();
        let total_gap = if n > 1 { gap * (n as f32 - 1.0) } else { 0.0 };
        let avail = (lr.w - total_gap).max(0.0);

        // 解析寬度
        let mut widths = vec![0.0f32; n];
        let mut total_fixed = 0.0f32;
        let mut total_flex = 0.0f32;

        for (i, item) in items.iter().enumerate() {
            match item {
                FlexLength::Fixed(px) => {
                    widths[i] = *px;
                    total_fixed += px;
                }
                FlexLength::Flex(w) => {
                    total_flex += w;
                }
                FlexLength::Content { min, .. } => {
                    widths[i] = *min;
                    total_fixed += min;
                }
            }
        }

        let remaining = (avail - total_fixed).max(0.0);
        if total_flex > 0.0 {
            for (i, item) in items.iter().enumerate() {
                if let FlexLength::Flex(w) = item {
                    widths[i] = remaining * w / total_flex;
                }
            }
        }

        // 計算矩形
        let mut rects = Vec::with_capacity(n);
        let mut x = lr.x;
        let y = self.cursor_y();
        for &width in &widths {
            rects.push(Rect::new(x, y, width, 0.0)); // height TBD
            x += width + gap;
        }

        if let Some(state) = self.layout_stack.last_mut() {
            state.flex_row = ContextFlexRowState {
                active: true,
                items: items.to_vec(),
                widths: widths.clone(),
                heights: vec![0.0; n],
                item_rects: rects,
                cmd_begin: vec![0; n],
                cmd_end: vec![0; n],
                index: 0,
                gap,
                y,
                row_height: 0.0,
                align,
            };
        }
    }

    pub fn next_flex_item(&mut self) -> Option<Rect> {
        let state = self.layout_stack.last_mut()?;
        let fr = &mut state.flex_row;
        if !fr.active || fr.index as usize >= fr.item_rects.len() {
            return None;
        }
        let idx = fr.index as usize;
        fr.cmd_begin[idx] = self.commands.len();
        let rect = fr.item_rects[idx];
        fr.index += 1;
        Some(rect)
    }

    pub fn finish_flex_item(&mut self, height: f32) {
        if let Some(state) = self.layout_stack.last_mut() {
            let fr = &mut state.flex_row;
            if !fr.active {
                return;
            }
            let idx = (fr.index - 1) as usize;
            if idx < fr.heights.len() {
                fr.heights[idx] = height;
                fr.cmd_end[idx] = self.commands.len();
                if height > fr.row_height {
                    fr.row_height = height;
                }
            }
        }
    }

    pub fn end_flex_row(&mut self) {
        if let Some(state) = self.layout_stack.last_mut() {
            let row_height = state.flex_row.row_height;
            let align = state.flex_row.align;

            // 應用垂直對齊
            if align != FlexAlign::Top {
                let n = state.flex_row.item_rects.len();
                for i in 0..n {
                    let h = state.flex_row.heights[i];
                    let offset = match align {
                        FlexAlign::Center => (row_height - h) * 0.5,
                        FlexAlign::Bottom => row_height - h,
                        FlexAlign::Top => 0.0,
                    };
                    if offset.abs() > 0.5 {
                        let begin = state.flex_row.cmd_begin[i];
                        let end = state.flex_row.cmd_end[i];
                        for cmd_idx in begin..end.min(self.commands.len()) {
                            self.commands[cmd_idx].rect.y += offset;
                            if self.commands[cmd_idx].has_clip {
                                // 不要移動剪輯矩形
                            }
                            self.commands[cmd_idx].visible_rect.y += offset;
                        }
                    }
                }
            }

            state.cursor_y = state.flex_row.y + row_height;
            state.flex_row.active = false;
        }
    }

    // ── 行（網格）──

    pub fn begin_row(&mut self, columns: i32, gap: f32) {
        if let Some(state) = self.layout_stack.last_mut() {
            state.row = ContextRowState {
                active: true,
                columns,
                index: 0,
                next_span: 1,
                gap,
                y: state.cursor_y,
                max_height: 0.0,
            };
        }
    }

    pub fn next_cell(&mut self) -> Rect {
        let lr = self.layout_rect();
        if let Some(state) = self.layout_stack.last_mut() {
            let row = &mut state.row;
            if !row.active {
                return Rect::ZERO;
            }
            let cols = row.columns.max(1) as f32;
            let total_gap = row.gap * (cols - 1.0);
            let cell_w = ((lr.w - total_gap) / cols).max(0.0);
            let col = row.index % row.columns;
            let span = row.next_span.max(1);
            let x = lr.x + col as f32 * (cell_w + row.gap);
            let w = cell_w * span as f32 + row.gap * (span - 1).max(0) as f32;
            let rect = Rect::new(x, row.y, w, 0.0);
            row.index += span;
            row.next_span = 1;
            rect
        } else {
            Rect::ZERO
        }
    }

    pub fn finish_cell(&mut self, height: f32) {
        if let Some(state) = self.layout_stack.last_mut() {
            let row = &mut state.row;
            if height > row.max_height {
                row.max_height = height;
            }
            if row.index >= row.columns {
                row.y += row.max_height + row.gap;
                row.max_height = 0.0;
                row.index = 0;
            }
        }
    }

    pub fn end_row(&mut self) {
        if let Some(state) = self.layout_stack.last_mut() {
            if state.row.max_height > 0.0 {
                state.row.y += state.row.max_height + state.row.gap;
            }
            state.cursor_y = state.row.y;
            state.row.active = false;
        }
    }

    // ── 運動（動畫狀態）──

    /// 使用匹配 C++ animate_motion_channel 的指數衰減對單一運動通道進行動畫處理。
    fn animate_motion_channel(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
        if (current - target).abs() <= 1.5e-3 {
            return target;
        }
        let blend = 1.0 - (-speed.max(0.0) * dt).exp();
        let next = current + (target - current) * blend;
        if (next - target).abs() <= 1.5e-3 { target } else { next }
    }

    pub fn motion(&mut self, id: u64, hovered: bool, pressed: bool) -> MotionResult {
        let m = self.motion_ex(id, hovered, pressed, false, false);
        MotionResult {
            hover: m.hover,
            press: m.press,
        }
    }

    pub fn presence(&mut self, id: u64, visible: bool) -> f32 {
        let dt = self.ui_dt;
        let speed = 6.0;
        let state = self.motion_states.entry(id).or_default();
        state.last_touched_frame = self.frame_index;

        let target = if visible { 1.0 } else { 0.0 };
        if !state.initialized {
            state.active = target;
            state.initialized = true;
        }
        state.active = Self::animate_motion_channel(state.active, target, speed, dt);
        state.active
    }

    pub fn presence_ex(&mut self, id: u64, visible: bool, speed_in: f32, speed_out: f32) -> f32 {
        let dt = self.ui_dt;
        let state = self.motion_states.entry(id).or_default();
        state.last_touched_frame = self.frame_index;

        let target = if visible { 1.0 } else { 0.0 };
        if !state.initialized {
            state.active = target;
            state.initialized = true;
        }
        let speed = if visible { speed_in } else { speed_out };
        state.active = Self::animate_motion_channel(state.active, target, speed, dt);
        state.active
    }

    pub fn animated_value(&mut self, id: u64, target: f32) -> f32 {
        self.animated_value_ex(id, target, 10.0)
    }

    pub fn animated_value_ex(&mut self, id: u64, target: f32, speed: f32) -> f32 {
        let dt = self.ui_dt;
        let state = self.motion_states.entry(id).or_default();
        state.last_touched_frame = self.frame_index;

        if !state.value_initialized {
            state.value = target;
            state.value_initialized = true;
        }
        let blend = 1.0 - (-speed * dt).exp();
        state.value += (target - state.value) * blend;
        state.value
    }

    pub fn animated_value_read(&self, id: u64, default: f32) -> f32 {
        self.motion_states.get(&id).map_or(default, |s| s.value)
    }

    pub fn animated_value_reset(&mut self, id: u64, value: f32) {
        let state = self.motion_states.entry(id).or_default();
        state.value = value;
        state.value_initialized = true;
    }

    /// 具有焦點和活動通道的全運動狀態，與 C++ update_motion_state 相符。
    pub fn motion_ex(&mut self, id: u64, hovered: bool, pressed: bool, focused: bool, active: bool) -> MotionResultEx {
        let dt = self.ui_dt;
        let state = self.motion_states.entry(id).or_default();
        state.last_touched_frame = self.frame_index;

        let hover_target = if hovered { 1.0 } else { 0.0 };
        let press_target = if pressed { 1.0 } else { 0.0 };
        let focus_target = if focused { 1.0 } else { 0.0 };
        let active_target = if active { 1.0 } else { 0.0 };

        if !state.initialized {
            state.hover = hover_target;
            state.press = press_target;
            state.focus = focus_target;
            state.active = active_target;
            state.initialized = true;
        } else {
            state.hover = Self::animate_motion_channel(state.hover, hover_target, if hovered { 18.0 } else { 12.0 }, dt);
            state.press = Self::animate_motion_channel(state.press, press_target, if pressed { 28.0 } else { 18.0 }, dt);
            state.focus = Self::animate_motion_channel(state.focus, focus_target, if focused { 16.0 } else { 11.0 }, dt);
            state.active = Self::animate_motion_channel(state.active, active_target, if active { 14.0 } else { 10.0 }, dt);
        }

        MotionResultEx {
            hover: state.hover,
            press: state.press,
            focus: state.focus,
            active: state.active,
        }
    }

    /// 將 0..1 運動值捕捉乾淨的步驟，與 C++ snap_visual_motion 相符。
    pub fn snap_visual_motion(value: f32, step: f32) -> f32 {
        let clamped = value.clamp(0.0, 1.0);
        if step <= 1e-6 {
            return clamped;
        }
        if clamped <= step * 0.5 {
            return 0.0;
        }
        if clamped >= 1.0 - step * 0.5 {
            return 1.0;
        }
        (clamped / step).round() * step
    }

    /// 與 C++ add_soft_glow_tracked 相符的柔和發光效果。
    pub fn paint_soft_glow(&mut self, rect: Rect, color: Color, radius: f32, intensity: f32, spread: f32) {
        let glow = intensity.clamp(0.0, 1.0);
        let area = rect.w.max(0.0) * rect.h.max(0.0);
        let inner_alpha = 0.08 * glow * color.a;
        let outer_alpha = 0.035 * glow * color.a;
        if glow <= 1e-3 || color.a <= 1e-3 || (inner_alpha < 0.010 && outer_alpha < 0.006) {
            return;
        }
        let allow_outer = outer_alpha >= 0.010 && area < 22000.0 && spread >= 4.0;
        let inner_spread = spread * (0.30 + glow * 0.28);
        let outer_spread = spread * (0.72 + glow * 0.48);
        if allow_outer {
            let outer_rect = context_expand_rect(&rect, outer_spread, outer_spread);
            let outer_color = rgba(color.r, color.g, color.b, 0.030 * glow * color.a);
            self.paint_filled_rect(outer_rect, outer_color, radius + outer_spread);
        }
        let inner_rect = context_expand_rect(&rect, inner_spread, inner_spread);
        let inner_color = rgba(color.r, color.g, color.b, 0.074 * glow * color.a);
        self.paint_filled_rect(inner_rect, inner_color, radius + inner_spread);
    }

    /// 輸入鑲邊（背景+帶有懸停/焦點動畫的邊框），匹配C++draw_input_chrome。
    pub fn draw_input_chrome(&mut self, id: u64, rect: Rect, hovered: bool, focused: bool,
                              base_fill: Color, radius: f32, base_thickness: f32) {
        let m = self.motion_ex(id, hovered, false, focused, focused);
        let hover_v = Self::snap_visual_motion(m.hover, 1.0 / 40.0);
        let focus_v = Self::snap_visual_motion(m.focus, 1.0 / 40.0);
        let glow = focus_v * 0.78 + hover_v * 0.06;
        if glow > 0.045 {
            let primary = self.theme.primary;
            self.paint_soft_glow(rect, primary, radius, glow, 7.0);
        }
        let secondary = self.theme.secondary;
        let primary = self.theme.primary;
        let fill = mix(base_fill, mix(secondary, primary, 0.14), hover_v * 0.08 + focus_v * 0.10);
        let border = mix(self.theme.input_border, self.theme.focus_ring, focus_v * 0.92 + hover_v * 0.24);
        let thickness = base_thickness + focus_v * 0.80 + hover_v * 0.06;
        self.paint_filled_rect(rect, fill, radius);
        self.paint_outline_rect(rect, border, radius, thickness);
    }

    // ── 命中測試 ──

    pub fn is_hovered(&self, rect: &Rect) -> bool {
        rect.contains(self.input.mouse_x, self.input.mouse_y)
    }

    pub fn is_mouse_pressed(&self) -> bool {
        self.input.mouse_pressed
    }

    pub fn is_mouse_released(&self) -> bool {
        self.input.mouse_released
    }

    pub fn is_mouse_down(&self) -> bool {
        self.input.mouse_down
    }

    // ── 小部件助手 ──

    #[allow(clippy::too_many_lines)]
    pub fn button(&mut self, id: u64, rect: Rect, label: &str, style: ButtonStyle) -> bool {
        // 乘數應用於每個按鈕標籤尺寸（圖示、普通、組合）。
        // 以前的按鈕看起來比周圍的標籤小，因為
        // 預設矩形高度 × 0.38 給出約 13.7 px，而標籤則成長到 1.5 ×。
        const BUTTON_TEXT_SCALE: f32 = 1.5;
        let k_icon_visual_scale: f32 = 1.15;
        let draw_label = label;

        // 檢測強制左對齊（製表符前綴）
        let (draw_label, force_left_align) = if let Some(stripped) = draw_label.strip_prefix('\t') {
            (stripped, true)
        } else {
            (draw_label, false)
        };

        // 檢測類似圖示的標籤（<=4 個字符，所有非 ASCII 或非字母數字）
        let icon_like = !draw_label.is_empty()
            && draw_label.chars().count() <= 4
            && draw_label.chars().all(|ch| ch as u32 >= 0x80 || !ch.is_alphanumeric());

        // 字體大小
        let text_size = if icon_like {
            (rect.h * 0.72 * k_icon_visual_scale * BUTTON_TEXT_SCALE)
                .clamp(13.0 * BUTTON_TEXT_SCALE, 36.0 * BUTTON_TEXT_SCALE)
        } else {
            (rect.h * 0.38 * BUTTON_TEXT_SCALE)
                .clamp(12.0 * BUTTON_TEXT_SCALE, 34.0 * BUTTON_TEXT_SCALE)
        };

        // 圖示+文字組合偵測 - 僅當前綴完全由以下組成時
        // 「類似圖示」的程式碼點（PUA 表示 Font Awesome/Material、表情符號範圍或
        // 雜項標誌/符號）。 CJK / 韓文 / 平假名 / 片假名 / 一般
        // punctuation are explicitly NOT icons; treating "敵近戰兵 HP:220" or
        // "切換 Player/Enemy" as icon+text combos scales the CJK prefix up to
        // 圖示大小 (~22px)，而 ASCII 回落到~12px，導致重疊。
        let mut icon_text_combo = false;
        let mut icon_part = "";
        let mut text_part = "";
        let is_icon_codepoint = |c: char| -> bool {
            let cp = c as u32;
            (0xE000..=0xF8FF).contains(&cp)
                || (0xF0000..=0xFFFFD).contains(&cp)
                || (0x100000..=0x10FFFD).contains(&cp)
                || (0x2190..=0x21FF).contains(&cp)   // Arrows
                || (0x2500..=0x257F).contains(&cp)   // Box Drawing
                || (0x2580..=0x259F).contains(&cp)   // Block Elements
                || (0x25A0..=0x25FF).contains(&cp)   // Geometric Shapes
                || (0x2600..=0x26FF).contains(&cp)   // Misc Symbols
                || (0x2700..=0x27BF).contains(&cp)   // Dingbats
                || (0x1F300..=0x1FAFF).contains(&cp) // Emoji / Symbols and Pictographs
        };
        if !draw_label.is_empty() {
            let first_ch = draw_label.chars().next().unwrap();
            if is_icon_codepoint(first_ch) {
                let split = draw_label.find("  ").or_else(|| draw_label.find(' '));
                if let Some(split_pos) = split {
                    if split_pos > 0 {
                        let text_start_pos = draw_label[split_pos..].find(|c: char| c != ' ');
                        if let Some(ts) = text_start_pos {
                            let ts = split_pos + ts;
                            if ts < draw_label.len() {
                                let prefix = &draw_label[..split_pos];
                                if prefix.chars().all(is_icon_codepoint) {
                                    icon_part = prefix;
                                    text_part = &draw_label[ts..];
                                    icon_text_combo = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 顏色
        let hovered = self.is_hovered(&rect);
        let held = hovered && self.is_mouse_down();

        let mut fill = match style {
            ButtonStyle::Primary => self.theme.primary,
            ButtonStyle::Secondary => self.theme.secondary,
            ButtonStyle::Ghost => {
                if hovered {
                    mix(self.theme.secondary, self.theme.panel, 0.5)
                } else {
                    self.theme.panel
                }
            }
        };
        let text_color = match style {
            ButtonStyle::Primary => self.theme.primary_text,
            ButtonStyle::Secondary | ButtonStyle::Ghost => self.theme.text,
        };

        if held {
            fill = mix(fill, self.theme.secondary_active, 0.35);
        } else if hovered && style != ButtonStyle::Ghost {
            fill = mix(fill, self.theme.secondary_hover, 0.30);
        }

        // 運動
        let m = self.motion_ex(id, hovered, held, false, false);
        let hover_v = Self::snap_visual_motion(m.hover, 1.0 / 48.0);
        let press_v = Self::snap_visual_motion(m.press, 1.0 / 36.0);

        // 視覺變換
        let visual_scale = 1.0 - press_v * 0.018;
        let mut visual_rect = context_scale_rect_from_center(&rect, visual_scale, visual_scale);
        visual_rect = context_translate_rect(&visual_rect, 0.0, press_v * 0.4);

        let outline_color = match style {
            ButtonStyle::Primary => mix(self.theme.primary, self.theme.focus_ring, hover_v * 0.26 + press_v * 0.12),
            _ => mix(self.theme.outline, self.theme.focus_ring, hover_v * 0.26 + press_v * 0.12),
        };

        let radius = self.theme.radius;
        let primary = self.theme.primary;
        let secondary = self.theme.secondary;
        let panel = self.theme.panel;

        if style == ButtonStyle::Primary {
            fill = mix(fill, panel, hover_v * 0.12);
            self.paint_soft_glow(visual_rect, primary, radius, hover_v * 0.54 + press_v * 0.14, 6.5);
        } else {
            fill = mix(fill, panel, hover_v * 0.05);
            self.paint_soft_glow(visual_rect, mix(primary, secondary, 0.52), radius, hover_v * 0.12, 5.0);
        }

        self.paint_filled_rect(visual_rect, fill, radius);
        self.paint_outline_rect(visual_rect, outline_color, radius, 1.0 + hover_v * 0.14);

        // 文字渲染
        if icon_text_combo {
            let pad = if force_left_align {
                (visual_rect.h * 0.24).clamp(9.0, 14.0)
            } else {
                (visual_rect.h * 0.24).clamp(8.0, 14.0)
            };
            let icon_size = if force_left_align {
                (visual_rect.h * 0.46 * k_icon_visual_scale).clamp(10.0, 24.0)
            } else {
                (visual_rect.h * 0.60 * k_icon_visual_scale).clamp(12.0, 36.0)
            };
            let icon_w = if force_left_align {
                (icon_size + 2.0).max(visual_rect.h * 0.44)
            } else {
                14.0_f32.max(visual_rect.h - pad * 0.2)
            };
            let icon_rect = Rect::new(visual_rect.x + pad, visual_rect.y, icon_w, visual_rect.h);
            let gap = if force_left_align {
                6.0_f32.max(pad * 0.58)
            } else {
                4.0_f32.max(pad * 0.45)
            };
            let text_x = icon_rect.x + icon_rect.w + gap;
            let text_rect_w = (visual_rect.x + visual_rect.w - text_x - pad).max(0.0);
            let combo_text_size = if force_left_align {
                (visual_rect.h * 0.37 * BUTTON_TEXT_SCALE)
                    .clamp(12.0 * BUTTON_TEXT_SCALE, 30.0 * BUTTON_TEXT_SCALE)
            } else {
                (visual_rect.h * 0.35 * BUTTON_TEXT_SCALE)
                    .clamp(11.0 * BUTTON_TEXT_SCALE, 30.0 * BUTTON_TEXT_SCALE)
            };
            self.paint_text(icon_rect, icon_part, icon_size, text_color, TextAlign::Center);
            self.paint_text(
                Rect::new(text_x, visual_rect.y, text_rect_w, visual_rect.h),
                text_part, combo_text_size, text_color, TextAlign::Left,
            );
        } else if force_left_align {
            let pad = (visual_rect.h * 0.24).clamp(8.0, 14.0);
            self.paint_text(
                Rect::new(visual_rect.x + pad, visual_rect.y, (visual_rect.w - pad * 1.5).max(0.0), visual_rect.h),
                draw_label, text_size, text_color, TextAlign::Left,
            );
        } else {
            self.paint_text(visual_rect, draw_label, text_size, text_color, TextAlign::Center);
        }

        hovered && self.input.mouse_pressed
    }

    #[allow(clippy::too_many_lines)]
    pub fn slider(&mut self, id: u64, rect: Rect, value: &mut f32, min: f32, max: f32) -> bool {
        self.slider_labeled_ex(id, rect, "", value, min, max, -1)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slider_labeled(&mut self, id: u64, rect: Rect, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
        self.slider_labeled_ex(id, rect, label, value, min, max, -1)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slider_labeled_ex(&mut self, id: u64, rect: Rect, label: &str, value: &mut f32, min: f32, max: f32, decimals: i32) -> bool {
        self.slider_labeled_styled(id, rect, label, value, min, max, decimals, None, None, None)
    }

    /// Slider 含完整字級 / 高度 override 的版本。所有 override 為 None 時與
    /// `slider_labeled_ex` 行為一致。
    #[allow(clippy::too_many_arguments)]
    pub fn slider_labeled_styled(
        &mut self,
        id: u64,
        rect: Rect,
        label: &str,
        value: &mut f32,
        min: f32,
        max: f32,
        decimals: i32,
        label_font_override: Option<f32>,
        value_font_override: Option<f32>,
        max_height_override: Option<f32>,
    ) -> bool {
        let min_value = min.min(max);
        let max_value = min.max(max);
        let radius = self.theme.radius;

        // 高度上限可被 override；預設 40px 對齊 C++ 原版行為
        let cap = max_height_override.unwrap_or(40.0);
        let effective_h = rect.h.min(cap);
        let rect = Rect::new(rect.x, rect.y, rect.w, effective_h);

        // Font sizing（可被 override）
        let label_font = label_font_override
            .unwrap_or_else(|| (rect.h * 0.36).clamp(13.0, 24.0));
        let value_font = value_font_override
            .unwrap_or_else(|| (label_font - 0.5_f32).max(12.0));
        let value_padding = (rect.h * 0.15).clamp(6.0, 12.0);
        let value_box_w = (rect.h * 1.8).clamp(64.0, 128.0);
        let value_box = Rect::new(
            rect.x + rect.w - value_box_w - value_padding,
            rect.y + value_padding,
            value_box_w,
            rect.h - value_padding * 2.0,
        );

        let hovered = self.is_hovered(&rect);
        let _value_hovered = value_box.contains(self.input.mouse_x, self.input.mouse_y);

        // 互動：主動滑桿狀態機
        let mut changed = false;
        if hovered && self.input.mouse_pressed && !_value_hovered {
            self.active_slider_id = id;
        }

        if self.active_slider_id == id {
            if self.input.mouse_down {
                let t = ((self.input.mouse_x - rect.x) / rect.w).clamp(0.0, 1.0);
                let new_value = min_value + (max_value - min_value) * t;
                if (new_value - *value).abs() > 1e-6 {
                    * 值=新值；
                    changed = true;
                }
            }
            if !self.input.mouse_down || self.input.mouse_released {
                self.active_slider_id = 0;
            }
        }

        // 填充計算匹配C++
        let range = (max_value - min_value).max(1e-6);
        let t = ((*value - min_value) / range).clamp(0.0, 1.0);
        let inner_x = rect.x + 1.0;
        let inner_y = rect.y + 1.0;
        let inner_w = (rect.w - 2.0).max(0.0);
        let inner_h = (rect.h - 2.0).max(0.0);
        let thumb_w = (rect.h * 0.24).clamp(4.0, 10.0);
        let _thumb_center_x = inner_x + inner_w * t;
        let thumb_x = (_thumb_center_x - thumb_w * 0.5).clamp(inner_x, inner_x + (inner_w - thumb_w).max(0.0));
        let mut fill_right = inner_x + inner_w * t;
        if t > 1e-4 {
            fill_right = (inner_x + inner_w).min(fill_right + thumb_w * 0.25);
        }
        if t >= 0.9999 {
            fill_right = inner_x + inner_w;
        }
        let fill = Rect::new(inner_x, inner_y, (fill_right - inner_x).max(0.0), inner_h);
        let thumb = Rect::new(thumb_x, inner_y, thumb_w, inner_h);

        // 運動
        let is_active = self.active_slider_id == id;
        let thumb_hovered = thumb.contains(self.input.mouse_x, self.input.mouse_y);
        let slider_m = self.motion_ex(
            context_hash_mix(id, 0x1a2b3c4d5e6f7003),
            hovered, is_active && self.input.mouse_down, false, is_active,
        );
        let thumb_m = self.motion_ex(
            context_hash_mix(id, 0x1a2b3c4d5e6f7004),
            thumb_hovered, is_active && self.input.mouse_down, false, is_active,
        );
        let slider_hover_v = Self::snap_visual_motion(slider_m.hover, 1.0 / 48.0);
        let _slider_active_v = Self::snap_visual_motion(slider_m.active, 1.0 / 40.0);
        let thumb_hover_v = Self::snap_visual_motion(thumb_m.hover, 1.0 / 48.0);
        let thumb_active_v = Self::snap_visual_motion(thumb_m.active, 1.0 / 36.0);

        // 背景
        let panel = self.theme.panel;
        let secondary = self.theme.secondary;
        let primary = self.theme.primary;
        let outline_col = self.theme.outline;

        self.paint_filled_rect(rect, mix(secondary, panel, slider_hover_v * 0.05), radius);

        // 填充欄
        if fill.w > 0.0 && fill.h > 0.0 {
            let fill_radius = (radius - 1.0).min(fill.h * 0.5).min(fill.w * 0.5).max(0.0);
            self.paint_filled_rect(fill, mix(primary, secondary, 0.75), fill_radius);
        }

        // 大綱
        self.paint_outline_rect(
            rect,
            mix(outline_col, primary, slider_hover_v * 0.44 + _slider_active_v * 0.16),
            radius, 1.0 + slider_hover_v * 0.08,
        );

        // 大拇指
        let thumb_radius = (radius - 1.0).min(thumb.w.min(thumb.h) * 0.5).max(0.0);
        let thumb_color = if is_active {
            mix(primary, panel, 0.18)
        } else if thumb_hovered {
            mix(primary, panel, 0.10)
        } else {
            primary
        };
        let thumb_scale = 1.0 + thumb_active_v * 0.05;
        let visual_thumb = context_scale_rect_from_center(&thumb, thumb_scale, thumb_scale);
        self.paint_soft_glow(visual_thumb, primary, thumb_radius, thumb_hover_v * 0.14 + thumb_active_v * 0.18, 3.6);
        self.paint_filled_rect(visual_thumb, thumb_color, thumb_radius);

        // 標籤文字（符合 C++ add_text(label, ..., theme_.text, label_font, Left) — 無 Clip_rect）
        if !label.is_empty() {
            let text_col = self.theme.text;
            self.paint_text(
                Rect::new(rect.x + value_padding, rect.y,
                           rect.w - value_box.w - value_padding * 2.0, rect.h),
                label, label_font, text_col, TextAlign::Left,
            );
        }

        // 解析小數位數（與 C++ 行為一致）
        let value_decimals = if decimals >= 0 {
            (decimals as usize).min(4)
        } else {
            let span = (max_value - min_value).abs();
            if span <= 1.0 { 2 } else if span <= 10.0 { 1 } else { 0 }
        };

        // Value box 作為可編輯的 numeric input
        let value_box_id = context_hash_mix(id, 0x1a2b3c4d5e6f7006);
        let filter = if value_decimals == 0 { CharFilter::Int } else { CharFilter::Float };

        let was_editing_value = self.focus_id == value_box_id;
        let default_text = format!("{:.prec$}", *value, prec = value_decimals);
        let mut buf = if was_editing_value {
            self.slider_edit_buffers
                .get(&value_box_id)
                .cloned()
                .unwrap_or_else(|| default_text.clone())
        } else {
            default_text
        };

        self.text_input_field_numeric_styled(value_box_id, value_box, &mut buf, value_font, filter);

        // Focus 結束 → parse + clamp 回寫；編輯中 → 保存 buffer
        let still_editing = self.focus_id == value_box_id;
        if was_editing_value && !still_editing {
            if let Ok(parsed) = buf.trim().parse::<f32>() {
                let new_value = parsed.clamp(min_value, max_value);
                if (new_value - *value).abs() > 1e-6 {
                    * 值=新值；
                    changed = true;
                }
            }
            self.slider_edit_buffers.remove(&value_box_id);
        } else if still_editing {
            self.slider_edit_buffers.insert(value_box_id, buf);
        }

        changed
    }

    pub fn progress(&mut self, id: u64, rect: Rect, label: &str, value: f32, height: f32) {
        let ratio = value.clamp(0.0, 1.0);

        // 動畫比例
        let animated_ratio = self.animated_value(id, ratio);

        // 文字大小調整
        let label_h = (height * 1.6).clamp(14.0, 26.0);
        let text_gap = 8.0_f32.max(label_h + 4.0);

        // 標籤文字（左 70%）
        if !label.is_empty() {
            let muted = self.theme.muted_text;
            self.paint_text(
                Rect::new(rect.x, rect.y, rect.w * 0.7, label_h),
                label, label_h, muted, TextAlign::Left,
            );

            // 百分比文字（右30%）
            let pct = format!("{:.0}%", ratio * 100.0);
            let text_col = self.theme.text;
            self.paint_text(
                Rect::new(rect.x + rect.w * 0.7, rect.y, rect.w * 0.3, label_h),
                &pct, label_h, text_col, TextAlign::Right,
            );
        }

        // 追蹤
        let track = Rect::new(rect.x, rect.y + text_gap, rect.w, height.max(4.0));
        let track_radius = track.h * 0.5;
        let track_col = self.theme.track;
        self.paint_filled_rect(track, track_col, track_radius);

        // 填滿 1px 內邊距
        let fill = Rect::new(
            track.x + 1.0,
            track.y + 1.0,
            (track.w * animated_ratio - 2.0).max(0.0),
            (track.h - 2.0).max(0.0),
        );
        if fill.w > 0.0 && fill.h > 0.0 {
            let fill_radius = (track_radius - 1.0).min(fill.h * 0.5).min(fill.w * 0.5).max(0.0);
            let fill_col = self.theme.track_fill;
            self.paint_soft_glow(fill, fill_col, fill_radius, animated_ratio.min(1.0) * 0.22, 4.5);
            self.paint_filled_rect(fill, fill_col, fill_radius);
        }
    }

    /// 無標籤的簡單進度條（向後相容）
    pub fn progress_bar(&mut self, rect: Rect, value: f32) {
        let ratio = value.clamp(0.0, 1.0);
        let track_radius = rect.h * 0.5;
        self.paint_filled_rect(rect, self.theme.track, track_radius);
        let fill = Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            (rect.w * ratio - 2.0).max(0.0),
            (rect.h - 2.0).max(0.0),
        );
        if fill.w > 0.0 && fill.h > 0.0 {
            let fill_radius = (track_radius - 1.0).min(fill.h * 0.5).min(fill.w * 0.5).max(0.0);
            self.paint_filled_rect(fill, self.theme.track_fill, fill_radius);
        }
    }

    // ── 文字測量 ──

    pub fn measure_text(&self, text: &str, font_size: f32) -> f32 {
        if let Some(ref measurer) = self.text_measurer {
            measurer.measure_width(text, font_size)
        } else {
            // 近似值：每個字元約 0.5 em            text.len() as f32 * font_size * 0.5
        }
    }

    // ── 吸氣劑 ──

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    pub fn text_arena(&self) -> &[u8] {
        &self.text_arena
    }

    pub fn brush_payloads(&self) -> &[Brush] {
        &self.brush_payloads
    }

    pub fn transform_payloads(&self) -> &[Transform3D] {
        &self.transform_payloads
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_corner_radius(&mut self, radius: f32) {
        self.theme.radius = radius.clamp(0.0, 28.0);
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    /// 以程式設計方式將鍵盤焦點設定為具有給定 id 的小工具。    pub fn set_focus(&mut self, id: u64) {
        self.focus_id = id;
    }

    /// 檢查特定小部件目前是否具有鍵盤焦點。
    pub fn has_focus(&self, id: u64) -> bool {
        self.focus_id == id
    }

    pub fn viewport_size(&self) -> (f32, f32) {
        (self.viewport_w, self.viewport_h)
    }

    pub fn dpi_scale(&self) -> f32 {
        self.dpi_scale
    }

    // ── 記憶資產──
    pub fn register_memory_asset(&mut self, name: &str, data: Vec<u8>) {
        if name.is_empty() {
            return;
        }
        if data.is_empty() {
            self.memory_assets.remove(name);
        } else {
            self.memory_assets.insert(name.to_string(), Arc::new(data));
        }
    }

    pub fn find_memory_asset(&self, name: &str) -> Option<Arc<Vec<u8>>> {
        self.memory_assets.get(name).cloned()
    }

    pub fn resolve_memory_asset_uri(&self, uri: &str) -> Option<Arc<Vec<u8>>> {
        let scheme = "asset://";
        if let Some(name) = uri.strip_prefix(scheme) {
            self.find_memory_asset(name)
        } else {
            None
        }
    }

    pub fn memory_asset_uri(name: &str) -> String {
        format!("asset://{}", name)
    }

    // ── 滾動區域 ──
    pub fn begin_scroll_area(&mut self, id: u64, viewport: Rect) -> f32 {
        let state = self.scroll_states.entry(id).or_default();
        state.last_touched_frame = self.frame_index;
        let scroll = state.scroll;

        self.push_clip(viewport);
        self.push_layout_rect(Rect::new(viewport.x, viewport.y - scroll, viewport.w, 100000.0));
        scroll
    }

    pub fn end_scroll_area(&mut self, id: u64, viewport: &Rect) {
        let content_h = self.cursor_y() - (viewport.y - self.scroll_states.get(&id).map(|s| s.scroll).unwrap_or(0.0));
        self.pop_layout_rect();
        self.pop_clip();

        let max_scroll = (content_h - viewport.h).max(0.0);

        let hovered = viewport.contains(self.input.mouse_x, self.input.mouse_y);
        let wheel_y = self.input.mouse_wheel_y;

        if let Some(state) = self.scroll_states.get_mut(&id) {
            state.content_height = content_h;

            // 滑鼠滾輪
            if hovered {
                state.scroll -= wheel_y * 40.0;
            }

            // 慣性
            state.scroll += state.velocity;
            state.velocity *= 0.92;

            state.scroll = state.scroll.clamp(0.0, max_scroll);
        }
    }

    // ── 文字輸入 ──

    pub fn text_input_field(&mut self, id: u64, rect: Rect, text: &mut String) -> bool {
        self.text_input_field_ex(id, rect, "", text, "")
    }

    /// 只讀可選擇文字欄位。支援滑鼠選擇和Ctrl+C複製，
    /// 但所有突變（鍵入、貼上、剪下、退格、刪除）都將被丟棄。
    pub fn selectable_text(&mut self, id: u64, rect: Rect, text: &str) {
        let mut buf = text.to_string();
        self.text_input_impl(id, rect, "", &mut buf, "", true, None, None, CharFilter::Any);
    }

    /// 具有自訂字體大小和顏色的唯讀可選文字。    pub fn selectable_text_styled(&mut self, id: u64, rect: Rect, text: &str, font_size: f32, color: Color) {
        let mut buf = text.to_string();
        self.text_input_impl(id, rect, "", &mut buf, "", true, Some(font_size), Some(color), CharFilter::Any);
    }

    pub fn selectable_text_ex(&mut self, id: u64, rect: Rect, label: &str, text: &str) {
        let mut buf = text.to_string();
        self.text_input_impl(id, rect, label, &mut buf, "", true, None, None, CharFilter::Any);
    }

    #[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
    pub fn text_input_field_ex(&mut self, id: u64, rect: Rect, label: &str, text: &mut String, placeholder: &str) -> bool {
        self.text_input_impl(id, rect, label, text, placeholder, false, None, None, CharFilter::Any)
    }

    /// 具有自訂字體大小的文字輸入欄位。
    #[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
    pub fn text_input_field_styled(&mut self, id: u64, rect: Rect, label: &str, text: &mut String, placeholder: &str, font_size: f32) -> bool {
        self.text_input_impl(id, rect, label, text, placeholder, false, Some(font_size), None, CharFilter::Any)
    }

    /// 純數位文字輸入欄位。在鍵入和貼上過程中拒絕非數字字元。    /// 使用“CharFilter::Int”表示整數（“0-9”、“-”）或“CharFilter::Float”（“0-9”、“-”、“.”）。
    #[allow(clippy::too_many_arguments)]
    pub fn text_input_field_numeric_styled(&mut self, id: u64, rect: Rect, text: &mut String, font_size: f32, filter: CharFilter) -> bool {
        self.text_input_impl(id, rect, "", text, "", false, Some(font_size), None, filter)
    }

    #[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
    fn text_input_impl(&mut self, id: u64, rect: Rect, label: &str, text: &mut String, placeholder: &str,
                        read_only: bool, font_override: Option<f32>, color_override: Option<Color>,
                        filter: CharFilter) -> bool {
        // 字體大小匹配 C++
        let label_font = (rect.h * 0.40).clamp(13.0, 24.0);
        let value_font = font_override.unwrap_or((label_font - 0.5_f32).max(12.0));
        let input_padding = (rect.h * 0.18).clamp(6.0, 12.0);
        let has_label = !label.is_empty();
        let content_padding = input_padding;
        let is_multiline = text.contains('\n') || rect.h > 80.0;

        let label_rect = if has_label {
            Rect::new(rect.x, rect.y, rect.w * 0.34, rect.h)
        } else {
            Rect::new(rect.x, rect.y, 0.0, rect.h)
        };
        let input_rect = if is_multiline {
            rect
        } else if has_label {
            Rect::new(rect.x + rect.w * 0.36, rect.y + input_padding * 0.5,
                       rect.w * 0.64, rect.h - input_padding)
        } else {
            Rect::new(rect.x, rect.y + input_padding * 0.5,
                       rect.w, rect.h - input_padding)
        };

        // 標籤
        if has_label {
            let text_col = self.theme.text;
            self.paint_text(label_rect, label, label_font, text_col, TextAlign::Left);
        }

        let hovered = input_rect.contains(self.input.mouse_x, self.input.mouse_y);

        if hovered && self.input.mouse_pressed {
            self.focus_id = id;
        }

        let editing = self.focus_id == id;

        // ── 文字區域狀態（遊標/選擇）──
        let ta_state = self.text_area_states.entry(id).or_default();
        ta_state.last_touched_frame = self.frame_index;
        // 將遊標/sel_start 固定到文字長度
        ta_state.cursor = ta_state.cursor.min(text.len());
        ta_state.sel_start = ta_state.sel_start.min(text.len());
        // 將狀態提取到本地變數以避免借用問題
        let mut cursor = ta_state.cursor;
        let mut sel_start = ta_state.sel_start;
        let mut dragging = ta_state.dragging;
        let mut scroll = ta_state.scroll;
        let mut preferred_x = ta_state.preferred_x;

        // ── 計算佈局參數 ──
        let text_col = color_override.unwrap_or(self.theme.text);
        let muted_col = self.theme.muted_text;
        let render_font = if let Some(fs) = font_override {
            fs
        } else if is_multiline {
            (input_rect.h * 0.13).clamp(13.0, 22.0)
        } else {
            value_font
        };
        let text_pad = if is_multiline {
            (input_rect.h * 0.03).clamp(6.0, 10.0)
        } else {
            content_padding
        };
        let line_h = render_font + 5.0;

        // 輸入鑲邊（跳過只讀以保留純文字外觀）
        let input_bg = self.theme.input_bg;
        let chrome_radius = (self.theme.radius - 2.0).max(0.0);
        if !read_only {
            let chrome_id = if is_multiline { 0x1a2b3c4d5e6f7010 } else { 0x1a2b3c4d5e6f7007 };
            self.draw_input_chrome(
                context_hash_mix(id, chrome_id),
                input_rect, hovered || editing, editing,
                if is_multiline { mix(input_bg, self.theme.secondary, 0.08) } else { input_bg },
                chrome_radius, if editing { 1.2 } else { 1.0 },
            );
        }

        // ── 計算多行換行 ──
        let content_w = if is_multiline {
            (input_rect.w - text_pad * 2.0 - 2.0).max(24.0)
        } else {
            (input_rect.w - text_pad * 2.0).max(0.0)
        };
        let lines: Vec<(usize, usize)> = if is_multiline {
            self.compute_wrapped_lines(text, render_font, content_w)
        } else {
            vec![(0, text.len())]
        };

        // ── Helper：命中測試距離像素位置的位元組偏移量──
        let content_top = if is_multiline { input_rect.y + text_pad - scroll } else { input_rect.y };
        let content_left = if is_multiline { input_rect.x + text_pad } else { input_rect.x + text_pad };

        let hit_test = |mx: f32, my: f32, text_bytes: &str, lines: &[(usize, usize)], measurer: &Option<TextMeasurer>| -> usize {
            let line_idx = if is_multiline {
                let row = ((my - content_top) / line_h).floor() as i32;
                (row.max(0) as usize).min(lines.len().saturating_sub(1))
            } else {
                0
            };
            let (line_start, line_len) = lines[line_idx];
            let line_str = &text_bytes[line_start..line_start + line_len];
            let rel_x = mx - content_left;
            // 遍歷字元以查找最近的位元組偏移量
            let mut x_acc = 0.0_f32;
            let mut best_offset = line_start;
            for (i, ch) in line_str.char_indices() {
                let adv = if let Some(ref m) = measurer {
                    m.measure_char_advance(ch, render_font)
                } else {
                    render_font * 0.5
                };
                if rel_x < x_acc + adv * 0.5 {
                    best_offset = line_start + i;
                    return best_offset;
                }
                x_acc += adv;
                best_offset = line_start + i + ch.len_utf8();
            }
            best_offset
        };

        // ── 滑鼠交互 ──
        let mut changed = false;
        if editing {
            // 按下滑鼠：設定遊標位置
            if self.input.mouse_pressed && hovered {
                let pos = hit_test(self.input.mouse_x, self.input.mouse_y, text, &lines, &self.text_measurer);
                cursor = pos;
                if !self.input.key_shift {
                    sel_start = pos;
                }
                dragging = true;
                preferred_x = -1.0;
            }
            // 滑鼠拖曳：擴展選擇範圍
            if self.input.mouse_down && dragging && !self.input.mouse_pressed {
                let pos = hit_test(self.input.mouse_x, self.input.mouse_y, text, &lines, &self.text_measurer);
                cursor = pos;
                preferred_x = -1.0;
            }
            if self.input.mouse_released {
                dragging = false;
            }

            let has_selection = cursor != sel_start;
            let sel_min = cursor.min(sel_start);
            let sel_max = cursor.max(sel_start);

            // ── 鍵盤導航 ──
            if self.input.key_left {
                if cursor > 0 {
                    // 後移一個字元（支援 UTF-8）
                    let mut pos = cursor - 1;
                    while pos > 0 && !text.is_char_boundary(pos) { pos -= 1; }
                    cursor = pos;
                }
                if !self.input.key_shift { sel_start = cursor; }
                preferred_x = -1.0;
            }
            if self.input.key_right {
                if cursor < text.len() {
                    let mut pos = cursor + 1;
                    while pos < text.len() && !text.is_char_boundary(pos) { pos += 1; }
                    cursor = pos;
                }
                if !self.input.key_shift { sel_start = cursor; }
                preferred_x = -1.0;
            }
            if self.input.key_home {
                // 移動到目前行的開頭
                let line_idx = find_line_for_offset(&lines, cursor);
                cursor = lines[line_idx].0;
                if !self.input.key_shift { sel_start = cursor; }
                preferred_x = -1.0;
            }
            if self.input.key_end {
                // 移至目前行末尾
                let line_idx = find_line_for_offset(&lines, cursor);
                let (start, len) = lines[line_idx];
                cursor = start + len;
                if !self.input.key_shift { sel_start = cursor; }
                preferred_x = -1.0;
            }
            if (self.input.key_up || self.input.key_down) && is_multiline {
                let line_idx = find_line_for_offset(&lines, cursor);
                // 如果未設定則計算preferred_x
                if preferred_x < 0.0 {
                    let (ls, _) = lines[line_idx];
                    preferred_x = self.measure_text(&text[ls..cursor], render_font);
                }
                let new_line = if self.input.key_up {
                    line_idx.saturating_sub(1)
                } else {
                    (line_idx + 1).min(lines.len() - 1)
                };
                if new_line != line_idx {
                    // 在最接近preferred_x的新行中尋找位元組偏移量
                    let (ls, ll) = lines[new_line];
                    let line_str = &text[ls..ls + ll];
                    let mut x_acc = 0.0_f32;
                    let mut new_cursor = ls;
                    for (i, ch) in line_str.char_indices() {
                        let adv = if let Some(ref m) = self.text_measurer {
                            m.measure_char_advance(ch, render_font)
                        } else {
                            render_font * 0.5
                        };
                        if x_acc + adv * 0.5 > preferred_x {
                            new_cursor = ls + i;
                            break;
                        }
                        x_acc += adv;
                        new_cursor = ls + i + ch.len_utf8();
                    }
                    cursor = new_cursor;
                }
                if !self.input.key_shift { sel_start = cursor; }
                // 不要在上/下重設preferred_x
            }

            // ── 選擇全部 ──
            if self.input.key_select_all {
                sel_start = 0;
                cursor = text.len();
            }

            // ── 複製 ──
            if self.input.key_copy && has_selection {
                self.input.clipboard_out = text[sel_min..sel_max].to_string();
            }

            // ── 剪切（唯讀：僅複製，不突變）—
            if self.input.key_cut && has_selection {
                self.input.clipboard_out = text[sel_min..sel_max].to_string();
                if !read_only {
                    text.replace_range(sel_min..sel_max, "");
                    cursor = sel_min;
                    sel_start = sel_min;
                    changed = true;
                    preferred_x = -1.0;
                }
            }

            if !read_only {
                // ── 貼 ──
                if self.input.key_paste && !self.input.clipboard_text.is_empty() {
                    let paste_text = self.input.clipboard_text.clone();
                    // 單行：去除換行符
                    let paste_text = if !is_multiline {
                        paste_text.replace('\n', " ").replace('\r', "")
                    } else {
                        paste_text.replace('\r', "")
                    };
                    let paste_text = filter.filter_str(&paste_text);
                    if !paste_text.is_empty() {
                        if has_selection {
                            text.replace_range(sel_min..sel_max, &paste_text);
                            cursor = sel_min + paste_text.len();
                        } else {
                            text.insert_str(cursor, &paste_text);
                            cursor += paste_text.len();
                        }
                        sel_start = cursor;
                        changed = true;
                        preferred_x = -1.0;
                    }
                }

                // ── 文字輸入（打字）──
                if !self.input.text_input.is_empty() {
                    let typed = filter.filter_str(&self.input.text_input);
                    if !typed.is_empty() {
                        if has_selection {
                            text.replace_range(sel_min..sel_max, &typed);
                            cursor = sel_min + typed.len();
                        } else {
                            text.insert_str(cursor, &typed);
                            cursor += typed.len();
                        }
                        sel_start = cursor;
                        changed = true;
                        preferred_x = -1.0;
                    }
                }

                // ── 退格鍵 ──
                if self.input.key_backspace {
                    if has_selection {
                        text.replace_range(sel_min..sel_max, "");
                        cursor = sel_min;
                        sel_start = sel_min;
                        changed = true;
                    } else if cursor > 0 {
                        let mut prev = cursor - 1;
                        while prev > 0 && !text.is_char_boundary(prev) { prev -= 1; }
                        text.replace_range(prev..cursor, "");
                        cursor = prev;
                        sel_start = prev;
                        changed = true;
                    }
                    preferred_x = -1.0;
                }

                // ── 刪除 ──
                if self.input.key_delete {
                    if has_selection {
                        text.replace_range(sel_min..sel_max, "");
                        cursor = sel_min;
                        sel_start = sel_min;
                        changed = true;
                    } else if cursor < text.len() {
                        let mut next = cursor + 1;
                        while next < text.len() && !text.is_char_boundary(next) { next += 1; }
                        text.replace_range(cursor..next, "");
                        changed = true;
                    }
                    preferred_x = -1.0;
                }

                // ── 輸入（僅限多行）──
                if self.input.key_enter {
                    if is_multiline {
                        if has_selection {
                            text.replace_range(sel_min..sel_max, "\n");
                            cursor = sel_min + 1;
                        } else {
                            text.insert(cursor, '\n');
                            cursor += 1;
                        }
                        sel_start = cursor;
                        changed = true;
                        preferred_x = -1.0;
                    } else {
                        // 單行：失去 Enter 焦點
                        self.focus_id = 0;
                    }
                }
            } // end if !read_only

            // ── 逃脫：失去焦點──
            if self.input.key_escape {
                self.focus_id = 0;
            }

            // ── 點選外部：失去焦點──
            if self.input.mouse_pressed && !hovered {
                self.focus_id = 0;
            }
        }

        // 強制 256 個字元限制（僅限單行，匹配 C++）
        if !is_multiline && text.len() > 256 {
            text.truncate(256);
            cursor = cursor.min(text.len());
            sel_start = sel_start.min(text.len());
        }

        // ── 編輯後重新計算換行 ──
        let lines: Vec<(usize, usize)> = if is_multiline {
            self.compute_wrapped_lines(text, render_font, content_w)
        } else {
            vec![(0, text.len())]
        };

        // ── 捲動跟隨遊標（多行）──
        if is_multiline && editing {
            let viewport_h = (input_rect.h - text_pad * 2.0).max(24.0);
            let total_h = lines.len() as f32 * line_h;
            let max_scroll = (total_h - viewport_h).max(0.0);

            // 滑鼠滾輪滾動
            if hovered && self.input.mouse_wheel_y.abs() > 0.001 {
                scroll -= self.input.mouse_wheel_y * line_h * 3.0;
            }

            let cursor_line = find_line_for_offset(&lines, cursor);
            let cursor_y_in_content = cursor_line as f32 * line_h;
            // 捲動以保持遊標可見            if cursor_y_in_content < scroll {
                scroll = cursor_y_in_content;
            }
            if cursor_y_in_content + line_h > scroll + viewport_h {
                scroll = cursor_y_in_content + line_h - viewport_h;
            }
            scroll = scroll.clamp(0.0, max_scroll);
        }

        // ── 渲染 ──
        if is_multiline {
            let viewport_h = (input_rect.h - text_pad * 2.0).max(24.0);
            let content_clip = Rect::new(
                input_rect.x + text_pad, input_rect.y + text_pad,
                content_w, viewport_h,
            );
            let total_h = lines.len() as f32 * line_h;
            let max_scroll = (total_h - viewport_h).max(0.0);

            // 捲軸
            let scrollbar_w = 8.0_f32;
            let track_rect = Rect::new(
                input_rect.x + input_rect.w - text_pad - scrollbar_w,
                input_rect.y + text_pad,
                scrollbar_w, viewport_h,
            );
            let thumb_h = if max_scroll > 0.0 {
                (viewport_h * (viewport_h / total_h.max(viewport_h + 1.0))).max(18.0)
            } else {
                viewport_h
            };
            let thumb_y = if max_scroll > 0.0 {
                track_rect.y + (scroll / max_scroll) * (viewport_h - thumb_h)
            } else {
                track_rect.y
            };
            // 當內容適合時隱藏滾動條以只讀
            if !read_only || max_scroll > 0.0 {
                let secondary = self.theme.secondary;
                let panel = self.theme.panel;
                let primary = self.theme.primary;
                self.paint_filled_rect(track_rect, mix(secondary, panel, 0.45), 3.0);
                self.paint_filled_rect(Rect::new(track_rect.x, thumb_y, track_rect.w, thumb_h), mix(primary, panel, 0.40), 3.0);
            }

            // 選擇突出顯示 + 文字行
            let sel_min = cursor.min(sel_start);
            let sel_max = cursor.max(sel_start);
            let sel_color = rgba(self.theme.primary.r, self.theme.primary.g, self.theme.primary.b, 0.35);
            let mut y = input_rect.y + text_pad - scroll;
            for (line_start, line_len) in &lines {
                let line_end = *line_start + *line_len;
                // 為這條線繪製選擇突出顯示
                if editing && sel_min != sel_max && sel_min < line_end && sel_max > *line_start {
                    let hl_start = sel_min.max(*line_start);
                    let hl_end = sel_max.min(line_end);
                    let x_start = self.measure_text(&text[*line_start..hl_start], render_font);
                    let x_end = self.measure_text(&text[*line_start..hl_end], render_font);
                    let hl_rect = Rect::new(
                        content_clip.x + x_start, y,
                        (x_end - x_start).max(0.0), line_h,
                    );
                    self.paint_filled_rect_clipped(hl_rect, sel_color, 0.0, Some(&content_clip));
                }
                // 繪製文字
                if *line_len > 0 {
                    let line_text = &text[*line_start..line_end];
                    self.paint_text_clipped(
                        Rect::new(content_clip.x, y, content_w, line_h),
                        line_text, render_font, text_col, TextAlign::Left, Some(&content_clip),
                    );
                }
                y += line_h;
            }

            // 閃爍插入符號
            if editing {
                let blink = (self.frame_index / 30) % 2 == 0;
                if blink {
                    let cursor_line = find_line_for_offset(&lines, cursor);
                    let (ls, _) = lines[cursor_line];
                    let caret_x = content_clip.x + self.measure_text(&text[ls..cursor], render_font);
                    let caret_y = input_rect.y + text_pad - scroll + cursor_line as f32 * line_h;
                    let caret_h = render_font + 2.0;
                    let caret_rect = Rect::new(caret_x, caret_y, 1.5, caret_h);
                    self.paint_filled_rect_clipped(caret_rect, text_col, 0.0, Some(&content_clip));
                }
            }
        } else {
            // ── 單線渲染 ──
            let content_clip = Rect::new(
                input_rect.x + text_pad,
                input_rect.y + 2.0,
                (input_rect.w - text_pad * 2.0).max(0.0),
                (input_rect.h - 4.0).max(0.0),
            );
            let display_text = if text.is_empty() && !placeholder.is_empty() && !editing {
                placeholder
            } else {
                text.as_str()
            };
            let display_color = if text.is_empty() && !placeholder.is_empty() && !editing {
                muted_col
            } else {
                text_col
            };

            // 選擇亮點
            let sel_min = cursor.min(sel_start);
            let sel_max = cursor.max(sel_start);
            if editing && sel_min != sel_max && !text.is_empty() {
                let sel_color = rgba(self.theme.primary.r, self.theme.primary.g, self.theme.primary.b, 0.35);
                let x_start = self.measure_text(&text[..sel_min], render_font);
                let x_end = self.measure_text(&text[..sel_max], render_font);
                let hl_rect = Rect::new(
                    content_clip.x + x_start,
                    input_rect.y + 2.0,
                    (x_end - x_start).max(0.0),
                    input_rect.h - 4.0,
                );
                self.paint_filled_rect_clipped(hl_rect, sel_color, 0.0, Some(&content_clip));
            }

            let text_w = self.measure_text(display_text, render_font);
            let text_rect = Rect::new(content_clip.x, input_rect.y, content_clip.w.max(text_w), input_rect.h);
            self.paint_text_clipped(text_rect, display_text, render_font, display_color, TextAlign::Left, Some(&content_clip));

            // 編輯時閃爍插入符
            if editing {
                let blink = (self.frame_index / 30) % 2 == 0;
                if blink {
                    let caret_x = content_clip.x + self.measure_text(&text[..cursor], render_font);
                    let caret_h = render_font + 2.0;
                    let caret_y = input_rect.y + (input_rect.h - caret_h) * 0.5;
                    self.paint_filled_rect(
                        Rect::new(caret_x, caret_y, 1.5, caret_h),
                        text_col, 0.0,
                    );
                }
            }
        }

        // ── 回寫狀態 ──
        if let Some(ta_state) = self.text_area_states.get_mut(&id) {
            ta_state.cursor = cursor;
            ta_state.sel_start = sel_start;
            ta_state.dragging = dragging;
            ta_state.scroll = scroll;
            ta_state.preferred_x = preferred_x;
        }

        changed
    }

    // ── 輸入唯讀 ──

    pub fn input_readonly(&mut self, id: u64, rect: Rect, label: &str, value: &str) {
        self.input_readonly_ex(id, rect, label, value, false, 1.0, true);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn input_readonly_ex(&mut self, id: u64, rect: Rect, label: &str, value: &str,
                              align_right: bool, value_font_scale: f32, muted: bool) {
        let label_font = (rect.h * 0.40).clamp(13.0, 24.0);
        let base_value_font = (rect.h * 0.44).clamp(12.0, 56.0);
        let scale = value_font_scale.clamp(0.5, 2.2);
        let value_font = (base_value_font * scale).clamp(12.0, 72.0);
        let input_padding = (rect.h * 0.18).clamp(6.0, 12.0);
        let has_label = !label.is_empty();

        let label_rect = Rect::new(rect.x, rect.y,
                                    if has_label { rect.w * 0.34 } else { 0.0 }, rect.h);
        let input_rect = if has_label {
            Rect::new(rect.x + rect.w * 0.36, rect.y + input_padding * 0.5,
                       rect.w * 0.64, rect.h - input_padding)
        } else {
            Rect::new(rect.x, rect.y + input_padding * 0.5,
                       rect.w, rect.h - input_padding)
        };

        if has_label {
            let text_col = self.theme.text;
            self.paint_text(label_rect, label, label_font, text_col, TextAlign::Left);
        }

        let hovered = input_rect.contains(self.input.mouse_x, self.input.mouse_y);
        let input_bg = self.theme.input_bg;
        let secondary = self.theme.secondary;
        let chrome_radius = (self.theme.radius - 2.0).max(0.0);
        self.draw_input_chrome(
            context_hash_mix(id, 0x1a2b3c4d5e6f7008),
            input_rect, hovered, false,
            mix(input_bg, secondary, 0.08), chrome_radius, 1.0,
        );

        // 值文字內容符合C++draw_static_input_content：
        // 文字矩形使用完整的input_rect.y/h，剪輯是content_clip(y+2, h-4)
        let content_clip = Rect::new(
            input_rect.x + input_padding,
            input_rect.y + 2.0,
            (input_rect.w - input_padding * 2.0).max(0.0),
            (input_rect.h - 4.0).max(0.0),
        );
        let text_w = self.measure_text(value, value_font);
        let origin_x = if align_right {
            content_clip.x + content_clip.w - text_w
        } else {
            content_clip.x
        };
        let text_rect = Rect::new(origin_x, input_rect.y, content_clip.w.max(text_w), input_rect.h);
        let text_color = if muted { self.theme.muted_text } else { self.theme.text };
        let align = if align_right { TextAlign::Right } else { TextAlign::Left };
        self.paint_text_clipped(text_rect, value, value_font, text_color, align, Some(&content_clip));
    }

    // ── 下拉式選單 ──

    pub fn dropdown(&mut self, id: u64, rect: Rect, items: &[&str], selected: &mut usize) -> bool {
        let hovered = self.is_hovered(&rect);
        let held = hovered && self.is_mouse_down();

        // 動態字型大小比對 C++
        let header_font = (rect.h * 0.38).clamp(13.0, 24.0);
        let header_pad = (rect.h * 0.28).clamp(10.0, 22.0);
        let indicator_size = (rect.h * 0.34).clamp(10.0, 18.0);

        // 4通道運動
        let m = self.motion_ex(id, hovered, held, false, false);
        let hover_v = Self::snap_visual_motion(m.hover, 1.0 / 48.0);
        let active_v = Self::snap_visual_motion(m.active, 1.0 / 40.0);

        // 填滿顏色匹配C++
        let panel = self.theme.panel;
        let secondary_hover = self.theme.secondary_hover;
        let fill = mix(panel, secondary_hover, hover_v * 0.32 + active_v * 0.08);
        let primary = self.theme.primary;
        let radius = self.theme.radius;

        // 柔和的光芒
        self.paint_soft_glow(rect, primary, radius, active_v * 0.26 + hover_v * 0.10, 5.0);

        self.paint_filled_rect(rect, fill, radius);

        // 大綱
        let outline_color = mix(self.theme.outline, self.theme.focus_ring,
                                 hover_v * 0.20 + active_v * 0.18);
        self.paint_outline_rect(rect, outline_color, radius, 1.0 + hover_v * 0.10);

        // 標籤文字
        let label = if *selected < items.len() { items[*selected] } else { "" };
        let text_rect = Rect::new(rect.x + header_pad, rect.y,
                                   rect.w - header_pad * 2.0 - indicator_size - 4.0, rect.h);
        let text_col = self.theme.text;
        self.paint_text(text_rect, label, header_font, text_col, TextAlign::Left);

        // 具有動畫色彩的 V 形圖案
        let chevron_size = indicator_size;
        let chevron_rect = Rect::new(
            rect.x + rect.w - header_pad - chevron_size,
            rect.y + (rect.h - chevron_size) * 0.5,
            chevron_size, chevron_size,
        );
        let chevron_color = mix(self.theme.muted_text, self.theme.text,
                                 active_v * 0.20 + hover_v * 0.10);
        self.paint_chevron(chevron_rect, chevron_color, 90.0);

        // 點擊循環
        let mut changed = false;
        if hovered && self.input.mouse_pressed && !items.is_empty() {
            * 所選值 = (*所選 + 1) % items.len();
            changed = true;
        }
        changed
    }

    /// 使用可選項目擴展下拉列表，匹配 C++ `internal_begin_dropdown`。
    /// 項目呈現為幽靈按鈕（填滿+輪廓+中心文字）。
    pub fn dropdown_select(
        &mut self, id: u64, rect: Rect, label: &str, open: &mut bool,
        items: &[&str], selected: &mut usize,
        body_height: f32, padding: f32, item_h: f32, item_gap: f32,
    ) -> bool {
        let body_height = body_height.max(36.0);
        let padding = padding.clamp(4.0, 24.0);
        let header_h = 34.0_f32.max(padding * 3.0);
        let header_rect = Rect::new(rect.x, rect.y, rect.w, header_h);

        // 動態調整大小比對 C++
        let header_font = (header_h * 0.38).clamp(13.0, 24.0);
        let header_pad = (header_h * 0.28).clamp(10.0, 22.0);
        let indicator_size = (header_h * 0.34).clamp(10.0, 18.0);

        // 顯示動畫（0 = 關閉，1 = 開啟）
        let reveal_id = context_hash_mix(id, 0x1a2b3c4d5e6f7013);
        let reveal = self.presence(reveal_id, *open);
        let reveal_alpha = 1.0 - (1.0 - reveal) * (1.0 - reveal); // ease-out

        // 標題懸停/單擊
        let header_hovered = self.is_hovered(&header_rect);
        let header_held = header_hovered && self.is_mouse_down();
        if header_hovered && self.input.mouse_pressed {
            * 打開=！ *打開；
        }

        // 標頭 4 通道運動
        let m = self.motion_ex(id, header_hovered, header_held, false, *open);
        let hover_v = Self::snap_visual_motion(m.hover, 1.0 / 48.0);
        let active_v = Self::snap_visual_motion(m.active, 1.0 / 40.0);

        // 標頭鍍鉻
        let panel = self.theme.panel;
        let secondary_hover = self.theme.secondary_hover;
        let primary = self.theme.primary;
        let radius = self.theme.radius;
        let fill = mix(panel, secondary_hover, hover_v * 0.32 + active_v * 0.08);
        self.paint_soft_glow(header_rect, primary, radius, active_v * 0.26 + hover_v * 0.10, 5.0);
        self.paint_filled_rect(header_rect, fill, radius);
        let outline_color = mix(self.theme.outline, self.theme.focus_ring,
                                 hover_v * 0.20 + active_v * 0.18);
        self.paint_outline_rect(header_rect, outline_color, radius, 1.0 + hover_v * 0.10);

        // 標題標籤
        let text_rect = Rect::new(
            header_rect.x + header_pad, header_rect.y,
            header_rect.w - header_pad * 2.0 - indicator_size - 6.0, header_rect.h,
        );
        let text_col = self.theme.text;
        self.paint_text(text_rect, label, header_font, text_col, TextAlign::Left);

        // Chevron：從 0（閉合）旋轉到 π/2（開放），與 C++ 匹配        let chevron_rect = Rect::new(
            header_rect.x + header_rect.w - header_pad - indicator_size,
            header_rect.y + (header_rect.h - indicator_size) * 0.5,
            indicator_size, indicator_size,
        );
        let chevron_color = mix(self.theme.muted_text, self.theme.text,
                                 active_v * 0.20 + hover_v * 0.10);
        let chevron_rotation = reveal * std::f32::consts::FRAC_PI_2;
        let chevron_thickness = (header_h * 0.065).clamp(1.4, 2.4);
        self.paint_chevron_ex(chevron_rect, chevron_color, chevron_rotation, chevron_thickness);

        // 主體（僅當動畫或開啟時）
        let mut changed = false;
        if reveal > 0.01 {
            let body_alpha = (0.16 + reveal_alpha * 0.84).clamp(0.0, 1.0);
            let content_alpha = (0.08 + reveal_alpha * 0.92).clamp(0.0, 1.0);
            let shell_offset_y = (1.0 - reveal) * 8.0;
            let content_offset_y = (1.0 - reveal) * 10.0;

            let body_rect = Rect::new(
                rect.x, header_rect.y + header_h + shell_offset_y,
                rect.w, body_height,
            );

            // 身體外殼：柔光+填充+輪廓
            let body_fill = mix(panel, self.theme.secondary_hover, 0.35);
            let body_fill_a = Color::new(body_fill.r, body_fill.g, body_fill.b, body_fill.a * body_alpha);
            self.paint_soft_glow(body_rect, primary, radius, (active_v * 0.10 + reveal_alpha * 0.14) * body_alpha, 6.0);
            self.paint_filled_rect(body_rect, body_fill_a, radius);
            let body_outline = mix(self.theme.outline, self.theme.focus_ring, active_v * 0.16);
            let outline_a = (0.18 + reveal_alpha * 0.82).clamp(0.0, 1.0) * body_alpha;
            let body_outline_a = Color::new(body_outline.r, body_outline.g, body_outline.b, body_outline.a * outline_a);
            self.paint_outline_rect(body_rect, body_outline_a, radius, 1.0 + active_v * 0.12);

            // 項目作為與 C++ 匹配的幽靈按鈕（填充 + 輪廓 + 中心文字）
            let btn_font = (item_h * 0.38).clamp(13.0, 24.0);
            let btn_x = body_rect.x + padding;
            let btn_w = body_rect.w - padding * 2.0;
            let mut btn_y = body_rect.y + padding + content_offset_y;

            for (i, item_label) in items.iter().enumerate() {
                let btn_rect = Rect::new(btn_x, btn_y, btn_w, item_h);
                let item_hovered = self.is_hovered(&btn_rect) && *open;
                let item_held = item_hovered && self.is_mouse_down();
                let item_id = context_hash_mix(id, 0x1a2b3c4d5e6f8000 + i as u64);

                // 幽靈按鈕：4通道動作
                let im = self.motion_ex(item_id, item_hovered, item_held, false, false);
                let ih = Self::snap_visual_motion(im.hover, 1.0 / 48.0);
                let ia = Self::snap_visual_motion(im.active, 1.0 / 40.0);

                // 幽靈按鈕鍍鉻：填充+輪廓（匹配C++button.ghost()）
                let btn_fill = mix(panel, secondary_hover, ih * 0.32 + ia * 0.08);
                let btn_fill_a = Color::new(btn_fill.r, btn_fill.g, btn_fill.b, btn_fill.a * content_alpha);
                self.paint_filled_rect(btn_rect, btn_fill_a, radius);
                let btn_outline = mix(self.theme.outline, self.theme.focus_ring,
                                       ih * 0.20 + ia * 0.18);
                let btn_outline_a = Color::new(btn_outline.r, btn_outline.g, btn_outline.b, btn_outline.a * content_alpha);
                self.paint_outline_rect(btn_rect, btn_outline_a, radius, 1.0);

                // 居中對齊文字（匹配 C++ 按鈕文字）
                let btn_text_color = self.theme.text;
                let btn_text_a = Color::new(btn_text_color.r, btn_text_color.g, btn_text_color.b, btn_text_color.a * content_alpha);
                self.paint_text(btn_rect, item_label, btn_font, btn_text_a, TextAlign::Center);

                // 點選選擇
                if item_hovered && self.input.mouse_pressed {
                    * 選定=我；
                    * 打開=假；
                    changed = true;
                }

                btn_y += item_h + item_gap;
            }
        }

        // 開啟時如果在外部按一下則關閉
        if *open && self.input.mouse_pressed && !self.is_hovered(&rect) {
            * 打開=假；
        }

        changed
    }

    // ── 標籤 ──

    pub fn tab_bar(&mut self, id: u64, rect: Rect, labels: &[&str], selected: &mut usize) -> bool {
        let n = labels.len();
        if n == 0 {
            return false;
        }
        let tab_w = rect.w / n as f32;
        let mut changed = false;

        for (i, label) in labels.iter().enumerate() {
            let tab_rect = Rect::new(rect.x + i as f32 * tab_w, rect.y, tab_w, rect.h);
            let is_selected = i == *selected;
            let hovered = self.is_hovered(&tab_rect);
            let held = hovered && self.is_mouse_down();
            let tab_id = context_hash_mix(id, i as u64);

            // 動態字型大小比對 C++
            let text_size = (tab_rect.h * 0.42).clamp(13.0, 26.0);

            // 4通道運動
            let m = self.motion_ex(tab_id, hovered, held, false, is_selected);
            let hover_v = Self::snap_visual_motion(m.hover, 1.0 / 48.0);
            let press_v = Self::snap_visual_motion(m.press, 1.0 / 36.0);
            let active_v = Self::snap_visual_motion(m.active, 1.0 / 40.0);

            // 填滿顏色（3層混合匹配C++）
            let primary = self.theme.primary;
            let secondary = self.theme.secondary;
            let panel = self.theme.panel;
            let mut fill = mix(secondary, mix(primary, panel, 0.72), active_v);
            fill = mix(fill, if is_selected { primary } else { self.theme.secondary_hover },
                       hover_v * if is_selected { 0.16 } else { 0.28 });
            fill = mix(fill, self.theme.secondary_active,
                       press_v * if is_selected { 0.18 } else { 0.32 });

            // 尺度變換
            let tab_radius = (self.theme.radius - 2.0).max(0.0);
            let visual_scale = 1.0 + active_v * 0.004 - press_v * 0.014;
            let mut visual_rect = context_scale_rect_from_center(&tab_rect, visual_scale, visual_scale);
            visual_rect = context_translate_rect(&visual_rect, 0.0, press_v * 0.3);

            // 柔和的光芒
            self.paint_soft_glow(visual_rect, primary, tab_radius, active_v * 0.34 + hover_v * 0.06, 5.0);

            // 充滿
            self.paint_filled_rect(visual_rect, fill, tab_radius);

            // 大綱
            let outline_base = mix(self.theme.outline, panel, 0.6);
            let outline_color = mix(outline_base, primary, active_v * 0.78 + hover_v * 0.18);
            self.paint_outline_rect(visual_rect, outline_color, tab_radius,
                                     1.0 + active_v * 0.36 + hover_v * 0.06);

            // 文字顏色與動畫混合
            let text_color = mix(self.theme.muted_text, self.theme.text, 0.38 + active_v * 0.62);
            self.paint_text(visual_rect, label, text_size, text_color, TextAlign::Center);

            if hovered && self.input.mouse_pressed && !is_selected {
                * 選定=我；
                changed = true;
            }
        }

        changed
    }

    // ── 全域阿爾法──

    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.global_alpha = alpha.clamp(0.0, 1.0);
    }

    pub fn global_alpha(&self) -> f32 {
        self.global_alpha
    }

    // ── 字形（圖示）渲染 ──

    pub fn paint_glyph(&mut self, rect: Rect, codepoint: u32, color: Color, font_size: f32) -> usize {
        let ch = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        // C++ Paint_icon 預設不傳遞 Clip_rect（僅當 ClipMode::bounds 時）
        self.paint_text_clipped(rect, s, font_size, color, TextAlign::Center, None)
    }

    // ── 變換負載 ──

    pub fn push_transform_3d(&mut self, transform: Transform3D) -> u32 {
        let idx = self.transform_payloads.len() as u32;
        self.transform_payloads.push(transform);
        idx
    }

    /// 推動旋轉變換。所有後續繪製命令都將具有此變換
    /// 直到呼叫“pop_transform()”。原點相對於每個指令的矩形。
    pub fn push_rotation(&mut self, angle_deg: f32, origin_x: f32, origin_y: f32) {
        let transform = Transform3D {
            rotation_z_deg: angle_deg,
            origin_x,
            origin_y,
            ..Default::default()
        };
        self.current_transform_index = self.push_transform_3d(transform);
    }

    pub fn pop_transform(&mut self) {
        self.current_transform_index = K_INVALID_PAYLOAD_INDEX;
    }

    /// 如果旋轉變換處於活動狀態，則推入一個具有相同角度的新旋轉變換，但
    /// 不同的起源。返回先前的變換索引以便稍後恢復。
    /// 如果沒有任何轉換處於活動狀態，則不執行任何操作並傳回 K_INVALID_PAYLOAD_INDEX。
    pub fn swap_rotation_origin(&mut self, new_ox: f32, new_oy: f32) -> u32 {
        let old_idx = self.current_transform_index;
        if old_idx != K_INVALID_PAYLOAD_INDEX {
            if let Some(t) = self.transform_payloads.get(old_idx as usize).copied() {
                if t.rotation_z_deg.abs() > 0.001 {
                    self.push_rotation(t.rotation_z_deg, new_ox, new_oy);
                }
            }
        }
        old_idx
    }

    /// 從 swap_rotation_origin 恢復先前儲存的變換索引。
    pub fn restore_transform(&mut self, idx: u32) {
        self.current_transform_index = idx;
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionResult {
    pub hover: f32,
    pub press: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionResultEx {
    pub hover: f32,
    pub press: f32,
    pub focus: f32,
    pub active: f32,
}
