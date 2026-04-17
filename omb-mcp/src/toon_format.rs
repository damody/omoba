use serde_json::Value;
use toon_format::EncodeOptions;

/// 將 JSON 資料轉換為 TOON 格式字串
pub fn json_to_toon(value: &Value) -> String {
    toon_format::encode(value, &EncodeOptions::default())
        .unwrap_or_else(|e| format!("TOON encode error: {}", e))
}
