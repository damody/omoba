//! Build-time generated template ids.
//!
//! Source of truth: `omb/Story/templates.json`.
//! Design: `docs/plans/2026-04-25-template-id-codegen-design.md`.
//!
//! Each category (tower, hero, ability, buff, summon, creep, projectile_kind)
//! gets its own `#[repr(transparent)]` newtype wrapping `u16`. Id 0 is reserved
//! as UNSPECIFIED. Forward lookup (`*_by_name`) + reverse lookup (`*_id_str`,
//! `*_display`) are generated as match statements.

#![allow(clippy::too_many_lines)]

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
