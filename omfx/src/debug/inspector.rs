//! Entity inspector panel for detailed debugging
//!
//! Provides detailed property inspection and history tracking
//! for selected game entities.

use std::collections::HashMap;

/// Detailed entity property value
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Vec2(f32, f32),
    List(Vec<PropertyValue>),
}

/// Entity property for inspector
#[derive(Debug, Clone)]
pub struct EntityProperty {
    pub name: String,
    pub value: PropertyValue,
    pub category: String,
}

/// Entity inspector data
#[derive(Debug, Clone)]
pub struct InspectedEntity {
    pub entity_id: u32,
    pub entity_type: String,
    pub name: String,
    pub properties: Vec<EntityProperty>,
}

/// Entity inspector
pub struct EntityInspector {
    /// Currently inspected entity
    pub inspected: Option<InspectedEntity>,
    /// Property history for tracking changes
    property_history: HashMap<String, Vec<PropertyValue>>,
    /// Max history entries per property
    max_history: usize,
    /// Is the inspector panel visible
    pub visible: bool,
}

impl EntityInspector {
    pub fn new() -> Self {
        Self {
            inspected: None,
            property_history: HashMap::new(),
            max_history: 100,
            visible: false,
        }
    }

    /// Toggle inspector visibility
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// Set entity to inspect
    pub fn inspect(&mut self, entity: InspectedEntity) {
        // Record property history
        for prop in &entity.properties {
            let key = format!("{}:{}", entity.entity_id, prop.name);
            let history = self.property_history.entry(key).or_insert_with(Vec::new);
            history.push(prop.value.clone());
            if history.len() > self.max_history {
                history.remove(0);
            }
        }
        self.inspected = Some(entity);
        self.visible = true;
    }

    /// Clear inspection
    pub fn clear(&mut self) {
        self.inspected = None;
    }

    /// Get property history
    pub fn get_property_history(
        &self,
        entity_id: u32,
        property_name: &str,
    ) -> Option<&Vec<PropertyValue>> {
        let key = format!("{}:{}", entity_id, property_name);
        self.property_history.get(&key)
    }

    /// Clear all property history
    pub fn clear_history(&mut self) {
        self.property_history.clear();
    }

    /// Build inspector from entity data
    pub fn build_from_entity(
        entity_id: u32,
        entity_type: &str,
        name: &str,
        position: (f32, f32),
        health: (f32, f32),
        additional_props: Vec<(String, String, PropertyValue)>,
    ) -> InspectedEntity {
        let mut properties = vec![
            EntityProperty {
                name: "Position".to_string(),
                value: PropertyValue::Vec2(position.0, position.1),
                category: "Transform".to_string(),
            },
            EntityProperty {
                name: "Health".to_string(),
                value: PropertyValue::String(format!("{:.0}/{:.0}", health.0, health.1)),
                category: "Stats".to_string(),
            },
            EntityProperty {
                name: "Health %".to_string(),
                value: PropertyValue::Float((health.0 / health.1.max(1.0) * 100.0) as f64),
                category: "Stats".to_string(),
            },
        ];

        for (name, category, value) in additional_props {
            properties.push(EntityProperty {
                name,
                value,
                category,
            });
        }

        InspectedEntity {
            entity_id,
            entity_type: entity_type.to_string(),
            name: name.to_string(),
            properties,
        }
    }

    /// Format property value as string
    pub fn format_value(value: &PropertyValue) -> String {
        match value {
            PropertyValue::Int(v) => v.to_string(),
            PropertyValue::Float(v) => format!("{:.2}", v),
            PropertyValue::String(v) => v.clone(),
            PropertyValue::Bool(v) => v.to_string(),
            PropertyValue::Vec2(x, y) => format!("({:.1}, {:.1})", x, y),
            PropertyValue::List(items) => {
                let strs: Vec<String> = items.iter().map(Self::format_value).collect();
                format!("[{}]", strs.join(", "))
            }
        }
    }

    /// Get properties grouped by category
    pub fn get_properties_by_category(&self) -> HashMap<String, Vec<&EntityProperty>> {
        let mut categories: HashMap<String, Vec<&EntityProperty>> = HashMap::new();

        if let Some(ref entity) = self.inspected {
            for prop in &entity.properties {
                categories
                    .entry(prop.category.clone())
                    .or_insert_with(Vec::new)
                    .push(prop);
            }
        }

        categories
    }

    /// Generate inspector summary text
    pub fn summary_text(&self) -> String {
        if let Some(ref entity) = self.inspected {
            let mut lines = vec![
                format!("=== Entity Inspector ==="),
                format!("ID: {} | Type: {}", entity.entity_id, entity.entity_type),
                format!("Name: {}", entity.name),
                format!("---"),
            ];

            let categories = self.get_properties_by_category();
            let mut sorted_categories: Vec<_> = categories.keys().collect();
            sorted_categories.sort();

            for category in sorted_categories {
                lines.push(format!("[{}]", category));
                if let Some(props) = categories.get(category) {
                    for prop in props {
                        lines.push(format!("  {}: {}", prop.name, Self::format_value(&prop.value)));
                    }
                }
            }

            lines.join("\n")
        } else {
            "No entity selected".to_string()
        }
    }
}

impl Default for EntityInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value() {
        assert_eq!(EntityInspector::format_value(&PropertyValue::Int(42)), "42");
        assert_eq!(
            EntityInspector::format_value(&PropertyValue::Float(3.14159)),
            "3.14"
        );
        assert_eq!(
            EntityInspector::format_value(&PropertyValue::String("test".to_string())),
            "test"
        );
        assert_eq!(
            EntityInspector::format_value(&PropertyValue::Bool(true)),
            "true"
        );
        assert_eq!(
            EntityInspector::format_value(&PropertyValue::Vec2(10.0, 20.0)),
            "(10.0, 20.0)"
        );
    }

    #[test]
    fn test_build_from_entity() {
        let entity = EntityInspector::build_from_entity(
            1,
            "Player",
            "TestPlayer",
            (100.0, 200.0),
            (80.0, 100.0),
            vec![],
        );

        assert_eq!(entity.entity_id, 1);
        assert_eq!(entity.entity_type, "Player");
        assert_eq!(entity.name, "TestPlayer");
        assert_eq!(entity.properties.len(), 3); // Position, Health, Health %
    }

    #[test]
    fn test_property_history() {
        let mut inspector = EntityInspector::new();

        let entity1 = EntityInspector::build_from_entity(
            1,
            "Player",
            "Test",
            (0.0, 0.0),
            (100.0, 100.0),
            vec![],
        );
        inspector.inspect(entity1);

        let entity2 = EntityInspector::build_from_entity(
            1,
            "Player",
            "Test",
            (10.0, 10.0),
            (90.0, 100.0),
            vec![],
        );
        inspector.inspect(entity2);

        let history = inspector.get_property_history(1, "Position");
        assert!(history.is_some());
        assert_eq!(history.unwrap().len(), 2);
    }
}
