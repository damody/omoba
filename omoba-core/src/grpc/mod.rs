pub mod client;

pub mod game_proto {
    tonic::include_proto!("game");
}

pub use client::GrpcClient;
