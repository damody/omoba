//! Entity list sidebar
//!
//! Displays a hierarchical list of all entities in the game:
//! - Grouped by entity type (Players, Minions, Summons, Projectiles)
//! - Collapsible groups
//! - Click to select and focus entity

use fyrox::{
    core::pool::Handle,
    gui::{
        UiNode,
        widget::WidgetBuilder,
        text::TextBuilder,
        list_view::{ListViewBuilder, ListViewMessage},
        scroll_viewer::ScrollViewerBuilder,
        border::BorderBuilder,
        stack_panel::StackPanelBuilder,
        Orientation,
        BuildContext,
        Thickness,
        brush::Brush,
    },
    core::color::Color,
};
use std::collections::HashMap;

/// Helper function to create margin with different values for each side
fn margin(left: f32, top: f32, right: f32, bottom: f32) -> Thickness {
    Thickness { left, top, right, bottom }
}

/// Entity list entry for display
#[derive(Debug, Clone)]
pub struct EntityListEntry {
    /// Unique entity ID
    pub id: u32,
    /// Display name
    pub name: String,
    /// Entity type for grouping (Player, Minion, Summon, Projectile, etc.)
    pub entity_type: String,
    /// Optional team/faction for coloring
    pub team: Option<String>,
}

impl EntityListEntry {
    /// Create a new entity list entry
    pub fn new(id: u32, name: String, entity_type: String) -> Self {
        Self {
            id,
            name,
            entity_type,
            team: None,
        }
    }

    /// Set team for this entry
    pub fn with_team(mut self, team: String) -> Self {
        self.team = Some(team);
        self
    }
}

/// Entity list sidebar panel
pub struct EntityListPanel {
    /// Root widget handle
    pub root: Handle<UiNode>,
    /// List view widget
    list_view: Handle<UiNode>,
    /// Header text widget
    header_text: Handle<UiNode>,
    /// Currently selected entity ID
    pub selected_entity_id: Option<u32>,
    /// Mapping from list index to entity ID
    index_to_entity: Vec<Option<u32>>,
    /// Collapsed state of groups
    collapsed_groups: HashMap<String, bool>,
}

impl EntityListPanel {
    /// Create a new entity list panel
    pub fn new(ctx: &mut BuildContext) -> Self {
        let header_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(5.0))
                .with_foreground(Brush::Solid(Color::from_rgba(200, 200, 200, 255)).into())
        )
        .with_text("Entities (0)")
        .build(ctx);

        let list_view = ListViewBuilder::new(
            WidgetBuilder::new()
        )
        .build(ctx);

        let scroll_viewer = ScrollViewerBuilder::new(
            WidgetBuilder::new()
                .with_child(list_view)
        )
        .build(ctx);

        let content = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_child(header_text)
                .with_child(scroll_viewer)
        )
        .with_orientation(Orientation::Vertical)
        .build(ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_background(Brush::Solid(Color::from_rgba(35, 35, 35, 230)).into())
                .on_column(1)
                .on_row(1)
                .with_child(content)
        )
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(ctx);

        Self {
            root,
            list_view,
            header_text,
            selected_entity_id: None,
            index_to_entity: Vec::new(),
            collapsed_groups: HashMap::new(),
        }
    }

    /// Update the entity list with new data
    pub fn update_entities(&mut self, entities: &[EntityListEntry], ctx: &mut BuildContext, ui: &mut fyrox::gui::UserInterface) {
        use fyrox::gui::text::TextMessage;
        use fyrox::gui::message::MessageDirection;

        // Group entities by type
        let mut grouped: HashMap<String, Vec<&EntityListEntry>> = HashMap::new();
        for entity in entities {
            grouped.entry(entity.entity_type.clone()).or_default().push(entity);
        }

        // Sort groups by type name
        let mut group_names: Vec<_> = grouped.keys().cloned().collect();
        group_names.sort();

        // Build list items and index mapping
        let mut items = Vec::new();
        self.index_to_entity.clear();

        for entity_type in &group_names {
            let group = &grouped[entity_type];
            let is_collapsed = *self.collapsed_groups.get(entity_type).unwrap_or(&false);

            // Group header with expand/collapse indicator
            let indicator = if is_collapsed { ">" } else { "v" };
            let header = TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(margin(3.0, 5.0, 3.0, 2.0))
                    .with_foreground(Brush::Solid(Color::from_rgba(180, 180, 100, 255)).into())
            )
            .with_text(format!("{} {} ({})", indicator, entity_type, group.len()))
            .build(ctx);
            items.push(header);
            self.index_to_entity.push(None); // Header item has no entity

            // Entity entries (if not collapsed)
            if !is_collapsed {
                for entity in group {
                    let item = TextBuilder::new(
                        WidgetBuilder::new()
                            .with_margin(margin(15.0, 2.0, 3.0, 2.0))
                            .with_foreground(Brush::Solid(Color::from_rgba(200, 200, 200, 255)).into())
                    )
                    .with_text(entity.name.clone())
                    .build(ctx);
                    items.push(item);
                    self.index_to_entity.push(Some(entity.id));
                }
            }
        }

        // Update header with total count
        let total = entities.len();
        ui.send_message(TextMessage::text(
            self.header_text,
            MessageDirection::ToWidget,
            format!("Entities ({})", total),
        ));

        // Update list view items
        ui.send_message(ListViewMessage::items(
            self.list_view,
            MessageDirection::ToWidget,
            items,
        ));
    }

    /// Toggle collapse state of a group
    pub fn toggle_group(&mut self, group_name: &str) {
        let collapsed = self.collapsed_groups.entry(group_name.to_string()).or_insert(false);
        *collapsed = !*collapsed;
    }

    /// Get entity ID from list selection index
    pub fn get_entity_at_index(&self, index: usize) -> Option<u32> {
        self.index_to_entity.get(index).copied().flatten()
    }

    /// Check if an index is a group header (not an entity)
    pub fn is_group_header(&self, index: usize) -> bool {
        self.index_to_entity.get(index).map_or(false, |opt| opt.is_none())
    }

    /// Select an entity by ID
    pub fn select_entity(&mut self, entity_id: Option<u32>) {
        self.selected_entity_id = entity_id;
    }

    /// Get the currently selected entity ID
    pub fn selected_entity(&self) -> Option<u32> {
        self.selected_entity_id
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selected_entity_id = None;
    }

    /// Get sorted entity type order for display
    pub fn entity_type_order() -> Vec<&'static str> {
        vec!["Player", "Summon", "Minion", "Projectile", "Structure", "Other"]
    }
}
