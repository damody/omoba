//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::info;

mod camera;
mod cli;
mod config;
mod debug;
mod game;
mod renderer;
mod ui;

use cli::CliArgs;
use config::OmfxConfig;

fn main() {
    // Parse command-line arguments first (before logging init for --help)
    let args = CliArgs::parse();

    // Setup logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Handle --generate-config
    if args.generate_config {
        let config = OmfxConfig::default();
        if let Err(e) = config.save_to_file("omfx_config.toml") {
            eprintln!("Failed to generate config: {}", e);
            std::process::exit(1);
        }
        println!("Generated default config: omfx_config.toml");
        return;
    }

    info!("Starting OMFX - OMOBA Fyrox Debug Frontend");

    // Log CLI overrides if any
    if let Some(ref config_path) = args.config {
        info!("Using config file: {:?}", config_path);
    }
    if let Some(ref host) = args.mqtt_host {
        info!("MQTT host override: {}", host);
    }
    if let Some(port) = args.mqtt_port {
        info!("MQTT port override: {}", port);
    }
    if let Some(ref player) = args.player_name {
        info!("Player name override: {}", player);
    }
    if args.start_paused {
        info!("Starting in paused state");
    }

    // Create and run the game
    let mut executor = Executor::new();
    executor.add_plugin(game::Game::new());
    executor.run()
}
