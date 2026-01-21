//! UI module for OMFX
//!
//! Provides the user interface components for the debug visualization tool:
//! - HUD panel for selected entity information
//! - Time control panel for playback speed
//! - Entity list sidebar for browsing entities
//! - Minimap for overview navigation

pub mod hud;
pub mod time_control;
pub mod entity_list;
pub mod minimap;

pub use hud::{HudPanel, SelectedEntityInfo};
pub use time_control::TimeControlPanel;
pub use entity_list::{EntityListPanel, EntityListEntry};
pub use minimap::{MinimapPanel, MinimapMarker};

use fyrox::{
    core::pool::Handle,
    gui::{
        UiNode,
        widget::WidgetBuilder,
        grid::{GridBuilder, Column, Row},
        BuildContext,
    },
};

/// Main UI manager that coordinates all UI panels
pub struct UiManager {
    pub hud: HudPanel,
    pub time_control: TimeControlPanel,
    pub entity_list: EntityListPanel,
    pub minimap: MinimapPanel,
    pub root: Handle<UiNode>,
}

impl UiManager {
    /// Create a new UI manager with all panels
    pub fn new(ctx: &mut BuildContext) -> Self {
        let hud = HudPanel::new(ctx);
        let time_control = TimeControlPanel::new(ctx);
        let entity_list = EntityListPanel::new(ctx);
        let minimap = MinimapPanel::new(ctx);

        // Create main layout grid
        // Layout:
        // Row 0: Time control (spans both columns)
        // Row 1: Main game view | Entity list
        // Row 2: HUD panel | Minimap
        let root = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(time_control.root)  // Row 0, Col 0-1
                .with_child(entity_list.root)   // Row 1, Col 1
                .with_child(hud.root)           // Row 2, Col 0
                .with_child(minimap.root)       // Row 2, Col 1
        )
        .add_row(Row::strict(40.0))    // Top toolbar
        .add_row(Row::stretch())        // Main area
        .add_row(Row::strict(80.0))    // Bottom panels
        .add_column(Column::stretch())  // Main view
        .add_column(Column::strict(200.0)) // Right sidebar
        .build(ctx);

        Self {
            hud,
            time_control,
            entity_list,
            minimap,
            root,
        }
    }

    /// Update the HUD with selected entity info
    pub fn update_hud(&mut self, info: Option<&SelectedEntityInfo>, ui: &mut fyrox::gui::UserInterface) {
        self.hud.update(info, ui);
    }

    /// Update time display
    pub fn update_time(&mut self, game_time: f32, time_scale: f32, is_paused: bool, ui: &mut fyrox::gui::UserInterface) {
        self.time_control.update_time(game_time, time_scale, is_paused, ui);
    }

    /// Update entity list
    pub fn update_entity_list(&mut self, entities: &[EntityListEntry], ctx: &mut BuildContext, ui: &mut fyrox::gui::UserInterface) {
        self.entity_list.update_entities(entities, ctx, ui);
    }

    /// Update minimap markers
    pub fn update_minimap(&mut self, markers: Vec<MinimapMarker>) {
        self.minimap.update_markers(markers);
    }
}
