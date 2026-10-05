//! `BuffId` — 5 個 standard status buff 的 enum 型別保護。
//!
//! 這個 module 與 [`crate::stat_keys`] 語義分離：
//! * `StatKey` 是 Dota 2 modifier property（`move_speed_bonus` 等屬性修改 key）。
//! * `BuffId` 是 buff identifier（`stun` / `root` 等狀態 buff 的名稱）。
//!
//! `GameWorld::add_buff` / `has_buff` / `remove_buff` 等 FFI API 仍收 `RStr<'_>`，
//! 因此 custom buff（如英雄 toggle buff id）仍可自由傳字串；standard 的 5 個
//! 狀態 buff 則建議走 `BuffId::Foo.as_rstr()` 取得型別保護並消除散亂 literal。

use abi_stable::{std_types::RStr, StableAbi};

/// Shared identity for declarative mana buffs, scoped to one caster life.
/// Pure formatting helper; does not change the FFI layout or add runtime deps.
pub fn generic_mana_buff_id(ability:&str,stat:&str,caster:crate::types::EntityHandle) -> String {
    format!("generic_mana:{ability}:{stat}:{}:{}",caster.id,caster.gen)
}

pub fn generic_slow_buff_id(ability: &str, caster: crate::types::EntityHandle) -> String {
    format!("generic_slow:{ability}:{}:{}", caster.id, caster.gen)
}

/// Host-local source parsing, never a wire identity. Reject ambiguous keys.
pub fn declarative_buff_source(key: &str) -> Option<(&str, Option<&str>)> {
    let parts: Vec<_> = key.split(':').collect();
    let (ability, stat, id, generation) = match parts.as_slice() {
        ["generic_slow", ability, id, generation] => (*ability, None, *id, *generation),
        ["generic_mana", ability, stat, id, generation] if !stat.is_empty() => (*ability, Some(*stat), *id, *generation),
        _ => return None,
    };
    if ability.is_empty() { return None; }
    for value in [id, generation] {
        let number: u32 = value.parse().ok()?;
        if number.to_string() != value { return None; }
    }
    Some((ability, stat))
}

/// 標準狀態 buff 識別字（stun / root / silence / invisible / invulnerable）。
///
/// ＃ 安全
/// Variant 順序 = FFI ABI 契約：新增只能**追加到尾端**，絕不可在中間 insert
/// 或更動 discriminant 值，否則 host 與 script DLL 版本不同步會 UB。
/// 每個 variant 顯式寫 `= N` 以鎖定值。
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BuffId {
    Stun = 0,
    Root = 1,
    Silence = 2,
    Invisible = 3,
    Invulnerable = 4,
}

impl BuffId {
    /// Stable wire format 字串（對應 BuffStore 內部 buff id key）。
    pub const fn as_str(self) -> &'static str {
        match self {
            BuffId::Stun => "stun",
            BuffId::Root => "root",
            BuffId::Silence => "silence",
            BuffId::Invisible => "invisible",
            BuffId::Invulnerable => "invulnerable",
        }
    }

    /// 給 FFI API（`GameWorld::add_buff` 等收 `RStr<'static>`）方便呼叫。
    pub const fn as_rstr(self) -> RStr<'static> {
        RStr::from_str(self.as_str())
    }

    /// 反向查詢（host 端若收到字串需判斷是否為 standard buff）。
    pub fn from_str_key(s: &str) -> Option<BuffId> {
        match s {
            "stun" => Some(BuffId::Stun),
            "root" => Some(BuffId::Root),
            "silence" => Some(BuffId::Silence),
            "invisible" => Some(BuffId::Invisible),
            "invulnerable" => Some(BuffId::Invulnerable),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn declarative_buff_source_identity_is_strict_and_shared() {
        let caster = crate::types::EntityHandle { id: 12, gen: 3 };
        assert_eq!(super::declarative_buff_source(&super::generic_slow_buff_id("a", caster)), Some(("a", None)));
        assert_eq!(super::declarative_buff_source(&super::generic_mana_buff_id("a", "mana_bonus", caster)), Some(("a", Some("mana_bonus"))));
        for key in ["generic_slow:a:12:3:extra", "generic_slow::12:3", "generic_slow:a:012:3",
            "generic_slow:a:-1:3", "generic_slow:a:4294967296:3", "generic_mana:a::12:3", "unknown:a:12:3"] {
            assert!(super::declarative_buff_source(key).is_none(), "{key}");
        }
    }
    #[test]
    fn generic_mana_buff_identity_is_shared_and_life_scoped() {
        let caster=crate::types::EntityHandle {id:12,gen:3};
        let id=super::generic_mana_buff_id("ranger_patch","mana_regen_constant",caster);
        assert_eq!(id,"generic_mana:ranger_patch:mana_regen_constant:12:3");
        assert_ne!(id,super::generic_mana_buff_id("ranger_patch","mana_regen_constant",
            crate::types::EntityHandle {gen:4,..caster}));
        assert_ne!(id,super::generic_mana_buff_id("other","mana_regen_constant",caster));
        assert_ne!(id,super::generic_mana_buff_id("ranger_patch","mana_bonus",caster));
    }
    use super::BuffId;

    const ALL: &[BuffId] = &[
        BuffId::Stun,
        BuffId::Root,
        BuffId::Silence,
        BuffId::Invisible,
        BuffId::Invulnerable,
    ];

    #[test]
    fn as_str_roundtrip() {
        for &b in ALL {
            assert_eq!(BuffId::from_str_key(b.as_str()), Some(b));
        }
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(BuffId::from_str_key("not_a_buff"), None);
    }

    #[test]
    fn as_rstr_matches_as_str() {
        for &b in ALL {
            assert_eq!(b.as_rstr().as_str(), b.as_str());
        }
    }
}
