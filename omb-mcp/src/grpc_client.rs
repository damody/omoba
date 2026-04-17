use anyhow::Result;
use tonic::transport::Channel;

pub mod game_proto {
    tonic::include_proto!("game");
}

use game_proto::game_service_client::GameServiceClient;
use game_proto::{GameStateRequest, GameStateResponse};

pub struct GameClient {
    client: GameServiceClient<Channel>,
}

impl GameClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let client = GameServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }

    pub async fn query_state(&mut self, query_type: &str, player_name: &str) -> Result<GameStateResponse> {
        let request = tonic::Request::new(GameStateRequest {
            query_type: query_type.to_string(),
            player_name: player_name.to_string(),
        });
        let response = self.client.query_game_state(request).await?;
        Ok(response.into_inner())
    }
}
