//! MQTT background worker
//!
//! Handles MQTT polling in a background thread and sends events to the main thread.

use std::sync::mpsc;
use std::thread;
use log::{info, error, debug};
use omoba_core::{MqttClient, MqttEvent, ServerConfig};

/// Message types sent from worker to main thread
pub enum GameMessage {
    /// MQTT event received
    MqttEvent(MqttEvent),
    /// Worker error
    Error(String),
}

/// MQTT background worker
pub struct MqttWorker {
    /// Channel to receive messages from worker
    receiver: mpsc::Receiver<GameMessage>,
    /// Channel to send shutdown signal to worker
    shutdown_tx: mpsc::Sender<()>,
    /// Worker thread handle
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl MqttWorker {
    /// Create and start a new MQTT worker
    pub fn new(server_config: &ServerConfig, player_name: &str) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let config = server_config.clone();
        let name = player_name.to_string();

        let handle = thread::spawn(move || {
            info!("MQTT worker thread started");

            // Create tokio runtime for async MQTT operations
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(GameMessage::Error(format!("Failed to create runtime: {}", e)));
                    return;
                }
            };

            rt.block_on(async {
                // Create MQTT client
                let mut client = match MqttClient::new(&config, &name, "omfx_game") {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(GameMessage::Error(format!("Failed to create MQTT client: {}", e)));
                        return;
                    }
                };

                // 等待連線完成 (ConnAck) - 必須在訂閱之前完成
                loop {
                    if let Some(event) = client.poll().await {
                        match &event {
                            MqttEvent::Connected => {
                                debug!("MQTT worker: Connection established");
                                let _ = tx.send(GameMessage::MqttEvent(event));
                                break;
                            }
                            MqttEvent::Error(e) => {
                                let _ = tx.send(GameMessage::Error(format!("Connection failed: {}", e)));
                                return;
                            }
                            _ => {}
                        }
                    }
                }

                // 連線成功後才訂閱（確保 broker 已確認連線）
                if let Err(e) = client.subscribe_to_game_topics().await {
                    let _ = tx.send(GameMessage::Error(format!("Failed to subscribe: {}", e)));
                    return;
                }

                info!("MQTT worker subscribed to game topics");

                // Main polling loop
                loop {
                    // Check for shutdown signal (non-blocking)
                    if shutdown_rx.try_recv().is_ok() {
                        info!("MQTT worker received shutdown signal");
                        break;
                    }

                    // Poll MQTT with timeout
                    if let Some(event) = client.poll().await {
                        debug!("MQTT worker received event: {:?}",
                            match &event {
                                MqttEvent::Connected => "Connected",
                                MqttEvent::Disconnected => "Disconnected",
                                MqttEvent::Message { topic, .. } => topic,
                                MqttEvent::Error(_) => "Error",
                            }
                        );

                        if tx.send(GameMessage::MqttEvent(event)).is_err() {
                            // Main thread dropped receiver, exit
                            break;
                        }
                    }
                }
            });

            info!("MQTT worker thread exiting");
        });

        Ok(Self {
            receiver: rx,
            shutdown_tx,
            thread_handle: Some(handle),
        })
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Option<GameMessage> {
        self.receiver.try_recv().ok()
    }

    /// Shutdown the worker
    pub fn shutdown(&mut self) {
        info!("Shutting down MQTT worker");
        let _ = self.shutdown_tx.send(());

        if let Some(handle) = self.thread_handle.take() {
            // Give the thread some time to shutdown gracefully
            let _ = handle.join();
        }
    }
}

impl Drop for MqttWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}
