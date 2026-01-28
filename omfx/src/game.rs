//! Main game implementation

use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    event::{ElementState, Event, WindowEvent},
    gui::message::UiMessage,
    keyboard::{KeyCode, PhysicalKey},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::{
        Scene,
        base::BaseBuilder,
        camera::{CameraBuilder, Projection, OrthographicProjection},
        node::Node,
        transform::TransformBuilder,
    },
};
use log::{debug, info, warn};

use omoba_core::{AppConfig, GameState, MqttEvent, MqttHandler};

use crate::camera::CameraController;
use crate::config::OmfxConfig;
use crate::debug::DebugOverlays;
use crate::mqtt_worker::{GameMessage, MqttWorker};
use crate::renderer::{EffectRenderer, EntityRenderer, FogOfWarRenderer, HealthBarRenderer};

/// Main game state
#[derive(Visit, Reflect)]
pub struct Game {
    #[visit(skip)]
    #[reflect(hidden)]
    scene: Handle<Scene>,
    #[visit(skip)]
    #[reflect(hidden)]
    core_config: AppConfig,
    #[visit(skip)]
    #[reflect(hidden)]
    omfx_config: OmfxConfig,
    #[visit(skip)]
    #[reflect(hidden)]
    game_state: GameState,
    #[visit(skip)]
    #[reflect(hidden)]
    mqtt_worker: Option<MqttWorker>,
    #[visit(skip)]
    #[reflect(hidden)]
    mqtt_handler: MqttHandler,
    #[visit(skip)]
    #[reflect(hidden)]
    entity_renderer: Option<EntityRenderer>,
    #[visit(skip)]
    #[reflect(hidden)]
    effect_renderer: Option<EffectRenderer>,
    #[visit(skip)]
    #[reflect(hidden)]
    fog_renderer: Option<FogOfWarRenderer>,
    #[visit(skip)]
    #[reflect(hidden)]
    healthbar_renderer: Option<HealthBarRenderer>,
    #[visit(skip)]
    #[reflect(hidden)]
    camera: CameraController,
    #[visit(skip)]
    #[reflect(hidden)]
    camera_node: Handle<Node>,
    #[visit(skip)]
    #[reflect(hidden)]
    debug_overlays: DebugOverlays,
    #[visit(skip)]
    #[reflect(hidden)]
    is_connected: bool,
    #[visit(skip)]
    #[reflect(hidden)]
    selected_entity_id: Option<u32>,
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
        let core_config = AppConfig::load();
        let omfx_config = OmfxConfig::load();
        let game_state = GameState::new(
            core_config.frontend.player_name.clone(),
            core_config.frontend.hero_type.clone(),
        );

        let mut camera = CameraController::new();
        camera.config.max_speed = omfx_config.camera.edge_scroll_speed;
        camera.config.edge_size = omfx_config.camera.edge_scroll_zone;
        camera.config.zoom_speed = omfx_config.camera.zoom_speed;
        camera.config.min_zoom = omfx_config.camera.min_zoom;
        camera.config.max_zoom = omfx_config.camera.max_zoom;
        camera.zoom = omfx_config.camera.default_zoom;

        Self {
            scene: Handle::NONE,
            core_config,
            omfx_config,
            game_state,
            mqtt_worker: None,
            mqtt_handler: MqttHandler::new(),
            entity_renderer: None,
            effect_renderer: None,
            fog_renderer: None,
            healthbar_renderer: None,
            camera,
            camera_node: Handle::NONE,
            debug_overlays: DebugOverlays::new(),
            is_connected: false,
            selected_entity_id: None,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register custom types here if needed
    }

    fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) {
        info!("Initializing OMFX game plugin");

        // Create a new empty scene for 2D rendering
        let mut scene = Scene::new();

        // Create 2D orthographic camera
        // Position camera at z=10 looking at z=0 (where entities are)
        // Use orthographic projection for 2D rendering
        self.camera_node = CameraBuilder::new(
            BaseBuilder::new()
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(Vector3::new(400.0, 300.0, 10.0))
                        .build()
                )
        )
        .with_projection(Projection::Orthographic(OrthographicProjection {
            // Vertical size: how many world units visible vertically
            // 600 means 600 units from top to bottom
            vertical_size: 600.0,
            // Near/far clipping planes
            z_near: 0.01,
            z_far: 100.0,
        }))
        .build(&mut scene.graph);

        info!("Created 2D camera at (400, 300, 10) with orthographic projection");

        self.scene = context.scenes.add(scene);

        // Initialize renderers
        self.entity_renderer = Some(EntityRenderer::new(self.core_config.frontend.player_name.clone()));
        self.effect_renderer = Some(EffectRenderer::new());
        self.fog_renderer = Some(FogOfWarRenderer::new());
        self.healthbar_renderer = Some(HealthBarRenderer::new());

        // Configure fog renderer
        if let Some(ref mut fog) = self.fog_renderer {
            fog.config.tile_size = self.omfx_config.render.fog_tile_size;
        }

        // Configure health bar renderer
        if let Some(ref mut hb) = self.healthbar_renderer {
            hb.config.width = self.omfx_config.render.health_bar_width;
            hb.config.height = self.omfx_config.render.health_bar_height;
        }

        // Initialize MQTT worker (background thread)
        match MqttWorker::new(
            &self.core_config.server,
            &self.core_config.frontend.player_name,
        ) {
            Ok(worker) => {
                self.mqtt_worker = Some(worker);
                info!("MQTT worker started");
            }
            Err(e) => {
                log::error!("Failed to create MQTT worker: {}", e);
            }
        }

        info!(
            "OMFX initialized - Player: {}, Hero: {}",
            self.core_config.frontend.player_name, self.core_config.frontend.hero_type
        );
    }

    fn update(&mut self, context: &mut PluginContext) {
        let dt = context.dt;

        // Process MQTT messages from background worker
        if let Some(ref worker) = self.mqtt_worker {
            while let Some(msg) = worker.try_recv() {
                match msg {
                    GameMessage::MqttEvent(MqttEvent::Message { topic, payload }) => {
                        if let Err(e) = self.mqtt_handler.handle_message(
                            &topic,
                            &payload,
                            &mut self.game_state
                        ) {
                            warn!("Failed to handle MQTT message: {}", e);
                        }
                    }
                    GameMessage::MqttEvent(MqttEvent::Connected) => {
                        self.is_connected = true;
                        info!("MQTT connected");
                    }
                    GameMessage::MqttEvent(MqttEvent::Disconnected) => {
                        self.is_connected = false;
                        warn!("MQTT disconnected");
                    }
                    GameMessage::MqttEvent(MqttEvent::Error(e)) => {
                        warn!("MQTT error: {}", e);
                    }
                    GameMessage::Error(e) => {
                        warn!("MQTT worker error: {}", e);
                    }
                }
            }
        }

        // Update game state cooldowns
        self.game_state.update_cooldowns(dt);

        // Update camera
        self.camera.update(dt);

        // Get scene for rendering
        if let Some(scene) = context.scenes.try_get_mut(self.scene) {
            // Sync entity visuals
            if let Some(ref mut renderer) = self.entity_renderer {
                renderer.sync_with_game_state(&self.game_state.entities, scene);
            }

            // Update effects
            if let Some(ref mut effects) = self.effect_renderer {
                effects.update(scene);
            }

            // Update fog of war
            if let Some(ref mut fog) = self.fog_renderer {
                let (viewport_min, viewport_max) = self.game_state.viewport.get_bounds();
                // Convert vek::Vec2 to fyrox::Vector2
                let viewport_min = Vector2::new(viewport_min.x, viewport_min.y);
                let viewport_max = Vector2::new(viewport_max.x, viewport_max.y);
                fog.update(viewport_min, viewport_max, scene);
            }

            // Render health bars
            if let Some(ref mut healthbars) = self.healthbar_renderer {
                healthbars.render(scene);
            }

            // Render debug overlays
            self.debug_overlays.render(scene);
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, context: PluginContext) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    // Handle debug overlays (F1-F4)
                    if let Some(scene) = context.scenes.try_get_mut(self.scene) {
                        let handled =
                            self.debug_overlays
                                .handle_key(key_event.physical_key, key_event.state, scene);
                        if handled {
                            return;
                        }
                    }

                    // Handle other keys on press only
                    if key_event.state == ElementState::Pressed {
                        if let PhysicalKey::Code(key_code) = key_event.physical_key {
                            match key_code {
                                KeyCode::KeyP => {
                                    self.game_state.toggle_pause();
                                    info!("Paused: {}", self.game_state.is_paused);
                                }
                                KeyCode::Digit1 => {
                                    self.game_state.set_time_scale(1.0);
                                    info!("Time scale: 1.0x");
                                }
                                KeyCode::Digit2 => {
                                    self.game_state.set_time_scale(2.0);
                                    info!("Time scale: 2.0x");
                                }
                                KeyCode::Digit3 => {
                                    self.game_state.set_time_scale(4.0);
                                    info!("Time scale: 4.0x");
                                }
                                KeyCode::Digit0 => {
                                    self.game_state.set_time_scale(0.5);
                                    info!("Time scale: 0.5x");
                                }
                                KeyCode::Home => {
                                    self.camera.go_to_center();
                                    info!("Camera centered");
                                }
                                KeyCode::Space => {
                                    // Focus on local player
                                    let pos = self.game_state.local_player.position;
                                    self.camera.focus_on(Vector2::new(pos.x, pos.y));
                                    info!("Camera focused on player");
                                }
                                _ => {
                                    debug!("Unhandled key: {:?}", key_code);
                                }
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.camera
                        .on_mouse_move(Vector2::new(position.x, position.y));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom_delta = match delta {
                        fyrox::event::MouseScrollDelta::LineDelta(_, y) => *y,
                        fyrox::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                    };
                    self.camera.on_zoom(zoom_delta);
                }
                WindowEvent::Resized(size) => {
                    self.camera.set_window_size(size.width as f32, size.height as f32);
                }
                _ => {}
            }
        }
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage) {
        // Handle UI messages
    }
}
