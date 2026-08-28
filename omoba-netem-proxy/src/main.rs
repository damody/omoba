use omoba_netem_proxy::{config::ProxyConfig, runtime, TOOL_VERSION};

#[tokio::main]
async fn main() {
    let config = match ProxyConfig::from_env_args() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("netem-proxy config error: {error}");
            std::process::exit(2)
        }
    };
    eprintln!(
        "netem-proxy starting version={} mode={:?} team1={} team2={} server={} seed={}",
        TOOL_VERSION,
        config.mode,
        config.routes[0].client_bind,
        config.routes[1].client_bind,
        config.server_addr,
        config.seed
    );
    if let Err(error) = runtime::run(config).await {
        eprintln!("netem-proxy failed: {error}");
        std::process::exit(1)
    }
}
