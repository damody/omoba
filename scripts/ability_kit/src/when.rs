//! v3 stat `when` (E508): a stat row that holds only under a condition. `self.is_hero` and
//! `primary_is(self, attr)` do not change while the row stays on its holder, so they are judged
//! when the row goes on; `game.is_night` / `not game.is_night` change during a match, so those
//! rows ride in the payload's `__when_v1` (omoba-core `rules::world::WHEN_KEY`) and the host
//! rewrites them when the night starts or ends (`sync_when_rows`).
use crate::codegen::{Cx, Res, StatKey, StatusDef};
use omb_script_abi::types::{EntityHandle, Fixed64};

/// A hero attribute (`primary_is(u, attr)`), in the order of the hero catalog's `primary_attribute`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attr { Strength, Agility, Intelligence, Universal }

/// When a stat row counts. `NotPrimary(attr)`: v3 `not primary_is(self, attr)` (不敗大劍's +15 to
/// the two attributes that are not its holder's primary one; a holder that is no hero gets it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When { Always, Night, Day, Primary(Attr), NotPrimary(Attr), Hero }

impl When {
    /// Whether a row of this condition counts for a holder that is a hero or not and has this
    /// primary attribute (`None`: no hero); `None` for a time-of-day row, which the host keeps
    /// rewriting.
    pub fn holds(self, hero: bool, primary: Option<Attr>) -> Option<bool> {
        match self {
            When::Always => Some(true),
            When::Hero => Some(hero),
            When::Primary(attr) => Some(primary == Some(attr)),
            When::NotPrimary(attr) => Some(primary != Some(attr)),
            When::Night | When::Day => None,
        }
    }
}

/// The `__when_v1` row of the time-of-day stats: `base` holds every unconditional stat, `night`
/// and `day` the ones of that half; a key given twice in one half adds up.
fn timed_row(base: &[(StatKey, i64)], night: &[(StatKey, i64)], day: &[(StatKey, i64)]) -> serde_json::Value {
    let half = |rows: &[(StatKey, i64)]| {
        let mut map = serde_json::Map::new();
        for (key, raw) in rows {
            let sum = map.get(key.as_str()).and_then(serde_json::Value::as_i64).unwrap_or(0).saturating_add(*raw);
            map.insert(key.as_str().into(), sum.into());
        }
        serde_json::Value::Object(map)
    };
    serde_json::json!({ "base": half(base), "night": half(night), "day": half(day) })
}

impl Cx<'_, '_> {
    /// v3 `primary_is(u, attr)`: `u` is a hero whose primary attribute is `attr`.
    pub fn primary_is(&self, u: EntityHandle, attr: Attr) -> bool {
        self.primary_of(u) == Some(attr)
    }

    /// The primary attribute of `u` when it is a catalog hero.
    pub fn primary_of(&self, u: EntityHandle) -> Option<Attr> {
        self.world.unit_id_of(u).into_option().and_then(|id| {
            let id = id.as_str();
            omoba_template_ids::hero_by_name(id.strip_prefix("hero_").unwrap_or(id))
        }).and_then(omoba_template_ids::active_hero_stats).and_then(|stats| match stats.primary_attribute {
            0 => Some(Attr::Strength), 1 => Some(Attr::Agility), 2 => Some(Attr::Intelligence), 3 => Some(Attr::Universal), _ => None,
        })
    }

    /// The rows that count for `holder` now; time-of-day rows go to the next status or passive
    /// row this run writes ([`Cx::when_row`]).
    fn split_when(&mut self, holder: EntityHandle, stats: &[(StatKey, i64, When)]) -> Vec<(StatKey, i64)> {
        let (mut base, mut night, mut day) = (Vec::new(), Vec::new(), Vec::new());
        let (hero, primary) = (self.is_hero(holder), self.primary_of(holder));
        for &(key, raw, when) in stats {
            match when.holds(hero, primary) {
                Some(true) => base.push((key, raw)),
                Some(false) => {}
                None if when == When::Night => night.push((key, raw)),
                None => day.push((key, raw)),
            }
        }
        self.when_row = (!night.is_empty() || !day.is_empty()).then(|| (timed_row(&base, &night, &day), self.is_night()));
        base
    }

    /// The rows of `stats` that count for `holder` now; time-of-day rows go to the next status row
    /// written, [`Cx::clear_when_row`] drops them when no row took them.
    pub fn when_rows(&mut self, holder: EntityHandle, stats: &[(StatKey, i64, When)]) -> Vec<(StatKey, i64)> {
        self.split_when(holder, stats)
    }

    pub fn clear_when_row(&mut self) {
        self.when_row = None;
    }

    /// [`Cx::apply_status`] with conditional stat rows.
    pub fn apply_status_when(&mut self, to: EntityHandle, status: &StatusDef, duration: Fixed64, stats: &[(StatKey, i64, When)]) -> Res {
        let rows = self.split_when(to, stats);
        let applied = self.apply_status(to, status, duration, &rows);
        self.when_row = None;
        applied
    }

    /// [`Cx::passive_stats`] with conditional stat rows (the learner holds them).
    pub fn passive_stats_when(&mut self, id: &str, stats: &[(StatKey, i64, When)]) -> Res {
        let holder = self.caster;
        let rows = self.split_when(holder, stats);
        let applied = self.passive_stats(id, &rows);
        self.when_row = None;
        applied
    }

    /// [`Cx::passive_marker`] with conditional stat rows (the learner holds them).
    pub fn passive_marker_when(&mut self, id: &str, stats: &[(StatKey, i64, When)], mask: u32) -> Res {
        let holder = self.caster;
        let rows = self.split_when(holder, stats);
        let applied = self.passive_marker(id, &rows, mask);
        self.when_row = None;
        applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `primary_is` / `not primary_is` (不敗大劍): a strength hero gets the agility and intellect
    /// rows of `not primary_is`, not the strength one; a holder that is no hero gets every one.
    #[test]
    fn primary_conditions_count_by_the_holders_primary_attribute() {
        let str_hero = (true, Some(Attr::Strength));
        assert_eq!(When::NotPrimary(Attr::Strength).holds(str_hero.0, str_hero.1), Some(false));
        assert_eq!(When::NotPrimary(Attr::Agility).holds(str_hero.0, str_hero.1), Some(true));
        assert_eq!(When::NotPrimary(Attr::Intelligence).holds(str_hero.0, str_hero.1), Some(true));
        assert_eq!(When::Primary(Attr::Strength).holds(str_hero.0, str_hero.1), Some(true));
        assert_eq!(When::NotPrimary(Attr::Strength).holds(false, None), Some(true), "no hero");
        assert_eq!(When::Primary(Attr::Strength).holds(false, None), Some(false));
        assert_eq!((When::Hero.holds(false, None), When::Night.holds(true, None)), (Some(false), None));
        // the hero catalog's primary_attribute numbers are the attributes in order
        assert_eq!((Attr::Strength as u8, Attr::Agility as u8, Attr::Intelligence as u8, Attr::Universal as u8), (0, 1, 2, 3));
    }

    /// The row keeps each half apart and adds a key given twice; the host resolves it by the time
    /// of day (rules::world::resolve_when).
    #[test]
    fn a_timed_row_resolves_to_base_plus_the_half_that_holds() {
        use omoba_core::runtime::native::rules::world::{resolve_when, WHEN_KEY};
        let row = timed_row(&[(StatKey::PhysicalArmorBonus, 2048)], &[(StatKey::PhysicalArmorBonus, 1024), (StatKey::PhysicalArmorBonus, 1024)],
            &[(StatKey::MoveSpeedBonus, 5120)]);
        let mut payload = serde_json::Map::new();
        payload.insert(WHEN_KEY.into(), row);
        assert!(resolve_when(&mut payload, true));
        assert_eq!((payload["physical_armor_bonus"].as_i64(), payload["move_speed_bonus"].as_i64()), (Some(4096), Some(0)));
        assert!(resolve_when(&mut payload, false));
        assert_eq!((payload["physical_armor_bonus"].as_i64(), payload["move_speed_bonus"].as_i64()), (Some(2048), Some(5120)));
    }
}
