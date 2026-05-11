pub use crate::runtime::item::*;

pub fn load_registry_from_path(path: &str) -> Result<ItemRegistry, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let cleaned = remove_json_comments(&raw);
    let list: Vec<ItemConfig> = serde_json::from_str(&cleaned)?;
    let registry = ItemRegistry::from_configs(list);
    log::info!("loaded {} item configs", registry.items.len());
    Ok(registry)
}

fn remove_json_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            out.push(ch);
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}
