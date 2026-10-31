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
  -- M6 third batch (2026-10-31): every ability generated, conformance 0 fail on live (harness efdc4728)
  {hero='kuroda_yoshitaka',package='hero_a19',dir='heroes/kuroda_yoshitaka'},
  {hero='sakuma_morimasa',package='hero_a24',dir='heroes/sakuma_morimasa'},
  {hero='koihime',package='hero_b18',dir='heroes/koihime'},
  {hero='usami_sadamitsu',package='hero_b19',dir='heroes/usami_sadamitsu'},
  {hero='uesugi_kenshin',package='hero_b32',dir='heroes/uesugi_kenshin'},
  {hero='akechi_mitsuhide',package='hero_c01',dir='heroes/akechi_mitsuhide'},
  {hero='akechi_hidemitsu',package='hero_c02',dir='heroes/akechi_hidemitsu'},
  {hero='miyamoto_musashi',package='hero_c21',dir='heroes/miyamoto_musashi'},
  {hero='sasaki_kojiro',package='hero_c22',dir='heroes/sasaki_kojiro'},
  -- M6 fourth batch (2026-10-31): every ability generated, conformance 0 fail on live d47398f0
  {hero='miyoshi_nagayoshi',package='hero_b10',dir='heroes/miyoshi_nagayoshi'},
  {hero='momochi_sandayu',package='hero_b13',dir='heroes/momochi_sandayu'},
  -- M6 fifth batch (2026-10-31): every ability generated, conformance 0 fail
  {hero='saito_tomonobu',package='hero_b03',dir='heroes/saito_tomonobu'},
  {hero='naoe_kanetsugu',package='hero_b36',dir='heroes/naoe_kanetsugu'},
  {hero='mogami_yoshihime',package='hero_b33',dir='heroes/mogami_yoshihime'},
  {hero='taigen_sessai',package='hero_b22',dir='heroes/taigen_sessai'},
  {hero='yamamoto_kansuke',package='hero_b12',dir='heroes/yamamoto_kansuke'},
  {hero='okuma_tomohide',package='hero_b26',dir='heroes/okuma_tomohide'},
  {hero='tachibana_dosetsu',package='hero_c07',dir='heroes/tachibana_dosetsu'},
  {hero='katakura_kagetsuna',package='hero_b14',dir='heroes/katakura_kagetsuna'},
  {hero='hirate_masahide',package='hero_a10',dir='heroes/hirate_masahide'},
  {hero='takeda_katsuyori',package='hero_b34',dir='heroes/takeda_katsuyori'},
  {hero='kuki_yoshitaka',package='hero_a20',dir='heroes/kuki_yoshitaka'},
  -- M6 batch (2026-10-31): every ability generated, conformance 0 fail on live a9022d2d
  {hero='ishikawa_kazumasa',package='hero_a03',dir='heroes/ishikawa_kazumasa'},
  {hero='ishida_mitsunari',package='hero_a29',dir='heroes/ishida_mitsunari'},
  {hero='niwa_nagahide',package='hero_a30',dir='heroes/niwa_nagahide'},
  {hero='honda_masanobu',package='hero_a33',dir='heroes/honda_masanobu'},
  {hero='takeda_shingen',package='hero_b31',dir='heroes/takeda_shingen'},
  {hero='izumo_no_okuni',package='hero_c04',dir='heroes/izumo_no_okuni'},
  -- M6 batch (2026-10-31): every ability generated, conformance 0 fail (full480 70526876; A11 A27 shadow)
  {hero='saito_dosan',package='hero_a11',dir='heroes/saito_dosan'},
  {hero='shibata_katsuie',package='hero_a27',dir='heroes/shibata_katsuie'},
  {hero='fuma_kotaro',package='hero_b02',dir='heroes/fuma_kotaro'},
  {hero='sanada_masayuki',package='hero_b09',dir='heroes/sanada_masayuki'},
  {hero='baba_nobufusa',package='hero_b21',dir='heroes/baba_nobufusa'},
  {hero='kato_kiyomasa',package='hero_c23',dir='heroes/kato_kiyomasa'},
  -- M6 batch (2026-10-31): follow, on_refreshed, knockback on_end and rank-up re-put generated; conformance 0 fail (shadow)
  {hero='takenaka_shigeharu',package='hero_a04',dir='heroes/takenaka_shigeharu'},
  {hero='maeda_toshiie',package='hero_a14',dir='heroes/maeda_toshiie'},
  -- M6 batch (2026-10-31): ability_in_slot generated; conformance 0 fail (shadow)
  {hero='chacha',package='hero_c25',dir='heroes/chacha'},
}
