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
  -- M6 (2026-10-30): v3-only heroes playable through scripts/v3_test_heroes.lua (`playable`: every
  -- ability generated, conformance 0 fail, a7719235)
  {hero='maeda_keiji',package='hero_c05',dir='heroes/maeda_keiji'},
  {hero='inaba_ittetsu',package='hero_a18',dir='heroes/inaba_ittetsu'},
  {hero='kakizaki_kageie',package='hero_b30',dir='heroes/kakizaki_kageie'},
  {hero='torii_mototada',package='hero_a16',dir='heroes/torii_mototada'},
  {hero='ukita_hideie',package='hero_a23',dir='heroes/ukita_hideie'},
  {hero='shima_sakon',package='hero_a35',dir='heroes/shima_sakon'},
  {hero='tokugawa_ieyasu',package='hero_a01',dir='heroes/tokugawa_ieyasu'},
  {hero='nohime',package='hero_a26',dir='heroes/nohime'},
  {hero='kojima_sadaoki',package='hero_b27',dir='heroes/kojima_sadaoki'},
  {hero='saito_yoshitatsu',package='hero_c14',dir='heroes/saito_yoshitatsu'},
  {hero='yagyu_muneyoshi',package='hero_c24',dir='heroes/yagyu_muneyoshi'},
  {hero='chosokabe_motochika',package='hero_c20',dir='heroes/chosokabe_motochika'},
  {hero='kamiizumi_nobutsuna',package='hero_b35',dir='heroes/kamiizumi_nobutsuna'},
  {hero='nene',package='hero_a15',dir='heroes/nene'},
  -- M6 second batch (2026-10-31): every ability generated, conformance 0 fail on live 6abaf6ce
  {hero='sakai_tadatsugu',package='hero_a05',dir='heroes/sakai_tadatsugu'},
  {hero='otani_yoshitsugu',package='hero_a12',dir='heroes/otani_yoshitsugu'},
  {hero='sengoku_hidehisa',package='hero_a22',dir='heroes/sengoku_hidehisa'},
  {hero='oda_nobunaga',package='hero_a25',dir='heroes/oda_nobunaga'},
  {hero='takigawa_kazumasu',package='hero_a32',dir='heroes/takigawa_kazumasu'},
  {hero='tachibana_muneshige',package='hero_c11',dir='heroes/tachibana_muneshige'},
}
