//! Main game implementation

use fyrox::{
    core::{
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    event::{Event, WindowEvent},
    gui::message::UiMessage,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::Scene,
};
use log::{info, debug};

use omoba_core::{AppConfig, GameState, MqttClient, MqttHandler};

use crate::renderer::EntityRenderer;

/// Main game state
#[derive(Visit, Reflect)]
pub struct Game {
    #[visit(skip)]
    #[reflect(hidden)]
    scene: Handle<Scene>,
    #[visit(skip)]
    #[reflect(hidden)]
    config: AppConfig,
    #[visit(skip)]
    #[reflect(hidden)]
    game_state: GameState,
    #[visit(skip)]
    #[reflect(hidden)]
    mqtt_client: Option<MqttClient>,
    #[visit(skip)]
    #[reflect(hidden)]
    mqtt_handler: MqttHandler,
    #[visit(skip)]
    #[reflect(hidden)]
    entity_renderer: Option<EntityRenderer>,
    #[visit(skip)]
    #[reflect(hidden)]
    is_connected: bool,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("scene", &self.scene)
            .field("is_connected", &self.is_connected)
            .finish_non_exhaustive()
    }
}

impl Game {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let game_state = GameState::new(
            config.frontend.player_name.clone(),
            config.frontend.hero_type.clone(),
        );

        Self {
            scene: Handle::NONE,
            config,
            game_state,
            mqtt_client: None,
            mqtt_handler: MqttHandler::new(),
            entity_renderer: None,
            is_connected: false,
        }
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register custom types here if needed
    }

    fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) {
        info!("Initializing OMFX game plugin");

        // Create a new empty scene for 2D rendering
        let scene = Scene::new();
        self.scene = context.scenes.add(scene);

        // Initialize entity renderer
        self.entity_renderer = Some(EntityRenderer::new(
            self.config.frontend.player_name.clone()
        ));

        // Initialize MQTT client
        match MqttClient::new(
            &self.config.server,
            &self.config.frontend.player_name,
            "omfx_client",
        ) {
            Ok(client) => {
                self.mqtt_client = Some(client);
                info!("MQTT client initialized");
            }
            Err(e) => {
                log::error!("Failed to create MQTT client: {}", e);
            }
        }

        info!("OMFX initialized - Player: {}, Hero: {}",
              self.config.frontend.player_name,
              self.config.frontend.hero_type);
    }

    fn update(&mut self, context: &mut PluginContext) {
        // Update game state cooldowns
        let dt = context.dt;
        self.game_state.update_cooldowns(dt);

        // Process MQTT messages (non-blocking)
        // Note: Full async integration will be added in later tasks

        // Sync entity visuals with game state
        if let Some(ref mut renderer) = self.entity_renderer {
            if let Some(scene) = context.scenes.try_get_mut(self.scene) {
                renderer.sync_with_game_state(&self.game_state.entities, scene);
            }
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, _context: PluginContext) {
        // Handle OS events (keyboard, mouse)
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    // Handle keyboard input
                    debug!("Key event: {:?}", key_event);
                }
                WindowEvent::CursorMoved { .. } => {
                    // Handle mouse movement for edge scrolling
                    // Will be implemented in camera task
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    // Handle zoom
                    debug!("Mouse wheel: {:?}", delta);
                }
                _ => {}
            }
        }
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage) {
        // Handle UI messages
    }
}
