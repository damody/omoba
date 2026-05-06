# ![允許(clippy::missing_safety_doc)]
# ![允許(clippy::too_many_arguments)]
# ![允許(clippy::new_without_default)]
# ![允許(clippy::type_complexity)]

pub mod color;
pub mod rect;
pub mod math;
pub mod graphics;
pub mod core;
pub mod animation;
pub mod text;
pub mod runtime;
pub mod renderer;
pub mod quick;
pub mod platform;
pub mod app;

// 重新匯出發光，以便下游板條箱使用相同的版本
pub use glow;

// 為方便再出口
pub use color::{rgba, rgb, mix, Color};
pub use rect::{Rect, SplitRects};
pub use core::foundation::{FlexLength, FlexAlign, Theme, ThemeMode, ButtonStyle, InputState, px, fr, fit, make_theme};
pub use core::draw_command::{TextAlign, CommandType, DrawCommand};
pub use core::context::Context;
pub use graphics::effects::{Brush, Stroke, Shadow, Blur, GfxColor, BrushKind, Point, ColorStop};
pub use graphics::transforms::{Transform2D, Transform3D};
pub use graphics::primitives::{CornerRadius, ClipRect, ImageFit, RectanglePrimitive, ImagePrimitive, IconPrimitive};
pub use animation::easing::{CubicBezier, EasingPreset, ease, ease_bezier, sample_bezier_y};
pub use animation::timeline::{TimelineClip, ScalarTrack, PropertyKind};
pub use animation::animator::{lerp_scalar, animate_scalar, interpolate_transform_2d, interpolate_transform_3d};
pub use quick::ui::UI;
pub use quick::gfx;
pub use quick::builders;
pub use quick::anchor;
pub use app::options::AppOptions;
pub use app::run::{run, run_with_options};
pub use runtime::contracts::{WindowMetrics, FrameClock};
pub use renderer::contracts::{ClearState, DrawDataView, RendererBackend};
