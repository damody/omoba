-- Heroes built as their own script DLL (M3' ①, docs/plans/2026-10-28-history-nobu-full-port/
-- m3-rust-codegen-design.md §3). Every hero here leaves base_content's generated registry
-- (gen_hero_registry.lua) and is registered by its own crate scripts/<dir> instead, which
-- builds <package>.dll. Launches stage base_content plus these DLLs (run_artifacts.lua).
--   hero     content hero id (templates/heroes.lua), also the hero's unit id
--   package  cargo package and DLL name
--   dir      crate directory under scripts/
return {
  {hero='inahime',package='hero_a02',dir='heroes/inahime'},
  {hero='saika_magoichi',package='hero_b01',dir='heroes/saika_magoichi'},
  {hero='oichi',package='hero_c17',dir='heroes/oichi'},
  -- M5 (2026-10-30): A34 every ability generated, conformance 0 regressions (A34T compare)
  {hero='omatsu',package='hero_a34',dir='heroes/omatsu'},
  -- M5 (2026-10-30): B08 every ability generated (reveal.ignore_terrain through line B's reveal_ex)
  {hero='asai_nagamasa',package='hero_b08',dir='heroes/asai_nagamasa'},
  -- M5 (2026-10-30): B29 every ability generated (aura, channel start, {rank} runtime ids), conformance 0 regressions
  {hero='murakami_yoshikiyo',package='hero_b29',dir='heroes/murakami_yoshikiyo'},
}
