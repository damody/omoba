//! Validate the build-time generated template id tables.
//!
//! These tests pin the wire contract: const values are part of the protocol —
//! changing them silently shifts ids on the wire and breaks client↔server.

use omoba_template_ids::*;

#[test]
fn tower_consts_sequential_from_one() {
    assert_eq!(TOWER_DART.0, 1);
    assert_eq!(TOWER_TACK.0, 2);
    assert_eq!(TOWER_BOMB.0, 3);
    assert_eq!(TOWER_ICE.0, 4);
}

#[test]
fn hero_consts_independent_namespace() {
    // Hero id 1 != Tower id 1 — separate u16 spaces per category.
    assert_eq!(HERO_SAIKA_MAGOICHI.0, 1);
    assert_eq!(HERO_DATE_MASAMUNE.0, 2);
}

#[test]
fn ability_buff_summon_creep_projectile_allocate() {
    assert_eq!(ABILITY_SNIPER_MODE.0, 1);
    assert_eq!(BUFF_STUN.0, 1);
    assert_eq!(BUFF_SLOW.0, 2);
    assert_eq!(SUMMON_SAIKA_GUNNER.0, 1);
    assert_eq!(CREEP_TRAINING_MAGE.0, 1);
    assert_eq!(PROJECTILE_TACK.0, 3);
}

#[test]
fn forward_lookup_by_name() {
    assert_eq!(tower_by_name("tower_tack"), Some(TOWER_TACK));
    assert_eq!(tower_by_name("nonexistent"), None);
    assert_eq!(hero_by_name("saika_magoichi"), Some(HERO_SAIKA_MAGOICHI));
    assert_eq!(ability_by_name("sniper_mode"), Some(ABILITY_SNIPER_MODE));
    assert_eq!(buff_by_name("stun"), Some(BUFF_STUN));
    assert_eq!(
        projectile_by_name("saika_shot"),
        Some(PROJECTILE_SAIKA_SHOT)
    );
}

#[test]
fn reverse_id_str_roundtrip() {
    for s in ["tower_dart", "tower_tack", "tower_bomb", "tower_ice"] {
        let id = tower_by_name(s).expect("known tower");
        assert_eq!(tower_id_str(id), s, "roundtrip fail: {}", s);
    }
    assert_eq!(tower_id_str(TowerId(0)), "");
}

#[test]
fn display_name_lookup() {
    assert_eq!(creep_display(CREEP_TRAINING_MAGE), "訓練法師");
    assert_eq!(hero_display(HERO_SAIKA_MAGOICHI), "雜賀孫市");
    assert_eq!(hero_title(HERO_SAIKA_MAGOICHI), "千里狙擊手");
    assert_eq!(tower_display(TOWER_TACK), "鐵釘射手");
}

#[test]
fn unspecified_id_zero() {
    assert_eq!(TowerId::UNSPECIFIED.0, 0);
    assert_eq!(HeroId::UNSPECIFIED.0, 0);
    assert_eq!(BuffId::UNSPECIFIED.0, 0);
    assert_eq!(ProjectileKindId::UNSPECIFIED.0, 0);
    assert_eq!(tower_display(TowerId::UNSPECIFIED), "");
    assert_eq!(creep_display(CreepId::UNSPECIFIED), "");
}

#[test]
fn projectile_kinds_no_display() {
    // Projectile kinds are visual kinds — no display_name fn generated, only id_str.
    assert_eq!(projectile_id_str(PROJECTILE_TACK), "tack");
    assert_eq!(projectile_id_str(PROJECTILE_BOMB_FRAG), "bomb_frag");
}

#[test]
fn td_stress_template_values_are_authoritative() {
    let id = creep_by_name("td_stress").expect("td_stress template exists");
    assert_eq!(id, CREEP_TD_STRESS);
    assert_eq!(id.0, 15);
    let stats = creep_stats(id).expect("td_stress has stats");
    assert_eq!(creep_display(id), "壓測怪");
    assert_eq!(stats.hp, Fixed64::from_i32(10_000));
    assert_eq!(stats.move_speed, Fixed64::from_i32(100));
}

#[test]
fn tower_dart_render_metadata_is_generated() {
    let render = tower_render_metadata(TOWER_DART).expect("tower_dart render metadata");
    assert_eq!(render.render_mode, TowerRenderModeC::BaseBarrel);
    assert_eq!(render.rotation_mode, TowerRotationModeC::Targeted);
    assert_eq!(render.barrel_layout, TowerBarrelLayoutC::Single);
    assert_eq!(render.base, "assets/towers/tower_dart_base.png");
    assert_eq!(render.barrel, "assets/towers/tower_dart_barrel.png");
    assert_eq!(render.visual_size, Fixed64::from_i32(180));
    let stats = tower_stats(TOWER_DART).expect("tower_dart stats");
    assert_eq!(stats.placement_radius, Fixed64::from_i32(90));
    assert!(render
        .barrel_frames
        .contains(&"assets/towers/tower_dart_barrel_frame_01.png"));
    assert_eq!(render.barrel_pivot.x, Fixed64::from_raw(512));
    assert_eq!(render.barrel_offset.y, Fixed64::from_i32(-6));
    assert_eq!(render.recoil.mode, TowerRecoilModeC::Directional);
    assert_eq!(render.recoil.distance, Fixed64::from_i32(6));
}

#[test]
fn tower_tack_render_metadata_has_fixed_radial_variants() {
    let render = tower_render_metadata(TOWER_TACK).expect("tower_tack render metadata");
    assert_eq!(render.rotation_mode, TowerRotationModeC::Fixed);
    assert_eq!(
        render.barrel_layout,
        TowerBarrelLayoutC::RadialCountVariants
    );
    assert_eq!(render.recoil.mode, TowerRecoilModeC::ScalePulse);
    let counts: Vec<u16> = render.barrel_variants.iter().map(|v| v.count).collect();
    assert_eq!(counts, vec![8, 12, 16]);
    assert_eq!(
        render.barrel_variants[0].image,
        "assets/towers/tower_tack_barrel_8.png"
    );
    assert_eq!(
        render.barrel_variants[1].image,
        "assets/towers/tower_tack_barrel_12.png"
    );
    assert_eq!(
        render.barrel_variants[2].image,
        "assets/towers/tower_tack_barrel_16.png"
    );
    assert!(render.barrel_variants[1]
        .frames
        .contains(&"assets/towers/tower_tack_barrel_12_frame_01.png"));
}

#[test]
fn tower_bomb_sizing_metadata_is_explicit() {
    let render = tower_render_metadata(TOWER_BOMB).expect("tower_bomb render metadata");
    assert_eq!(render.visual_size, Fixed64::from_i32(225));
    let stats = tower_stats(TOWER_BOMB).expect("tower_bomb stats");
    assert_eq!(stats.placement_radius, Fixed64::from_i32(96));
}

#[test]
fn attack_timing_weights_validate_with_integer_sum() {
    assert!(TOWER_DART_ATTACK_TIMING.is_valid());
    assert!(HERO_SAIKA_MAGOICHI_ATTACK_TIMING.is_valid());
    assert!(!AttackTimingConst {
        windup: 350,
        backswing: 450,
    }
    .is_valid());
}

#[test]
fn generated_td_stories_are_available_without_json_sources() {
    assert!(story_by_name("TD_1").is_some());
    assert!(story_by_name("TD_STRESS").is_some());
    assert!(story_ids().contains(&"TD_1"));
    assert!(story_ids().contains(&"TD_STRESS"));

    let root = workspace_root().join("scripts/lua_data");
    let mut json_files = Vec::new();
    collect_files_with_extension(&root, "json", &mut json_files);
    assert!(
        json_files.is_empty(),
        "shipped JSON sources remain: {json_files:?}"
    );
}

#[test]
fn generated_map_creep_references_resolve_and_are_slim() {
    let forbidden = [
        "Label",
        "HP",
        "DefendPhysic",
        "DefendMagic",
        "MoveSpeed",
        "damage",
        "attack_range",
        "enemy_type",
        "ai_type",
        "exp_reward",
        "gold_reward",
        "coins",
    ];

    for story_id in story_ids() {
        let story = story_by_name(story_id).expect("known generated story");
        let creeps = object_field(&story.map, "Creep")
            .and_then(StoryValueExt::as_array)
            .unwrap_or_else(|| panic!("story {story_id} map has no Creep[]"));
        for creep in creeps {
            let name = object_field(creep, "Name")
                .and_then(StoryValueExt::as_str)
                .unwrap_or_else(|| panic!("story {story_id} Creep[] entry missing Name"));
            let id = creep_by_name(name)
                .unwrap_or_else(|| panic!("story {story_id} creep '{name}' has no template"));
            assert!(
                creep_stats(id).is_some(),
                "story {story_id} creep '{name}' has no stats"
            );
            for field in forbidden {
                assert!(
                    object_field(creep, field).is_none(),
                    "story {story_id} creep '{name}' has forbidden field '{field}'"
                );
            }
        }
    }
}

#[test]
fn runtime_crates_do_not_depend_on_mlua() {
    let root = workspace_root();
    let mut manifests = Vec::new();
    collect_cargo_manifests(&root, &mut manifests);
    for manifest in manifests {
        if manifest
            .components()
            .any(|c| c.as_os_str() == "omoba-template-ids")
        {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).expect("read Cargo.toml");
        assert!(
            !text.contains("mlua"),
            "mlua dependency outside omoba-template-ids: {manifest:?}"
        );
    }
}

#[test]
fn runtime_and_tooling_do_not_reference_old_story_json_paths() {
    let root = workspace_root();
    let mut files = Vec::new();
    for extension in ["rs", "ps1", "bat"] {
        collect_files_with_extension(&root, extension, &mut files);
    }
    for file in files {
        if file.components().any(|c| c.as_os_str() == "target") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let old_omb_story = ["omb", "/", "Story"].concat();
        let old_story_slash = ["Story", "/"].concat();
        let old_templates_json = ["templates", ".", "json"].concat();
        let old_lua_data = ["scripts", "/", "lua_data", "/"].concat();
        let old_lua_data_win = ["scripts", "\\", "lua_data", "\\"].concat();
        assert!(
            !text.contains(&old_omb_story),
            "old omb story path reference: {file:?}"
        );
        assert!(
            !text.contains(&old_story_slash),
            "old Story path reference: {file:?}"
        );
        assert!(
            !text.contains(&old_templates_json),
            "old template JSON reference: {file:?}"
        );
        for old_name in ["entity.json", "ability.json", "mission.json", "map.json"] {
            assert!(
                !text.contains(&(old_lua_data.clone() + old_name))
                    && !text.contains(&(old_lua_data_win.clone() + old_name)),
                "old lua_data JSON reference: {file:?}"
            );
        }
    }
}

trait StoryValueExt {
    fn as_array(&self) -> Option<&'static [StoryValue]>;
    fn as_str(&self) -> Option<&'static str>;
}

impl StoryValueExt for StoryValue {
    fn as_array(&self) -> Option<&'static [StoryValue]> {
        match self {
            StoryValue::Array(values) => Some(values),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&'static str> {
        match self {
            StoryValue::String(value) => Some(value),
            _ => None,
        }
    }
}

fn object_field(value: &StoryValue, key: &str) -> Option<&'static StoryValue> {
    match value {
        StoryValue::Object(fields) => fields
            .iter()
            .find_map(|(field_key, field_value)| (*field_key == key).then_some(field_value)),
        _ => None,
    }
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("omoba-template-ids has workspace parent")
        .to_path_buf()
}

fn collect_files_with_extension(
    dir: &std::path::Path,
    extension: &str,
    out: &mut Vec<std::path::PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
}

fn collect_cargo_manifests(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let skip = [".git", "target", "graphify-out"];
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if skip.iter().any(|skip| name == *skip) {
                continue;
            }
            collect_cargo_manifests(&path, out);
        } else if name == "Cargo.toml" {
            out.push(path);
        }
    }
}
