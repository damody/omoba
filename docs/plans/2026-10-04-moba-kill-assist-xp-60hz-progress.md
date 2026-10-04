# 擊殺／助攻 XP 60Hz 增量

## 計畫與決策

1. 先補既有權威kill／assist結算缺少的XP，不讓Unreal自行累加。
2. Lua定義hero_kill_xp=100／hero_assist_xp=50，生成Rust常數並納入compiled agreement與完整內容hash；更改數值須重建所有peers，拒絕dev hot reload。
3. 維持同一權威正傷害／唯一生命去重，不以ACK或公開KDA推算玩法獎勵；沿既有CommittedProgression／EquipmentStats安全投影與通用HUD輸出。
4. 回歸全部單路生命週期，實際60Hz三玩家KCP，重建bridge與OmGame；問題記E116。

## 實作

- SingleLaneConfig明確XP獎勵、<=1,000,000驗證與replay digest；預設使用generated Lua值。
- 活體killer／assistant寫ECS Hero；死亡assistant寫持久slot.hero，重生恢复。已退休victim不能再發獎勵，同生命Heal／再致死沿既有lethal_pending拒絕重付。
- MOBA獨立add_moba_experience：非負／飽和、連續升級、25級上限清餘XP；門檻floor(100×6^(level-1)/5^(level-1))以u128整數計算，不用浮點數。每級一skill point，技能本身不自動升級。旧TD add_experience／level_up不改。
- 活體升級只增加level growth的HP上限與攻擊delta，不重算／清除已套用裝備、不補HP或復活；死亡助攻者新生命由既有spawn規則建立成長數值。
- 原通用效果用entity存在當存活，HP0但pending-deletion可queued cast成功並啟動CD，而filtered已Forget該NPC。新增get_hp>0 gate，保留owner冷卻重演、不加入ComponentRepair，補真實adapter回歸。
- Rust capture verifier新增combat.xp_contract=1：Playing每live owner逐tick對原始CommittedProgression；所有live樣本對score及Lua XP總獎勵核對，要求實際killer／assistant XP樣本。Warmup不冒充active結算，死亡無Hero時不虛構零XP。

## 驗證

- core332／base_content86全數通過；base含15／60／120Hz完整filtered生命周期零repair／hash一致、死亡assistant XP重生保留、duplicate lethal、inactive／self／NPC拒絕、成長不補血及零HP queued cast。
- template runtime-lua-content完整suite64通過（32 lib＋23 hero abilities＋8 numeric registry＋1 ID registry；含新hash／hot reload／完整型別負向回歸）。
- server154／1 ignored、runtime59／6外部capture ignored、bridge51／1外部capture ignored；Fyrox check --tests通過。
- 正式KCP `target/interactive-runs/moba-runtime-1791081719` success／cleanup_verified true：player3 kill1、player1 assist1、player2 death1，原AttackTarget ID2各套用一次；不注入傷害／XP／Gold，也不是Unreal或真人操作。
- 原始wire／IPC verifier另行執行成功：1389＋1279＋1255共3,923筆snapshots。owner XP／level／SP與Playing原始progression精確對照，公開board／私人score及income＋reward也逐tick一致。player3擊殺升2級／餘XP0／SP1，player1助攻仍1級／XP50／SP0；fixture沒有兵線經驗，所以可精確核對唯一reward。
- 雙隊各10 checkpoints、player3獨立9個至1200，各至少2個post-kill checkpoints；保存IPC延伸至1567／1561／1556。這是離散hash檢查，不宣稱hard realtime deadline。
- server92656／runtime90048／51760／81956獨立Lua inspect確認退出。
- 最末Unreal build-only／staging／codegen --check均成功：generated C++11檔／15Lua輸入content hash仍de9c7fcfc98d6479、既有ABI9不改，bridge SHA-256 `d918c4533c5ee7e0d1fd93a350746c13ab1b9ccb80eca85d274eed92f0ba5188`與staged相同。XP玩法agreement用完整templates rules hash，不把UE英雄資料hash當XP規則hash。
- OpenSpec strict validation與scoped diff whitespace檢查通過；失敗及修正全部記E116，沒有未完成測試程序或提交／推送。

## 限制與下一步

這不是完整5.3。兵線目前仍有舊Bounty／distribute_bounty經驗路徑，需要統一XP規則、範圍／分享／零HP與去重語意；本輪未把它算作完成，也未改既有TD經驗。後續優先兵線XP與正式skill upgrade capability／IPC／Unreal操作，不能只因HUD已顯示SP就宣稱技能升級可用。未重跑本輪Unreal實戰XP畫面／Editor測試，不引用上輪畫面當新增證據。OpenSpec維持19/30。
