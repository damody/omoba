use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OutboundMsg {
    pub topic: String,
    pub msg: String,
    #[serde(skip)]
    pub entity_pos: Option<(f32, f32)>,
}

impl OutboundMsg {
    pub fn new_s(topic: &str, t: &str, a: &str, v: serde_json::Value) -> Self {
        Self {
            topic: topic.to_string(),
            msg: json!({ "t": t, "a": a, "d": v }).to_string(),
            entity_pos: None,
        }
    }

    pub fn new_s_all(topic: &str, t: &str, a: &str, v: serde_json::Value) -> Self {
        Self::new_s(topic, t, a, v)
    }

    pub fn new_s_at(
        topic: &str,
        t: &str,
        a: &str,
        v: serde_json::Value,
        x: f32,
        y: f32,
    ) -> Self {
        let mut msg = Self::new_s(topic, t, a, v);
        msg.entity_pos = Some((x, y));
        msg
    }
}
