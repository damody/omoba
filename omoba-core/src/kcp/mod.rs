pub mod framing;
pub mod client;

pub mod game_proto {
    include!(concat!(env!("OUT_DIR"), "/game.rs"));
}

pub use client::KcpClient;
pub use client::GameEventData;
pub use client::LockstepInbound;
