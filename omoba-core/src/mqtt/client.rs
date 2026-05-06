//! MQTT 用戶端包裝器

use rumqttc::{AsyncClient, MqttOptions, QoS, EventLoop, Event, Packet};
use log::{info, warn, debug, error};
use anyhow::Result;

use crate::config::ServerConfig;

/// 遊戲通訊MQTT客戶端
pub struct MqttClient {
    client: AsyncClient,
    event_loop: EventLoop,
    player_name: String,
}

/// 供前端處理的 MQTT 事件
#[derive(Debug, Clone)]
pub enum MqttEvent {
    Connected,
    Disconnected,
    Message { topic: String, payload: Vec<u8> },
    Error(String),
}

impl MqttClient {
    /// 建立新的 MQTT 用戶端
    pub fn new(config: &ServerConfig, player_name: &str, client_id: &str) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(
            client_id,
            &config.mqtt_host,
            config.mqtt_port,
        );
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(30));

        let (client, event_loop) = AsyncClient::new(mqttoptions, 10);

        info!("MQTT client created - Host: {}:{}, Client ID: {}",
              config.mqtt_host, config.mqtt_port, client_id);

        Ok(Self {
            client,
            event_loop,
            player_name: player_name.to_string(),
        })
    }

    /// 訂閱遊戲主題
    pub async fn subscribe_to_game_topics(&self) -> Result<()> {
        // 訂閱廣播訊息
        self.client.subscribe("td/all/res", QoS::AtMostOnce).await?;

        // 訂閱特定於玩家的訊息
        let player_topic = format!("td/{}/send", self.player_name);
        self.client.subscribe(&player_topic, QoS::AtMostOnce).await?;

        // 訂閱螢幕回應
        let screen_topic = format!("td/{}/screen_response", self.player_name);
        self.client.subscribe(&screen_topic, QoS::AtMostOnce).await?;

        // 訂閱能力測驗答案
        self.client.subscribe("ability_test/response", QoS::AtMostOnce).await?;

        info!("Subscribed to game topics for player: {}", self.player_name);

        Ok(())
    }

    /// 發布玩家動作
    pub async fn publish_action(&self, action: &str, data: serde_json::Value) -> Result<()> {
        let topic = format!("td/{}/action", self.player_name);
        let message = serde_json::json!({
            "t": "player_action",
            "a": action,
            "d": data
        });

        let payload = serde_json::to_string(&message)?;
        self.client.publish(&topic, QoS::AtMostOnce, false, payload).await?;

        debug!("Published action: {} to {}", action, topic);

        Ok(())
    }

    /// 下次活動的投票
    pub async fn poll(&mut self) -> Option<MqttEvent> {
        match self.event_loop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                Some(MqttEvent::Message {
                    topic: publish.topic.to_string(),
                    payload: publish.payload.to_vec(),
                })
            }
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                info!("MQTT connected");
                Some(MqttEvent::Connected)
            }
            Ok(Event::Incoming(Packet::Disconnect)) => {
                warn!("MQTT disconnected");
                Some(MqttEvent::Disconnected)
            }
            Ok(_) => None,
            Err(e) => {
                error!("MQTT error: {}", e);
                Some(MqttEvent::Error(e.to_string()))
            }
        }
    }

    /// 取得玩家姓名
    pub fn player_name(&self) -> &str {
        &self.player_name
    }
}
