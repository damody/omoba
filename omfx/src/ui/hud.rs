//! HUD panel for selected entity information
//!
//! Displays detailed information about the currently selected entity including:
//! - Entity name and type
//! - Health and mana bars
//! - Position coordinates
//! - Current status/state

use fyrox::{
    core::pool::Handle,
    gui::{
        UiNode,
        widget::WidgetBuilder,
        text::TextBuilder,
        border::BorderBuilder,
        grid::{GridBuilder, Column, Row},
        BuildContext,
        Thickness,
        brush::Brush,
    },
    core::color::Color,
};

/// Selected entity information for HUD display
#[derive(Debug, Clone, Default)]
pub struct SelectedEntityInfo {
    /// Entity display name
    pub name: String,
    /// Entity type (Player, Minion, Summon, Projectile, etc.)
    pub entity_type: String,
    /// Current and max health (current, max)
    pub health: (f32, f32),
    /// Optional mana (current, max), None for entities without mana
    pub mana: Option<(f32, f32)>,
    /// World position (x, y)
    pub position: (f32, f32),
    /// Current status description
    pub status: String,
}

impl SelectedEntityInfo {
    /// Create a new SelectedEntityInfo with basic values
    pub fn new(name: String, entity_type: String) -> Self {
        Self {
            name,
            entity_type,
            health: (100.0, 100.0),
            mana: None,
            position: (0.0, 0.0),
            status: "Idle".to_string(),
        }
    }

    /// Set health values
    pub fn with_health(mut self, current: f32, max: f32) -> Self {
        self.health = (current, max);
        self
    }

    /// Set mana values
    pub fn with_mana(mut self, current: f32, max: f32) -> Self {
        self.mana = Some((current, max));
        self
    }

    /// Set position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Set status
    pub fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }
}

/// HUD panel showing selected entity info
pub struct HudPanel {
    /// Root widget handle
    pub root: Handle<UiNode>,
    /// Name text widget
    name_text: Handle<UiNode>,
    /// Health text widget
    health_text: Handle<UiNode>,
    /// Mana text widget
    mana_text: Handle<UiNode>,
    /// Position text widget
    position_text: Handle<UiNode>,
    /// Status text widget
    status_text: Handle<UiNode>,
}

impl HudPanel {
    /// Create a new HUD panel
    pub fn new(ctx: &mut BuildContext) -> Self {
        let name_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .on_column(0)
                .on_row(0)
        )
        .with_text("No entity selected")
        .build(ctx);

        let health_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .on_column(1)
                .on_row(0)
        )
        .with_text("HP: --/--")
        .build(ctx);

        let mana_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .on_column(2)
                .on_row(0)
        )
        .with_text("MP: --/--")
        .build(ctx);

        let position_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .on_column(3)
                .on_row(0)
        )
        .with_text("Pos: (---, ---)")
        .build(ctx);

        let status_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .on_column(4)
                .on_row(0)
        )
        .with_text("Status: ---")
        .build(ctx);

        let content = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(name_text)
                .with_child(health_text)
                .with_child(mana_text)
                .with_child(position_text)
                .with_child(status_text)
        )
        .add_column(Column::stretch())  // Name
        .add_column(Column::stretch())  // HP
        .add_column(Column::stretch())  // MP
        .add_column(Column::stretch())  // Position
        .add_column(Column::stretch())  // Status
        .add_row(Row::stretch())
        .build(ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_background(Brush::Solid(Color::from_rgba(30, 30, 30, 220)).into())
                .on_column(0)
                .on_row(2)
                .with_child(content)
        )
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(ctx);

        Self {
            root,
            name_text,
            health_text,
            mana_text,
            position_text,
            status_text,
        }
    }

    /// Update the HUD panel with entity information
    pub fn update(&mut self, info: Option<&SelectedEntityInfo>, ui: &mut fyrox::gui::UserInterface) {
        use fyrox::gui::text::TextMessage;
        use fyrox::gui::message::MessageDirection;

        if let Some(entity) = info {
            // Update name
            ui.send_message(TextMessage::text(
                self.name_text,
                MessageDirection::ToWidget,
                format!("{} ({})", entity.name, entity.entity_type),
            ));

            // Update health
            let hp_pct = if entity.health.1 > 0.0 {
                (entity.health.0 / entity.health.1 * 100.0) as u32
            } else {
                0
            };
            ui.send_message(TextMessage::text(
                self.health_text,
                MessageDirection::ToWidget,
                format!("HP: {:.0}/{:.0} ({}%)", entity.health.0, entity.health.1, hp_pct),
            ));

            // Update mana
            if let Some((mp_cur, mp_max)) = entity.mana {
                let mp_pct = if mp_max > 0.0 {
                    (mp_cur / mp_max * 100.0) as u32
                } else {
                    0
                };
                ui.send_message(TextMessage::text(
                    self.mana_text,
                    MessageDirection::ToWidget,
                    format!("MP: {:.0}/{:.0} ({}%)", mp_cur, mp_max, mp_pct),
                ));
            } else {
                ui.send_message(TextMessage::text(
                    self.mana_text,
                    MessageDirection::ToWidget,
                    "MP: N/A".to_string(),
                ));
            }

            // Update position
            ui.send_message(TextMessage::text(
                self.position_text,
                MessageDirection::ToWidget,
                format!("Pos: ({:.1}, {:.1})", entity.position.0, entity.position.1),
            ));

            // Update status
            ui.send_message(TextMessage::text(
                self.status_text,
                MessageDirection::ToWidget,
                format!("Status: {}", entity.status),
            ));
        } else {
            // Clear all fields when no entity selected
            ui.send_message(TextMessage::text(
                self.name_text,
                MessageDirection::ToWidget,
                "No entity selected".to_string(),
            ));
            ui.send_message(TextMessage::text(
                self.health_text,
                MessageDirection::ToWidget,
                "HP: --/--".to_string(),
            ));
            ui.send_message(TextMessage::text(
                self.mana_text,
                MessageDirection::ToWidget,
                "MP: --/--".to_string(),
            ));
            ui.send_message(TextMessage::text(
                self.position_text,
                MessageDirection::ToWidget,
                "Pos: (---, ---)".to_string(),
            ));
            ui.send_message(TextMessage::text(
                self.status_text,
                MessageDirection::ToWidget,
                "Status: ---".to_string(),
            ));
        }
    }

    /// Clear the HUD panel (deselect entity)
    pub fn clear(&mut self, ui: &mut fyrox::gui::UserInterface) {
        self.update(None, ui);
    }
}
