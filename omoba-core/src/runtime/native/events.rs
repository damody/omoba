use serde_json::Value;

/// Transport-neutral broadcast target used by deterministic runtime code.
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeBroadcast {
    All,
    AoiPoint(f32, f32),
    AoiEntity(u64),
    PlayerOnly(String),
}

/// Runtime-owned event emitted by deterministic gameplay code.
///
/// Backend adapters decide how to encode these events into concrete transports.
/// Local replicas may ignore them or project selected events into render data.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEvent {
    pub topic: String,
    pub kind: String,
    pub action: String,
    pub data: Value,
    pub entity_pos: Option<(f32, f32)>,
    pub broadcast: Option<RuntimeBroadcast>,
}

impl RuntimeEvent {
    pub fn new(
        topic: impl Into<String>,
        kind: impl Into<String>,
        action: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            topic: topic.into(),
            kind: kind.into(),
            action: action.into(),
            data,
            entity_pos: None,
            broadcast: None,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.entity_pos = Some((x, y));
        self.broadcast = Some(RuntimeBroadcast::AoiPoint(x, y));
        self
    }

    pub fn with_broadcast(mut self, broadcast: RuntimeBroadcast) -> Self {
        self.broadcast = Some(broadcast);
        self
    }
}

/// Sink abstraction that keeps deterministic runtime independent of backend transport types.
pub trait RuntimeEventSink {
    fn emit(&mut self, event: RuntimeEvent);
}

pub type RuntimeEvents = Vec<RuntimeEvent>;

#[derive(Default)]
pub struct RuntimeEventVecSink {
    pub events: RuntimeEvents,
}

impl RuntimeEventSink for RuntimeEventVecSink {
    fn emit(&mut self, event: RuntimeEvent) {
        self.events.push(event);
    }
}

impl RuntimeEventSink for RuntimeEvents {
    fn emit(&mut self, event: RuntimeEvent) {
        self.push(event);
    }
}
