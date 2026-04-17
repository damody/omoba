mod grpc_client;
mod toon_format;

use anyhow::Result;
use rmcp::{ServiceExt, tool, tool_router};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;
use std::sync::Arc;

use grpc_client::GameClient;
use toon_format::json_to_toon;

#[derive(Clone)]
struct OmobaMcp {
    client: Arc<Mutex<GameClient>>,
}

#[derive(Deserialize, JsonSchema)]
struct InspectPlayerParams {
    /// 玩家名稱
    player_name: String,
}

#[tool_router(server_handler)]
impl OmobaMcp {
    #[tool(description = "列出當前所有玩家及基本資訊（英雄名、等級、HP、位置）")]
    async fn list_players(&self) -> String {
        let mut client = self.client.lock().await;
        let resp = match client.query_state("list_players", "").await {
            Ok(r) => r,
            Err(e) => return format!("Error: gRPC call failed: {}", e),
        };

        if !resp.success {
            return format!("Error: {}", resp.error);
        }

        match serde_json::from_slice::<serde_json::Value>(&resp.data_json) {
            Ok(json) => json_to_toon(&json),
            Err(e) => format!("Error: JSON parse failed: {}", e),
        }
    }

    #[tool(description = "查看指定玩家視角中所有單位的狀態（位置、HP、攻擊目標、移動目標、技能冷卻）")]
    async fn inspect_player_view(&self, Parameters(params): Parameters<InspectPlayerParams>) -> String {
        let mut client = self.client.lock().await;
        let resp = match client.query_state("inspect_player_view", &params.player_name).await {
            Ok(r) => r,
            Err(e) => return format!("Error: gRPC call failed: {}", e),
        };

        if !resp.success {
            return format!("Error: {}", resp.error);
        }

        match serde_json::from_slice::<serde_json::Value>(&resp.data_json) {
            Ok(json) => json_to_toon(&json),
            Err(e) => format!("Error: JSON parse failed: {}", e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("omb_mcp=info".parse()?)
        )
        .with_writer(std::io::stderr)
        .init();

    let grpc_addr = std::env::var("OMB_GRPC_ADDR")
        .unwrap_or_else(|_| "http://localhost:50061".to_string());

    tracing::info!("Connecting to omb backend at {}", grpc_addr);

    let client = GameClient::connect(&grpc_addr).await?;

    let server = OmobaMcp {
        client: Arc::new(Mutex::new(client)),
    };

    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
