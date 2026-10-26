# 玉子：驗收

status: implemented

kit 狀態：`needs_framework`。只能用薄切片建置：

```bat
tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\tamako --slice c15_scatter_shot,c15_precise_light
```

切片中 Q 在第 1 欄、E 往前補到第 2 欄。W、R 的缺口見 `assumptions.md`。`catalog_ready` 維持 `false`。

| 技能 | 狀態 | 實作 | L1（shadow 測試） |
|---|---|---|---|
| `c15_scatter_shot` Q 亂射 | supported | `No1_scatter_shot`：落點 275 內每個可見敵人各一顆 G-1 Homing 箭，命中時魔法傷害＋Stun；最多 32 支 | 6 個單元測試＋parity soak |
| `c15_burst_arrow` W 爆裂箭 | needs_framework | 缺 `ground_area_line`；未寫 handler | — |
| `c15_precise_light` E 精準之光 | supported | `No3_precise_light`：on_learn 永久 buff（攻擊 % 與靈活換算），升級取代 | 2 個單元測試 |
| `c15_hundred_arrows` R 百箭齊發 | needs_framework | 缺 `retarget_on_death`、`summon_forced_target`、`summon_invulnerable`；未寫 handler | — |

## L1 指令（作者私人樹）

```bat
tools\lua\lua.exe scripts\hero_author.lua shadow docs\character_pipeline\packages\tamako --tree D:\omoba\target\blind-c15\tree --slice c15_scatter_shot,c15_precise_light
set CARGO_TARGET_DIR=D:\omoba\target\blind-c15\cargo
cargo test --manifest-path D:\omoba\target\blind-c15\tree\scripts\Cargo.toml -p base_content --lib -- heroes::C15_tamako
cargo test --release --manifest-path D:\omoba\target\blind-c15\tree\scripts\Cargo.toml -p base_content --lib -- tamako_kit_keeps
```

parity soak 寫在 shadow 樹的 `scripts/base_content/src/single_lane_match_tests.rs`，位置是 `mod live_soak_replica_parity`。package 沒有放這個檔案的位置，整合時要由整合者照著陸流程合併：

```rust
#[test]
fn tamako_kit_keeps_team_replicas_identical() {
    // Spawn points are a base apart, so the enemy hero starts 450 in front of
    // tamako and Q is aimed at its live position (and at a point offset into
    // the lane, which also catches creeps or nothing).
    let spawns=std::cell::Cell::new(None);
    let mismatches=soak_match(SingleLaneConfig {mana_enabled:true,heroes:["tamako".into(),"training_luminary".into()],..three_lane_config()},7200,1,
        &|authority|{
            let pair=hero_pair(authority);
            for hero in pair {
                authority.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
                let mut heroes=authority.write_storage::<Hero>();let h=heroes.get_mut(hero).unwrap();h.level=12;
                for id in h.abilities.clone() {h.ability_levels.insert(id,1);}
            }
            let mut pos=authority.write_storage::<Pos>();
            let at=pos.get(pair[0]).unwrap().0+omoba_sim::Vec2::new(Fixed64::from_i32(450),Fixed64::ZERO);
            pos.get_mut(pair[1]).unwrap().0=at;
        },
        &mut |authority,tick|{
            let state=authority.read_resource::<MobaMatch>();
            let (Some(first),Some(second))=(state.heroes[0].entity,state.heroes[1].entity) else {return Vec::new();};
            let positions=authority.read_storage::<Pos>();
            let (Some(a),Some(b))=(positions.get(first).map(|p|p.0),positions.get(second).map(|p|p.0)) else {return Vec::new();};
            if spawns.get().is_none() {spawns.set(Some((a,b)));}
            let (home,away)=spawns.get().unwrap();
            match tick {
                t if t%600==6=>vec![(1,attack_move(away)),(2,attack_move(home))],
                t if t%600==300=>vec![(1,attack_move(home)),(2,attack_move(away))],
                t if t>6 && t%120==10=>vec![(1,cast_at(0,b)),(2,cast(0,Some(first)))],
                t if t>6 && t%120==50=>vec![(1,cast_at(0,a+omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::from_i32(100))))],
                t if t>6 && t%120==90=>vec![(1,cast(1,None)),(1,attack_move(b))],
                _=>vec![],
            }
        });
    assert!(mismatches.is_empty(),"{:#?}",mismatches);
}
```

L1 結果（2026-10-26，shadow 樹）：
- `heroes::C15_tamako`：8 passed。
- `tamako_kit_keeps_team_replicas_identical`（release、7200 tick、每 tick 比對）：1 passed。
- 另外用暫時的 eprintln 探針確認過，soak 內確實有多支箭成功發射（一次 3 支、一次 1 支）。探針已移除。

## 實機觀察點（quick 地圖，10 分鐘內）

1. **Q 亂射**
   - 對 3～5 隻小兵聚集處施放：每隻小兵各飛來一支箭，遠的較晚命中。命中時才扣 75 血（rank 1）並出現暈眩圖示約 0.6 秒。
   - 範圍外 1 個身位的小兵、我方單位、敵方塔都不會被鎖定。
   - 對空地施放：照常扣 140 魔、進 18 秒冷卻，沒有箭。
   - 對迷霧中的敵人施放：不會飛出箭。
   - 落點超過 700：被拒絕，不扣魔。
2. **E 精準之光**
   - 學 rank 1 後，面板攻擊力約為 (50＋5)×1.10、攻速點 +5、護甲 +0.83。
   - 升到 rank 2：數值換成 (50＋10)×1.18，不疊加。
   - 死亡重生後仍保留。
   - 不能主動按。
3. **W／R**：切片建置不含這兩招，技能列不應出現。缺口補上後再補驗：
   - W：火焰沿路徑長 2000、半寬 170、持續 12 秒，每秒 100～160 傷害，重疊不疊加。
   - R：暈眩加 10 支無敵箭矢，目標死亡後改打附近敵人；被動每 8 擊一支。
