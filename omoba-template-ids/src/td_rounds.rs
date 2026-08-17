//! Canonical BTD-style TD round catalog shared by Lua content compilation and runtime setup.

#[allow(dead_code)] // build.rs compiles this module but only runtime setup uses the cadence.
pub const SPAWN_INTERVAL_SECS: f32 = 0.18;

/// Stable TD damage-profile bits.  These values are shared with the script ABI;
/// never renumber an existing bit.
pub mod damage_profile {
    pub const SHARP: u32 = 1 << 0;
    pub const EXPLOSIVE: u32 = 1 << 1;
    pub const ENERGY: u32 = 1 << 2;
    pub const FIRE: u32 = 1 << 3;
    pub const COLD: u32 = 1 << 4;
    pub const NORMAL: u32 = 1 << 5;
    pub const CRUSHING: u32 = 1 << 6;
    pub const TRUE: u32 = 1 << 7;
    pub const KNOWN: u32 = SHARP | EXPLOSIVE | ENERGY | FIRE | COLD | NORMAL | CRUSHING | TRUE;
}

pub mod layer_property {
    pub const CAMO: u32 = 1 << 0;
    pub const REGROW: u32 = 1 << 1;
    pub const FORTIFIED: u32 = 1 << 2;
    pub const MOAB_CLASS: u32 = 1 << 3;
}

/// Dependency-light, authoritative data for one layer in the TD graph.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TdLayerMetadata {
    pub id: &'static str,
    pub label: &'static str,
    pub hp: u32,
    pub move_speed: u32,
    pub children: &'static [&'static str],
    /// Cash credited when this layer is removed. Multi-HP shells intentionally
    /// carry more than one cash so the authored total remains pop-equivalent.
    pub cash: u32,
    /// Life damage if this complete layer graph reaches the exit.
    pub leak_value: u32,
    pub properties: u32,
    /// Tags which may damage this layer. `TRUE` is included unless a future
    /// content entry explicitly declares an invulnerable layer.
    pub accepted_damage: u32,
}

const ALL_DAMAGE: u32 = damage_profile::KNOWN;
const WITHOUT_EXPLOSIVE: u32 = ALL_DAMAGE & !damage_profile::EXPLOSIVE;
const WITHOUT_COLD: u32 = ALL_DAMAGE & !damage_profile::COLD;
const WITHOUT_ENERGY_FIRE: u32 = ALL_DAMAGE & !damage_profile::ENERGY & !damage_profile::FIRE;
const WITHOUT_SHARP: u32 = ALL_DAMAGE & !damage_profile::SHARP;
const WITHOUT_EXPLOSIVE_COLD: u32 = ALL_DAMAGE & !damage_profile::EXPLOSIVE & !damage_profile::COLD;

const PINK_CHILDREN: &[&str] = &["pink", "pink"];
const ZEBRA_CHILDREN: &[&str] = &["black", "white"];
const LEAD_CHILDREN: &[&str] = &["black", "black"];
const RAINBOW_CHILDREN: &[&str] = &["zebra", "zebra"];
const CERAMIC_CHILDREN: &[&str] = &["rainbow", "rainbow"];
const MOAB_CHILDREN: &[&str] = &["ceramic", "ceramic", "ceramic", "ceramic"];
const BFB_CHILDREN: &[&str] = &["moab", "moab", "moab", "moab"];
const ZOMG_CHILDREN: &[&str] = &["bfb", "bfb", "bfb", "bfb"];

pub const TD_LAYER_CATALOG: &[TdLayerMetadata] = &[
    TdLayerMetadata {
        id: "red",
        label: "Red",
        hp: 1,
        move_speed: 120,
        children: &[],
        cash: 1,
        leak_value: 1,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "blue",
        label: "Blue",
        hp: 1,
        move_speed: 140,
        children: &["red"],
        cash: 1,
        leak_value: 2,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "green",
        label: "Green",
        hp: 1,
        move_speed: 160,
        children: &["blue"],
        cash: 1,
        leak_value: 3,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "yellow",
        label: "Yellow",
        hp: 1,
        move_speed: 185,
        children: &["green"],
        cash: 1,
        leak_value: 4,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "pink",
        label: "Pink",
        hp: 1,
        move_speed: 220,
        children: &["yellow"],
        cash: 1,
        leak_value: 5,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "black",
        label: "Black",
        hp: 1,
        move_speed: 180,
        children: PINK_CHILDREN,
        cash: 1,
        leak_value: 11,
        properties: 0,
        accepted_damage: WITHOUT_EXPLOSIVE,
    },
    TdLayerMetadata {
        id: "white",
        label: "White",
        hp: 1,
        move_speed: 180,
        children: PINK_CHILDREN,
        cash: 1,
        leak_value: 11,
        properties: 0,
        accepted_damage: WITHOUT_COLD,
    },
    TdLayerMetadata {
        id: "purple",
        label: "Purple",
        hp: 1,
        move_speed: 180,
        children: PINK_CHILDREN,
        cash: 1,
        leak_value: 11,
        properties: 0,
        accepted_damage: WITHOUT_ENERGY_FIRE,
    },
    TdLayerMetadata {
        id: "zebra",
        label: "Zebra",
        hp: 1,
        move_speed: 120,
        children: ZEBRA_CHILDREN,
        cash: 1,
        leak_value: 23,
        properties: 0,
        accepted_damage: WITHOUT_EXPLOSIVE_COLD,
    },
    TdLayerMetadata {
        id: "lead",
        label: "Lead",
        hp: 1,
        move_speed: 120,
        children: LEAD_CHILDREN,
        cash: 1,
        leak_value: 23,
        properties: 0,
        accepted_damage: WITHOUT_SHARP,
    },
    TdLayerMetadata {
        id: "rainbow",
        label: "Rainbow",
        hp: 1,
        move_speed: 195,
        children: RAINBOW_CHILDREN,
        cash: 1,
        leak_value: 47,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "ceramic",
        label: "Ceramic",
        hp: 10,
        move_speed: 210,
        children: CERAMIC_CHILDREN,
        cash: 10,
        leak_value: 104,
        properties: 0,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "moab",
        label: "MOAB",
        hp: 200,
        move_speed: 80,
        children: MOAB_CHILDREN,
        cash: 200,
        leak_value: 616,
        properties: layer_property::MOAB_CLASS,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "bfb",
        label: "BFB",
        hp: 700,
        move_speed: 60,
        children: BFB_CHILDREN,
        cash: 700,
        leak_value: 3164,
        properties: layer_property::MOAB_CLASS,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "zomg",
        label: "ZOMG",
        hp: 4000,
        move_speed: 45,
        children: ZOMG_CHILDREN,
        cash: 4000,
        leak_value: 16656,
        properties: layer_property::MOAB_CLASS,
        accepted_damage: ALL_DAMAGE,
    },
    TdLayerMetadata {
        id: "ddt",
        label: "DDT",
        hp: 152,
        move_speed: 260,
        children: &[],
        cash: 152,
        leak_value: 152,
        properties: layer_property::MOAB_CLASS | layer_property::CAMO,
        accepted_damage: WITHOUT_SHARP,
    },
    TdLayerMetadata {
        id: "bad",
        label: "BAD",
        hp: 67200,
        move_speed: 35,
        children: &[],
        cash: 67200,
        leak_value: 67200,
        properties: layer_property::MOAB_CLASS,
        accepted_damage: ALL_DAMAGE,
    },
];

pub fn layer(base: &str) -> Option<&'static TdLayerMetadata> {
    TD_LAYER_CATALOG.iter().find(|entry| entry.id == base)
}

pub fn validate_layer_catalog() -> Result<(), String> {
    use std::collections::BTreeSet;

    let ids: BTreeSet<&str> = TD_LAYER_CATALOG.iter().map(|entry| entry.id).collect();
    if ids.len() != TD_LAYER_CATALOG.len() {
        return Err("TD layer catalog contains duplicate ids".into());
    }
    for entry in TD_LAYER_CATALOG {
        if entry.hp == 0 {
            return Err(format!("TD layer '{}' hp must be positive", entry.id));
        }
        if entry.accepted_damage == 0 || entry.accepted_damage & !damage_profile::KNOWN != 0 {
            return Err(format!(
                "TD layer '{}' has invalid accepted_damage mask",
                entry.id
            ));
        }
        for child in entry.children {
            if !ids.contains(child) {
                return Err(format!(
                    "TD layer '{}' children references unknown layer '{}'",
                    entry.id, child
                ));
            }
        }
    }

    fn visit(
        id: &'static str,
        visiting: &mut Vec<&'static str>,
        done: &mut BTreeSet<&'static str>,
    ) -> Result<(), String> {
        if let Some(start) = visiting.iter().position(|candidate| *candidate == id) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(id);
            return Err(format!("TD layer cycle: {}", cycle.join(" -> ")));
        }
        if done.contains(id) {
            return Ok(());
        }
        visiting.push(id);
        let entry = layer(id).ok_or_else(|| format!("TD layer '{}' is missing", id))?;
        for child in entry.children {
            visit(child, visiting, done)?;
        }
        visiting.pop();
        done.insert(id);
        Ok(())
    }

    let mut done = BTreeSet::new();
    for entry in TD_LAYER_CATALOG {
        visit(entry.id, &mut Vec::new(), &mut done)?;
    }
    Ok(())
}

const ROUND_DESCRIPTIONS: [&str; 100] = [
    "20 Reds",
    "35 Reds",
    "25 Reds, 5 Blues",
    "35 Reds, 18 Blues",
    "5 Reds, 27 Blues",
    "15 Reds, 15 Blues, 4 Greens",
    "20 Reds, 20 Blues, 5 Greens",
    "10 Reds, 20 Blues, 14 Greens",
    "30 Greens",
    "102 Blues",
    "10 Reds, 10 Blues, 12 Greens, 3 Yellows",
    "15 Blues, 10 Greens, 5 Yellows",
    "50 Blues, 23 Greens",
    "49 Reds, 15 Blues, 10 Greens, 9 Yellows",
    "20 Reds, 15 Blues, 12 Greens, 10 Yellows, 5 Pinks",
    "40 Greens, 8 Yellows",
    "12 Regrow Yellows",
    "80 Greens",
    "10 Greens, 4 Yellows, 5 Regrow Yellows, 15 Pinks",
    "6 Blacks",
    "40 Yellows, 14 Pinks",
    "16 Whites",
    "7 Blacks, 7 Whites",
    "20 Blues, Camo Green",
    "25 Regrow Yellows, 10 Purples",
    "23 Pinks, 4 Zebras",
    "100 Reds, 60 Blues, 45 Greens, 45 Yellows",
    "6 Leads",
    "50 Yellows, 15 Regrow Yellows",
    "9 Leads",
    "8 Blacks, 8 Whites, 8 Zebras, 2 Regrow Zebras",
    "15 Blacks, 20 Whites, 10 Purples",
    "20 Camo Reds, 13 Camo Yellows",
    "160 Yellows, 6 Zebras",
    "35 Pinks, 30 Blacks, 25 Whites, 5 Rainbows",
    "140 Pinks, 20 Camo Regrow Greens",
    "25 Blacks, 25 Whites, 7 Camo Whites, 10 Zebras, 15 Leads",
    "42 Pinks, 17 Whites, 10 Zebras, 14 Leads, 2 Ceramics",
    "10 Blacks, 10 Whites, 20 Zebras, 18 Rainbows, 2 Regrow Rainbows",
    "MOAB",
    "60 Blacks, 60 Zebras",
    "6 Regrow Rainbows, 5 Camo Rainbows",
    "10 Rainbows, 7 Ceramics",
    "50 Zebras",
    "180 Pinks, 10 Camo Purples, 4 Fortified Leads, 25 Rainbows",
    "6 Fortified Ceramics",
    "70 Camo Pinks, 12 Ceramics",
    "40 Regrow Pinks, 30 Camo Regrow Purples, 40 Rainbows, 3 Fortified Ceramics",
    "343 Greens, 20 Zebras, 20 Rainbows, 10 Regrow Rainbows, 18 Ceramics",
    "20 Reds, 8 Fortified Leads, 20 Ceramics, 2 MOABs",
    "10 Regrow Rainbows, 15 Camo Ceramics",
    "25 Rainbows, 10 Ceramics, 2 MOABs",
    "80 Camo Pinks, 3 MOABs",
    "35 Ceramics, 2 MOABs",
    "45 Ceramics, MOAB",
    "40 Camo Rainbows, MOAB",
    "40 Rainbows, 4 MOABs",
    "15 Ceramics, 10 Fortified Ceramics, 5 MOABs",
    "50 Camo Leads, 20 Ceramics, 10 Regrow Ceramics",
    "BFB",
    "150 Regrow Zebras, 5 MOABs",
    "250 Purples, 15 Camo Regrow Rainbows, 5 MOABs, 2 Fortified MOABs",
    "75 Leads, 122 Ceramics",
    "6 MOABs, 3 Fortified MOABs",
    "100 Zebras, 70 Rainbows, 50 Ceramics, 3 MOABs, 2 BFBs",
    "8 MOABs, 3 Fortified MOABs",
    "13 Camo Regrow Fortified Ceramics, 8 MOABs",
    "4 MOABs, BFB",
    "40 Regrow Blacks, 40 Fortified Leads, 50 Ceramics",
    "120 Camo Regrow Whites, 200 Rainbows, 4 MOABs",
    "30 Ceramics, 10 MOABs",
    "38 Regrow Ceramics, 2 BFBs",
    "8 MOABs, 2 BFBs",
    "50 Ceramics, 60 Fortified Ceramics, 25 Camo Regrow Fortified Ceramics, BFB",
    "14 Leads, 14 Fortified Leads, 3 Fortified MOABs, 7 BFBs",
    "60 Regrow Ceramics",
    "11 MOABs, 5 BFBs",
    "80 Purples, 150 Rainbows, 75 Ceramics, 72 Camo Ceramics, BFB",
    "500 Regrow Rainbows, 4 BFBs, 2 Fortified BFBs",
    "ZOMG",
    "17 BFBs",
    "10 BFBs, 5 Fortified BFBs",
    "40 Ceramics, 40 Regrow Ceramics, 40 Fortified Ceramics, 30 MOABs",
    "50 MOABs, 10 BFBs",
    "2 ZOMGs",
    "5 Fortified BFBs",
    "4 ZOMGs",
    "18 MOABs, 8 BFBs, 2 ZOMGs",
    "20 Fortified MOABs, 8 Fortified BFBs",
    "50 Camo Regrow Fortified Leads, 3 DDTs",
    "100 Fortified Ceramics, 20 BFBs",
    "50 Fortified MOABs, 4 ZOMGs",
    "10 Fortified BFBs, 6 DDTs",
    "25 BFBs, 6 ZOMGs",
    "500 Camo Regrow Purples, 250 Camo Regrow Fortified Leads, 50 Fortified MOABs, 30 DDTs",
    "40 Fortified MOABs, 30 BFBs, 6 ZOMGs",
    "2 Fortified ZOMGs",
    "30 Fortified BFBs, 8 ZOMGs",
    "60 MOABs, 9 Fortified DDTs",
    "BAD",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalloonSpec {
    pub id: String,
    pub label: String,
    pub base: &'static str,
    /// Authoritative current-layer HP used to initialize runtime `CProperty`.
    pub hp: u32,
    /// Read-only graph HP exposed to authored `SelectSpawnPath` callbacks.
    /// This compatibility value is never used to construct or flatten a creep.
    pub selector_hp: u32,
    pub camo: bool,
    pub regrow: bool,
    pub fortified: bool,
}

pub fn round_count() -> usize {
    ROUND_DESCRIPTIONS.len()
}

pub fn round(round_index: usize) -> Vec<BalloonSpec> {
    grouped_round(round_index)
        .into_iter()
        .flat_map(|(count, spec)| std::iter::repeat_n(spec, count))
        .collect()
}

pub fn grouped_round(round_index: usize) -> Vec<(usize, BalloonSpec)> {
    let Some(description) = ROUND_DESCRIPTIONS.get(round_index) else {
        return Vec::new();
    };
    description.split(',').filter_map(parse_part).collect()
}

fn parse_part(part: &str) -> Option<(usize, BalloonSpec)> {
    let cleaned = part
        .split('(')
        .next()
        .unwrap_or(part)
        .replace('\u{2002}', " ");
    let mut count = 1usize;
    let mut words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    if let Ok(parsed) = words[0].parse::<usize>() {
        count = parsed;
        words.remove(0);
    }
    let camo = words.iter().any(|word| word.eq_ignore_ascii_case("camo"));
    let regrow = words.iter().any(|word| word.eq_ignore_ascii_case("regrow"));
    let fortified = words
        .iter()
        .any(|word| word.eq_ignore_ascii_case("fortified"));
    let base = words.iter().rev().find_map(|word| normalize_base(word))?;
    let layer_hp = layer(base)?.hp;
    let hp = if fortified {
        layer_hp.saturating_mul(2)
    } else {
        layer_hp
    };
    let selector_hp = selector_graph_hp(base, fortified)?;
    let id = creep_key(base, camo, regrow, fortified);
    let mut label_parts = Vec::new();
    if camo {
        label_parts.push("Camo");
    }
    if regrow {
        label_parts.push("Regrow");
    }
    if fortified {
        label_parts.push("Fortified");
    }
    label_parts.push(match base {
        "moab" => "MOAB",
        "bfb" => "BFB",
        "zomg" => "ZOMG",
        "ddt" => "DDT",
        "bad" => "BAD",
        other => other,
    });
    Some((
        count,
        BalloonSpec {
            id,
            label: label_parts.join(" "),
            base,
            hp,
            selector_hp,
            camo,
            regrow,
            fortified,
        },
    ))
}

fn selector_graph_hp(base: &str, fortified: bool) -> Option<u32> {
    let metadata = layer(base)?;
    let multiplier = if fortified { 2 } else { 1 };
    let mut total = metadata.hp.saturating_mul(multiplier);
    for child in metadata.children {
        total = total.saturating_add(selector_graph_hp(child, fortified)?);
    }
    Some(total)
}

fn creep_key(base: &str, camo: bool, regrow: bool, fortified: bool) -> String {
    let mut key = String::from("td_btd");
    if camo {
        key.push_str("_camo");
    }
    if regrow {
        key.push_str("_regrow");
    }
    if fortified {
        key.push_str("_fortified");
    }
    key.push('_');
    key.push_str(base);
    key
}

fn normalize_base(token: &str) -> Option<&'static str> {
    match token.trim().to_ascii_lowercase().as_str() {
        "red" | "reds" => Some("red"),
        "blue" | "blues" => Some("blue"),
        "green" | "greens" => Some("green"),
        "yellow" | "yellows" => Some("yellow"),
        "pink" | "pinks" => Some("pink"),
        "black" | "blacks" => Some("black"),
        "white" | "whites" => Some("white"),
        "purple" | "purples" => Some("purple"),
        "zebra" | "zebras" => Some("zebra"),
        "lead" | "leads" => Some("lead"),
        "rainbow" | "rainbows" => Some("rainbow"),
        "ceramic" | "ceramics" => Some("ceramic"),
        "moab" | "moabs" => Some("moab"),
        "bfb" | "bfbs" => Some("bfb"),
        "zomg" | "zomgs" => Some("zomg"),
        "ddt" | "ddts" => Some("ddt"),
        "bad" | "bads" => Some("bad"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_grouped_round_in_original_order() {
        let balloons = round(2);
        assert_eq!(balloons.len(), 30);
        assert!(balloons[..25].iter().all(|balloon| balloon.base == "red"));
        assert!(balloons[25..].iter().all(|balloon| balloon.base == "blue"));
    }

    #[test]
    fn exposes_variant_flags_and_label() {
        let balloons = round(16);
        assert_eq!(balloons.len(), 12);
        assert!(balloons.iter().all(|balloon| balloon.regrow));
        assert!(balloons
            .iter()
            .all(|balloon| balloon.label == "Regrow yellow"));
    }

    #[test]
    fn layer_catalog_is_valid_and_covers_every_round_reference() {
        validate_layer_catalog().unwrap();
        for round_index in 0..round_count() {
            for balloon in round(round_index) {
                assert!(
                    layer(balloon.base).is_some(),
                    "round {} missing {}",
                    round_index + 1,
                    balloon.base
                );
            }
        }
    }

    #[test]
    fn representative_layer_graphs_have_expected_leak_value() {
        for (id, hp) in [
            ("black", 11),
            ("zebra", 23),
            ("ceramic", 104),
            ("moab", 616),
            ("bfb", 3164),
            ("zomg", 16656),
        ] {
            assert_eq!(layer(id).unwrap().leak_value, hp, "{id}");
        }
    }

    #[test]
    fn spawn_path_selector_hp_is_derived_from_the_layer_graph() {
        assert_eq!(selector_graph_hp("red", false), Some(1));
        assert_eq!(selector_graph_hp("blue", false), Some(2));
        assert_eq!(selector_graph_hp("ceramic", false), Some(104));
        assert_eq!(selector_graph_hp("ceramic", true), Some(208));

        let blue = round(2).pop().expect("round 3 has blue balloons");
        assert_eq!(blue.base, "blue");
        assert_eq!(blue.hp, 1, "runtime receives current-layer HP");
        assert_eq!(blue.selector_hp, 2, "map selector receives graph HP");
    }
}
