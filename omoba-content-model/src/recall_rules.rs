//! Shared authored recall duration policy. Omitted economy uses the legacy default.
pub const DEFAULT_RECALL_CHANNEL_SECONDS:u32=8;
pub fn validate_recall_channel_seconds(seconds:u32)->Result<(),String>{
    if (1..=60).contains(&seconds){Ok(())}else{Err("MOBA recall channel must be 1..60 seconds".into())}
}
pub fn recall_channel_seconds(template:&serde_json::Value)->Result<u32,String>{
    let Some(economy)=template.get("moba_economy")else{return Ok(DEFAULT_RECALL_CHANNEL_SECONDS)};
    let seconds=economy.get("recall_channel_seconds").and_then(serde_json::Value::as_u64)
        .and_then(|v|u32::try_from(v).ok()).ok_or("MOBA recall channel must be an integer number of seconds")?;
    validate_recall_channel_seconds(seconds)?;Ok(seconds)
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn duration_policy_rejects_ambiguous_values_and_preserves_authority_default(){
        assert_eq!(recall_channel_seconds(&serde_json::json!({})).unwrap(),8);
        for value in [1,8,9,60]{assert_eq!(recall_channel_seconds(&serde_json::json!({"moba_economy":{"recall_channel_seconds":value}})).unwrap(),value);}
        for value in [serde_json::json!(0),serde_json::json!(61),serde_json::json!(-1),serde_json::json!(8.5),serde_json::json!("8"),serde_json::Value::Null]{
            assert!(recall_channel_seconds(&serde_json::json!({"moba_economy":{"recall_channel_seconds":value}})).is_err());
        }
        assert!(recall_channel_seconds(&serde_json::json!({"moba_economy":{}})).is_err());
        assert!(recall_channel_seconds(&serde_json::json!({"moba_economy":null})).is_err());
    }
}
