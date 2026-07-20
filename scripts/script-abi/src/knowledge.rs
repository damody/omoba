//! 將軍知識（General Knowledge）節點的 abi_stable 型別。
//! 供 script 側透過 `GameWorld::get_unlocked_knowledge_nodes()` 查詢用。
//! 只暴露腳本需要的最小介面；資料載入與解鎖邏輯在 host 端。

use abi_stable::std_types::{ROption, RString, RVec};

/// 單一 Knowledge 節點加成項（abi_stable）。
#[repr(C)]
#[derive(Clone, Debug, abi_stable::StableAbi)]
pub struct KnowledgeBonusFFI {
    pub stat_key: RString,
    /// additive 加成（對應 `sum_add`）。無加法加成時為 0。
    pub add: i64,
    /// multiplicative 加成（對應 `product_mult`，已乘 1024 的 Fixed64 raw 值）。
    /// 無乘法加成時為 0。
    pub multiply_raw: i64,
}

/// 單一 Knowledge 節點（abi_stable）。
#[repr(C)]
#[derive(Clone, Debug, abi_stable::StableAbi)]
pub struct KnowledgeNodeFFI {
    pub id: RString,
    pub category: RString,
    pub kp_cost: u32,
    pub requires: RVec<RString>,
    pub bonuses: RVec<KnowledgeBonusFFI>,
}

/// 查詢結果包裝，供 script 判斷節點是否存在。
pub type KnowledgeNodeOption = ROption<KnowledgeNodeFFI>;
