//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::{info, warn};

mod backend;
mod camera;
mod cli;
mod config;
mod debug;
mod game;
mod renderer;
mod ui;

use backend::BackendManager;
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

    // Load configuration
    let config = if let Some(ref config_path) = args.config {
        info!("Using config file: {:?}", config_path);
        OmfxConfig::load_from(config_path.to_string_lossy().as_ref())
    } else {
        OmfxConfig::load()
    };

    // Log CLI overrides if any
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

    // Start backend if auto_start is enabled
    let mut backend_manager = BackendManager::new(config.backend.clone());
    if config.backend.auto_start {
        info!("Auto-starting backend after {}ms delay...", config.backend.start_delay_ms);
        std::thread::sleep(std::time::Duration::from_millis(config.backend.start_delay_ms));

        if let Err(e) = backend_manager.start() {
            warn!("Failed to auto-start backend: {}", e);
            warn!("You may need to start the backend manually");
        }
    }

    // Create and run the game
    let mut executor = Executor::new();
    executor.add_plugin(game::Game::new());
    executor.run();

    // Backend will be automatically stopped when backend_manager is dropped
    info!("OMFX shutting down...");
}
