//! Time control panel
//!
//! Provides playback controls for the debug visualization:
//! - Pause/Resume button
//! - Speed adjustment buttons (0.5x, 1x, 2x, 4x)
//! - Current speed display
//! - Game time display

use fyrox::{
    core::pool::Handle,
    gui::{
        UiNode,
        widget::WidgetBuilder,
        text::TextBuilder,
        button::ButtonBuilder,
        stack_panel::StackPanelBuilder,
        border::BorderBuilder,
        Orientation,
        BuildContext,
        Thickness,
        VerticalAlignment,
        brush::Brush,
    },
    core::color::Color,
};

/// Helper function to create margin with different values for each side
fn margin(left: f32, top: f32, right: f32, bottom: f32) -> Thickness {
    Thickness { left, top, right, bottom }
}

/// Time control panel widget handles
pub struct TimeControlPanel {
    /// Root widget handle
    pub root: Handle<UiNode>,
    /// Pause/Resume button
    pub pause_button: Handle<UiNode>,
    /// 0.5x speed button
    pub speed_05x_button: Handle<UiNode>,
    /// 1x speed button
    pub speed_1x_button: Handle<UiNode>,
    /// 2x speed button
    pub speed_2x_button: Handle<UiNode>,
    /// 4x speed button
    pub speed_4x_button: Handle<UiNode>,
    /// Speed display text
    speed_text: Handle<UiNode>,
    /// Game time display text
    time_text: Handle<UiNode>,
    /// Current playback state
    is_paused: bool,
    /// Current time scale
    time_scale: f32,
}

impl TimeControlPanel {
    /// Create a new time control panel
    pub fn new(ctx: &mut BuildContext) -> Self {
        // Pause button
        let pause_button = ButtonBuilder::new(
            WidgetBuilder::new()
                .with_width(40.0)
                .with_height(30.0)
                .with_margin(Thickness::uniform(2.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("||")
        .build(ctx);

        // Speed buttons
        let speed_05x_button = ButtonBuilder::new(
            WidgetBuilder::new()
                .with_width(50.0)
                .with_height(30.0)
                .with_margin(Thickness::uniform(2.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("0.5x")
        .build(ctx);

        let speed_1x_button = ButtonBuilder::new(
            WidgetBuilder::new()
                .with_width(40.0)
                .with_height(30.0)
                .with_margin(Thickness::uniform(2.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("1x")
        .build(ctx);

        let speed_2x_button = ButtonBuilder::new(
            WidgetBuilder::new()
                .with_width(40.0)
                .with_height(30.0)
                .with_margin(Thickness::uniform(2.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("2x")
        .build(ctx);

        let speed_4x_button = ButtonBuilder::new(
            WidgetBuilder::new()
                .with_width(40.0)
                .with_height(30.0)
                .with_margin(Thickness::uniform(2.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("4x")
        .build(ctx);

        // Speed display
        let speed_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(margin(10.0, 5.0, 10.0, 5.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("Speed: 1.0x")
        .build(ctx);

        // Time display
        let time_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(margin(10.0, 5.0, 10.0, 5.0))
                .with_vertical_alignment(VerticalAlignment::Center)
        )
        .with_text("Time: 00:00:00")
        .build(ctx);

        // Layout all controls horizontally
        let content = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_child(pause_button)
                .with_child(speed_05x_button)
                .with_child(speed_1x_button)
                .with_child(speed_2x_button)
                .with_child(speed_4x_button)
                .with_child(speed_text)
                .with_child(time_text)
        )
        .with_orientation(Orientation::Horizontal)
        .build(ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_background(Brush::Solid(Color::from_rgba(40, 40, 40, 240)).into())
                .on_column(0)
                .on_row(0)
                .with_child(content)
        )
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(ctx);

        Self {
            root,
            pause_button,
            speed_05x_button,
            speed_1x_button,
            speed_2x_button,
            speed_4x_button,
            speed_text,
            time_text,
            is_paused: false,
            time_scale: 1.0,
        }
    }

    /// Update time and speed display
    pub fn update_time(&mut self, game_time: f32, time_scale: f32, is_paused: bool, ui: &mut fyrox::gui::UserInterface) {
        use fyrox::gui::text::TextMessage;
        use fyrox::gui::message::MessageDirection;

        self.is_paused = is_paused;
        self.time_scale = time_scale;

        // Format time as HH:MM:SS
        let total_seconds = game_time as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        ui.send_message(TextMessage::text(
            self.time_text,
            MessageDirection::ToWidget,
            format!("Time: {:02}:{:02}:{:02}", hours, minutes, seconds),
        ));

        // Update speed display
        let speed_str = if is_paused {
            "Speed: PAUSED".to_string()
        } else {
            format!("Speed: {:.1}x", time_scale)
        };

        ui.send_message(TextMessage::text(
            self.speed_text,
            MessageDirection::ToWidget,
            speed_str,
        ));
    }

    /// Update pause button text based on current state
    #[allow(unused_variables)]
    pub fn update_pause_button(&self, is_paused: bool, ui: &mut fyrox::gui::UserInterface) {
        // Note: Button text update requires finding the text widget inside the button
        // For now, we use simple symbols that indicate state
        let _text = if is_paused { ">" } else { "||" };
        // Button text updates in Fyrox require more complex widget traversal
        // This is left as a TODO for full implementation
    }

    /// Check if playback is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Get current time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Format duration as HH:MM:SS string
    pub fn format_time(seconds: f32) -> String {
        let total_seconds = seconds as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Format duration with milliseconds as HH:MM:SS.mmm
    pub fn format_time_ms(seconds: f32) -> String {
        let total_seconds = seconds as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;
        let ms = ((seconds - seconds.floor()) * 1000.0) as u32;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, ms)
    }
}
