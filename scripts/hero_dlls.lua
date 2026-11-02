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
  -- M6 batch (2026-10-31): item_tag and prd chance generated; conformance 0 fail (shadow)
  {hero='ryuzoji_takanobu',package='hero_c13',dir='heroes/ryuzoji_takanobu'},
  {hero='sassa_narimasa',package='hero_a17',dir='heroes/sassa_narimasa'},
  -- M6 batch (2026-10-31): independent instance vars and remove_this_instance generated; conformance 0 fail (shadow)
  {hero='ii_naomasa',package='hero_a06',dir='heroes/ii_naomasa'},
  -- M6 batch (2026-10-31): every ability generated; conformance 0 fail (live 27b3faf2)
  {hero='mori_ranmaru',package='hero_a31',dir='heroes/mori_ranmaru'},
  {hero='yamagata_masakage',package='hero_b20',dir='heroes/yamagata_masakage'},
  {hero='shimazu_yoshihiro',package='hero_c03',dir='heroes/shimazu_yoshihiro'},
  {hero='otomo_sorin',package='hero_c12',dir='heroes/otomo_sorin'},
  {hero='matsuhime',package='hero_c19',dir='heroes/matsuhime'},
  -- M6 batch (2026-10-31): every ability generated; conformance 0 fail (shadow, independent-instance vars)
  {hero='mori_motonari',package='hero_b05',dir='heroes/mori_motonari'},
  -- M6 batch (2026-10-31): every ability generated; conformance 0 fail (C09 live a2f4a0fa; C16 shadow, no_attack)
  {hero='tachibana_ginchiyo',package='hero_c09',dir='heroes/tachibana_ginchiyo'},
  {hero='kashin_koji',package='hero_c16',dir='heroes/kashin_koji'},
  -- M6 batch (2026-10-31): every ability generated; stat.when and impulse generated; conformance 0 fail (shadow)
  {hero='kosokabe_chikayasu',package='hero_c10',dir='heroes/kosokabe_chikayasu'},
  {hero='honganji_kennyo',package='hero_b25',dir='heroes/honganji_kennyo'},
  -- M6 batch (2026-10-31): every ability generated (modify_base_stat E523); conformance 0 fail
  {hero='kikuhime',package='hero_b11',dir='heroes/kikuhime'},
  -- M6 batch (2026-10-31): every ability generated (模板 passive.triggers 合成狀態); conformance 0 fail
  {hero='hachisuka_masakatsu',package='hero_a09',dir='heroes/hachisuka_masakatsu'},
  -- M6 batch (2026-10-31): every ability generated (模板 abilities 以 grant_unit_ability 授予); conformance 0 fail
  {hero='maeno_nagayasu',package='hero_a36',dir='heroes/maeno_nagayasu'},
  -- M6 batch (2026-10-31): every ability generated (chain 每道彈跳各自命中表); conformance 0 fail
  {hero='matsunaga_hisahide',package='hero_b28',dir='heroes/matsunaga_hisahide'},
  -- M6 batch (2026-10-31): every ability generated (法球槽 orb_slot（E534）); conformance 0 fail
  {hero='imagawa_yoshimoto',package='hero_b15',dir='heroes/imagawa_yoshimoto'},
  -- M6 batch (2026-10-31): every ability generated (受傷前 routine 在呼叫端展開；heal 檢查 E536 修正後全數通過); conformance 0 fail
  {hero='sakakibara_yasumasa',package='hero_a08',dir='heroes/sakakibara_yasumasa'},
  -- M6 batch (2026-10-31): every ability generated (法球槽與 landmark_of（E534、E538）；珍珠圓盤近戰射程 0（E546）); conformance 0 fail
  {hero='zuiryuin_nisshu',package='hero_a21',dir='heroes/zuiryuin_nisshu'},
  -- M6 batch (2026-10-31): every ability generated (模板 abilities、青蛙 block_disabled；harness _r 名稱比對修正（E543）); conformance 0 fail
  {hero='hashiba_hideyoshi',package='hero_a28',dir='heroes/hashiba_hideyoshi'},
  -- M6 batch (2026-10-31): every ability generated (include_dead 用引擎屍體保留時間（E539）、on_moved 被動；harness 屍體前提（E547）); conformance 0 fail
  {hero='ise_hime',package='hero_b23',dir='heroes/ise_hime'},
  -- M6 batch (2026-10-31): every ability generated (attack.effect_multiplier（E562）、dodge_projectiles.of（E557）); conformance 0 fail
  {hero='hattori_hanzo',package='hero_a13',dir='heroes/hattori_hanzo'},
  -- M6 batch (2026-10-31): every ability generated (status_source（E570）、pick_item 與 modify_cooldown.item（E561、E566、E569）); conformance 0 fail
  {hero='kobayakawa_takakage',package='hero_c18',dir='heroes/kobayakawa_takakage'},
  -- M6 batch (2026-10-31): every ability generated (add_ability（E560）、set_base_max_hp（E563）、has_item、dodge_projectiles.of（E557）；harness 先召喚月（E568）); conformance 0 fail
  {hero='mochizuki_chiyome',package='hero_b16',dir='heroes/mochizuki_chiyome'},
  -- M6 batch (2026-10-31): every ability generated (選取器 segment、add_ability 指定格位與 slot_ref、remove_ability(slot_ref)、set_ability_active event.ability); conformance 0 fail
  {hero='kato_danzo',package='hero_c08',dir='heroes/kato_danzo'},
  -- M6 batch (2026-10-31): every ability generated (道具 pick／give／remove、實例變數與 item_name（E561、E571）、unit attr hero（E575）、英雄文件道具進入道具目錄); conformance 0 fail
  {hero='ishikawa_goemon',package='hero_c06',dir='heroes/ishikawa_goemon'},
  -- M6 batch (2026-10-31): every ability generated (aura exclude_invisible、光環的 independent 狀態每個來源一份、召喚物 tag ward（E530）); conformance 0 fail
  {hero='kirigakure_saizo',package='hero_b17',dir='heroes/kirigakure_saizo'},
  -- M6 batch (2026-10-31): every ability generated (召喚物背包：pick_item→give_item 給召喚物（E587）); conformance 0 fail
  {hero='ashikaga_yoshiaki',package='hero_b07',dir='heroes/ashikaga_yoshiaki'},
  -- M6 batch (2026-10-31): every ability generated (redirect_motion、只命中攻擊者的直線飛彈（E586）；host 放行生成單位的 SetPos、dummy 的狀態圖示投影（E590、E597）); conformance 0 fail
  {hero='akiyama_nobutomo',package='hero_b24',dir='heroes/akiyama_nobutomo'},
  -- A07: every ability generated (a07_strongest 1c655563, conformance 0 fail); handlers deleted
  {hero='honda_tadakatsu',package='hero_a07',dir='heroes/honda_tadakatsu'},
}
