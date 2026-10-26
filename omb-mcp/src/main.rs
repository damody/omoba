mod kcp_client;
mod run_status;
mod toon_format;

use anyhow::Result;
use rmcp::{ServiceExt, tool, tool_router};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

use kcp_client::GameClient;
use toon_format::json_to_toon;

/// Query-only status tools. Server queries need a server started with
/// OMB_DEBUG_QUERY=1 (local dev topologies set it); run status reads files only.
#[derive(Clone)]
struct OmobaMcp {
    kcp_addr: String,
    repo: PathBuf,
}

#[derive(Deserialize, JsonSchema)]
struct InspectPlayerParams {
    /// 玩家名稱
    player_name: String,
}

#[derive(Deserialize, JsonSchema)]
struct RunStatusParams {
    /// Run directory (for example target/split-runs/<id>). Empty = newest local run.
    #[serde(default)]
    run_dir: String,
}

/// One fresh query per call: the MCP keeps working when the server restarts.
async fn query(addr: &str, query_type: &str, player_name: &str) -> String {
    let mut client = match GameClient::connect(addr).await {
        Ok(client) => client,
        Err(error) => return format!("Error: cannot reach server at {addr}: {error}"),
    };
    let response = match tokio::time::timeout(std::time::Duration::from_secs(5), client.query_state(query_type, player_name)).await {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => return format!("Error: KCP call failed: {error}"),
        Err(_) => return format!("Error: no reply from {addr} within 5s (is OMB_DEBUG_QUERY=1 set on the server?)"),
    };
    if !response.success {
        return format!("Error: {}", response.error);
    }
    match serde_json::from_slice::<serde_json::Value>(&response.data_json) {
        Ok(json) => json_to_toon(&json),
        Err(error) => format!("Error: JSON parse failed: {error}"),
    }
}

fn run_status_text(repo: &std::path::Path, run_dir: &str) -> String {
    let dir = if run_dir.is_empty() {
        match run_status::latest_run(repo) {
            Some(dir) => dir,
            None => return "Error: no run directory with children.json under target/split-runs or target/mvp-a3".into(),
        }
    } else {
        let path = PathBuf::from(run_dir);
        if path.is_absolute() { path } else { repo.join(path) }
    };
    json_to_toon(&run_status::run_status(&dir))
}

#[tool_router(server_handler)]
impl OmobaMcp {
    #[tool(description = "整局權威狀態（debug、未經視野過濾）：階段、勝負、經過時間、各隊剩餘塔、基地、每位英雄 KDA／血量／位置／重生")]
    async fn match_status(&self) -> String {
        query(&self.kcp_addr, "match_status", "").await
    }

    #[tool(description = "本機 run 目錄狀態（不需 server）：每個 client 的 autopilot 進度、各 intent 的 ACK／未送出統計、最近問題、是否結束、最新截圖")]
    async fn run_status(&self, Parameters(params): Parameters<RunStatusParams>) -> String {
        run_status_text(&self.repo, &params.run_dir)
    }

    #[tool(description = "列出當前所有玩家及基本資訊（英雄名、等級、HP、位置）")]
    async fn list_players(&self) -> String {
        query(&self.kcp_addr, "list_players", "").await
    }

    #[tool(description = "查看指定玩家視角中所有單位的狀態（位置、HP、攻擊目標、移動目標、技能冷卻）")]
    async fn inspect_player_view(&self, Parameters(params): Parameters<InspectPlayerParams>) -> String {
        query(&self.kcp_addr, "inspect_player_view", &params.player_name).await
    }
}

fn repo_root() -> PathBuf {
    // omb-mcp lives at <repo>/omb-mcp; OMOBA_ROOT overrides for other layouts.
    std::env::var_os("OMOBA_ROOT").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
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

    let kcp_addr = std::env::var("OMB_KCP_ADDR").unwrap_or_else(|_| "127.0.0.1:57061".to_string());
    let repo = repo_root();

    // CLI: omb-mcp --once match_status | run_status [DIR] | list_players
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--once") {
        let output = match args.get(1).map(String::as_str) {
            Some("match_status") => query(&kcp_addr, "match_status", "").await,
            Some("list_players") => query(&kcp_addr, "list_players", "").await,
            Some("run_status") => run_status_text(&repo, args.get(2).map(String::as_str).unwrap_or("")),
            _ => "Usage: omb-mcp --once match_status | list_players | run_status [RUN_DIR]".into(),
        };
        println!("{output}");
        return Ok(());
    }

    tracing::info!("omb-mcp ready; server queries go to {}", kcp_addr);
    let server = OmobaMcp { kcp_addr, repo };
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
