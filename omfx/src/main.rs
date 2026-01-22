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
mod mqtt_worker;
mod renderer;
mod ui;

use backend::BackendManager;
use cli::CliArgs;
use config::OmfxConfig;
use omoba_core::{MqttClient, MqttEvent};

/// Perform health check by connecting to MQTT FIRST, then starting backend
/// This ensures we don't miss the initial heartbeat
async fn health_check_with_backend_start(
    config: &OmfxConfig,
    backend_manager: &mut BackendManager,
) -> Result<(), String> {
    let server_config = config.to_core_server_config();

    // Step 1: Connect to MQTT FIRST (before starting backend)
    let mut mqtt_client = MqttClient::new(
        &server_config,
        &config.frontend.player_name,
        "omfx_health_check",
    ).map_err(|e| format!("Failed to create MQTT client: {}", e))?;

    info!("Health check: Connecting to MQTT broker {}:{} BEFORE starting backend...",
          config.server.mqtt_host, config.server.mqtt_port);

    // Step 2: Wait for MQTT connection and subscribe
    let connect_timeout = Duration::from_secs(5);
    let connect_result = timeout(connect_timeout, async {
        loop {
            if let Some(event) = mqtt_client.poll().await {
                match event {
                    MqttEvent::Connected => {
                        info!("Health check: MQTT connected successfully");
                        // Subscribe to topics BEFORE starting backend
                        if let Err(e) = mqtt_client.subscribe_to_game_topics().await {
                            return Err(format!("Failed to subscribe: {}", e));
                        }
                        info!("Health check: Subscribed to game topics");
                        return Ok(());
                    }
                    MqttEvent::Error(e) => {
                        return Err(format!("MQTT error: {}", e));
                    }
                    MqttEvent::Disconnected => {
                        return Err("MQTT disconnected".to_string());
                    }
                    _ => {}
                }
            }
        }
    }).await;

    match connect_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("MQTT connection timeout".to_string()),
    }

    // Step 3: NOW start the backend (after we're already subscribed)
    info!("Health check: Starting backend NOW (MQTT already subscribed)...");
    if let Err(e) = backend_manager.start() {
        return Err(format!("Failed to start backend: {}", e));
    }

    // Step 4: Wait for heartbeat message from backend
    let timeout_duration = Duration::from_millis(config.backend.health_check_timeout_ms);
    info!("Health check: Waiting for heartbeat from backend (timeout: {}ms)...",
          config.backend.health_check_timeout_ms);

    let result = timeout(timeout_duration, async {
        loop {
            if let Some(event) = mqtt_client.poll().await {
                match event {
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
                    _ => {}
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

    // Create backend manager
    let mut backend_manager = BackendManager::new(config.backend.clone());

    // If auto_start and health check are both enabled, use the integrated flow
    // This ensures MQTT is subscribed BEFORE backend starts (to catch initial heartbeat)
    if config.backend.auto_start && config.backend.require_health_check {
        info!("Starting integrated backend health check flow...");
        info!("  1. Connect to MQTT and subscribe");
        info!("  2. Start backend");
        info!("  3. Wait for heartbeat");

        // Create tokio runtime for health check
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        match rt.block_on(health_check_with_backend_start(&config, &mut backend_manager)) {
            Ok(()) => {
                info!("Backend health check passed! Backend is running and sending heartbeats.");
            }
            Err(e) => {
                error!("Backend health check failed: {}", e);
                error!("Make sure the MQTT broker is running at {}:{}",
                       config.server.mqtt_host, config.server.mqtt_port);
                error!("Exiting.");
                std::process::exit(1);
            }
        }
    } else if config.backend.auto_start {
        // Just start backend without health check
        info!("Auto-starting backend (health check disabled)...");
        std::thread::sleep(std::time::Duration::from_millis(config.backend.start_delay_ms));

        if let Err(e) = backend_manager.start() {
            error!("Failed to start backend: {}", e);
            warn!("Continuing without backend (require_health_check is disabled)");
        }
    }

    // Create and run the game
    let mut executor = Executor::new();
    executor.add_plugin(game::Game::new());
    executor.run();

    // Backend will be automatically stopped when backend_manager is dropped
    info!("OMFX shutting down...");
}
