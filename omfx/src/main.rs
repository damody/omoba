//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::{info, warn, error};
use std::time::Duration;
use tokio::time::timeout;

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
use omoba_core::{MqttClient, MqttEvent};

/// Perform health check by connecting to MQTT and waiting for a message
async fn health_check(config: &OmfxConfig) -> Result<(), String> {
    let server_config = config.to_core_server_config();

    let mut mqtt_client = MqttClient::new(
        &server_config,
        &config.frontend.player_name,
        "omfx_health_check",
    ).map_err(|e| format!("Failed to create MQTT client: {}", e))?;

    info!("Health check: Connecting to MQTT broker {}:{}...",
          config.server.mqtt_host, config.server.mqtt_port);

    let timeout_duration = Duration::from_millis(config.backend.health_check_timeout_ms);

    // Wait for MQTT connection or message
    let result = timeout(timeout_duration, async {
        loop {
            if let Some(event) = mqtt_client.poll().await {
                match event {
                    MqttEvent::Connected => {
                        info!("Health check: MQTT connected successfully");
                        // Subscribe to topics to receive messages
                        if let Err(e) = mqtt_client.subscribe_to_game_topics().await {
                            warn!("Health check: Failed to subscribe: {}", e);
                        }
                        // Continue waiting for actual message from backend
                    }
                    MqttEvent::Message { topic, .. } => {
                        info!("Health check: Received message on topic '{}' - Backend is alive!", topic);
                        return Ok(());
                    }
                    MqttEvent::Error(e) => {
                        return Err(format!("MQTT error: {}", e));
                    }
                    MqttEvent::Disconnected => {
                        return Err("MQTT disconnected".to_string());
                    }
                }
            }
        }
    }).await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!(
            "Health check timeout: No MQTT message received within {}ms. Backend may not be running.",
            config.backend.health_check_timeout_ms
        )),
    }
}

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
            error!("Failed to start backend: {}", e);
            if config.backend.require_health_check {
                error!("Backend is required. Exiting.");
                std::process::exit(1);
            }
            warn!("Continuing without backend (require_health_check is disabled)");
        }
    }

    // Perform health check if required
    if config.backend.require_health_check && config.backend.auto_start {
        info!("Performing backend health check (timeout: {}ms)...",
              config.backend.health_check_timeout_ms);

        // Create tokio runtime for health check
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        match rt.block_on(health_check(&config)) {
            Ok(()) => {
                info!("Backend health check passed!");
            }
            Err(e) => {
                error!("Backend health check failed: {}", e);
                error!("Make sure the backend (omb) is running and sending MQTT messages.");
                error!("Exiting.");
                std::process::exit(1);
            }
        }
    }

    // Create and run the game
    let mut executor = Executor::new();
    executor.add_plugin(game::Game::new());
    executor.run();

    // Backend will be automatically stopped when backend_manager is dropped
    info!("OMFX shutting down...");
}
