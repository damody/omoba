# Design

UnitHex optionally carries a closed stun/root/silence kind and a positive fixed-point duration up to 60 seconds. Content validation, generated Rust constants, runtime configuration and generated native shop metadata preserve the same policy. Existing items omit the optional control.

Target admission retains visibility, enemy allegiance, range, life and cooldown checks. Control cannot target buildings. Preflight completes before damage, status or cooldown writes; only surviving targets receive canonical control buffs. Existing BuffStore refresh, pause, expiration, death and snapshot rules apply. The UI uses typed metadata, rejects invalid control policy and describes duration and building exclusion.

An appended item (13, moba_binding_charm) deals 30 direct damage, roots 1.5 seconds, range 450, cooldown 18 seconds, cost 650. Tests cover real catalog purchase/use, rejection atomicity, movement/expiry/pause, snapshot, schema and native metadata/presentation. No C ABI change or special case by item/hero/camp name.
