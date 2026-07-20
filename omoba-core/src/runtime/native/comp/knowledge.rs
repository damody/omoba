//! `KnowledgeBonusResource` — ECS resource 持有當前對局已解鎖知識節點的
//! 加成映射，由 `omb` 在初始化時填入。
//!
//! 設計：host 端唯讀 resource，不進 lockstep 廣播；每局初始化一次。

use std::collections::HashMap;

/// 知識加成 ECS resource。
///
/// - `enabled`: false → CHIMPS 模式或系統關閉，略過所有加成。
/// - `bonuses_by_category`: category string（如 "tower_dart", "global"）
///   → Vec<(buff_id, json_payload)>，由 `omb::knowledge::loader::build_bonus_map` 建立。
#[derive(Default)]
pub struct KnowledgeBonusResource {
    pub enabled: bool,
    /// category → Vec<(buff_id, payload_json)>
    pub bonuses_by_category: HashMap<String, Vec<(String, String)>>,
    /// 已解鎖節點 id 列表（供 `get_unlocked_knowledge_nodes` FFI 回傳）。
    pub unlocked_nodes: Vec<String>,
}

impl KnowledgeBonusResource {
    /// 回傳指定 category 的加成列表（空 slice = 無加成）。
    pub fn bonuses_for(&self, category: &str) -> &[(String, String)] {
        self.bonuses_by_category
            .get(category)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 取得 global 加成。
    pub fn global_bonuses(&self) -> &[(String, String)] {
        self.bonuses_for("global")
    }
}
