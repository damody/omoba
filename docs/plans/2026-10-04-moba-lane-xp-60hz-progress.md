# 兵線經驗 60Hz 增量

## 計畫與決策

1. 接在權威 tracked creep 死亡結算，不讓 renderer／Unreal 累加經驗。
2. Lua 定義每兵 25 XP、範圍 1200；敵隊存活 roster 英雄包含邊界，等分整數 floor，餘數捨棄，不要求英雄最後一擊。
3. 與 kill／assist 共用 MOBA 整數多級成長、25 級上限、SP、HP 上限／攻擊 delta；不補血，不改 TD 升級。
4. 完整 filtered lifecycle、真實 KCP 60Hz、bridge／Unreal 建置與 staged DLL 核對；問題記 E117。

## 實作與限制

- `scripts/lua_data/templates/moba_economy.lua` → template-id generated 常數 → SingleLaneConfig。完整 canonical content hash、replay digest、compiled agreement／dev hot-reload 拒絕均包含新增兩個值。XP <=1,000,000；radius 1..10,000。
- 只在 Playing、非暫停、真實 HP<=0 的 tracked Creep 死亡處理；先移除 tracked unit，重送／重複死亡不再支付。範圍使用 Fixed64，不用浮點距離或 ECS iteration order 決定餘數。
- 只對 roster 的敵隊、目前 entity 存活且 HP>0、非 lethal_pending 英雄分享；死亡／範圍外／同隊／缺少位置／零 share 不支付。NPC／塔擊殺也可支付。英雄死亡後進度存 slot、重生恢復。
- 正式單路生成的兵原本沒有 Bounty；本輪不引入兵線 Gold。MobaMatch 關閉舊 Bounty 的最近英雄、浮點單級 XP 路徑，避免元件新增時雙付。沒有 MobaMatch 的 legacy MOBA 與 TD 原獎勵保持不變。
- Unreal 沿用既有 CommittedProgression／EquipmentStats → runtime／IPC → 共用 HUD；不新增角色 C++、Blueprint graph、ABI 或協議欄位。

## 回歸與真實網路證據

- 新增三個實際 ECS 回歸：NPC 擊殺／1200 邊界／25→12+12／死亡一次性／假 Bounty 不雙付／HP0 不分母／37 XP 死亡重生；inactive／缺位置／正 HP／範圍外／同隊／退休重送拒絕；365 XP 跨多級、零 share 與非法規則原子拒絕。60Hz fixture 重跑 digest 一致。
- template 契約 65 passed（33 lib＋23 hero＋8 numeric＋1 catalog），含 lane rule 改變 hash、拒絕 hot reload、compiled mismatch、負數／浮點／字串與缺欄位。
- core 332、server 154＋1 ignored、runtime 59＋6 ignored、bridge 51＋1 ignored、Fyrox check --tests 通過。base_content 89 項含15／60／120Hz逐tick完整 filtered hash、零 ComponentRepair、雙方死亡／重生、唯一 winner、15 tick Finished freeze；加入重生斷言後最終89項再次全數通過。
- 最末獨立60Hz完整filtered再跑成功：2580 ticks／5158雙隊applied steps，Finished winner0於2566，雙方各death1／respawn1，完整hash與零repair檢查維持。OpenSpec strict、tracked diff whitespace與本輪untracked檔案Lua whitespace檢查通過。
- 兵線 XP 使原 Push／Guard fixture 在部分 fps 可以長時間持續對抗，出現測試 timeout 而非 hash 分歧。候選 Bot 改寫未穩定解決，已全部撤回。生命週期測試改為正式输入劇本：雙方死亡重生後守方 MoveTo 離線，推方仍用原 Push。沒有改 XP／HP／傷害／建築／勝負，不當作五位置或平衡 Bot 證據，5.5 仍未完成。
- 最終真實 `target/interactive-runs/moba-runtime-1791083346`：success／cleanup_verified／60Hz；player3 kill1、player1 assist1、player2 death1，settlement tick1015，兩名攻擊者原 attack input2 各一次 acceptance。兩隊10／10 checkpoints、player3獨立9，至1200且各2 post-kill checkpoints。
- 保存的原始 TeamTickFrame 與 IPC 驗證：1374＋1265＋1244＝**3883 snapshots**，Playing 活體 progression 的level／XP／SP精確一致；公開／owner KDA、Gold收入＋kill／assist reward亦逐樣本精確一致。有實際 XP 超過只由 KDA 推算的最低值，確認兵線 XP 經正式網路到達。不能從安全 view 推算隱藏兵死亡總數，故不宣稱網路驗證逐兵精確分帳；精確25→12+12與去重由權威 ECS 回歸驗證。
- xp_contract=2 保留 raw authority progression exact gate，KDA 僅當最低值；要求觀測额外兵線 XP，不再錯誤以 KDA 斷言全部 XP。舊 xp_contract=1 capture 仍維持原精確 kill／assist-only 契約。
- 四 owned PID 92700／42752／94156／24272 已由 Lua process.inspect 獨立確認退出。第一輪1791082860與3875 snapshots亦通過，但以上以最終 run 為準。

## Unreal 建置

- build-only 經 omfue/restart 成功重建／stage base_content 與 Rust bridge；OmGameEditor incremental build成功。
- staged bridge SHA-256：`e2c3aa85af13f1e369382eaee8924ef31feeca79d4210007e663be0182cda26e`。
- 生成11 files／15 Lua inputs，hero presentation hash仍 `de9c7fcfc98d6479`；此不是完整玩法規則 hash，兵線經驗不改角色呈現schema。
- 本輪未啟動 Editor／MCP／雙 Unreal XP 畫面測試，不能引用前輪結果當此輪畫面驗收。codegen --check11files／15inputs與最終stage核對已通過。

## 未完成

5.3 尚缺正式技能升級輸入／四槽rank／UE操作與相關非法輸入驗收；Unreal XP畫面、多英雄兵種／野怪XP與完整對局UI尚未驗收。OpenSpec維持19/30，不勾選整個5.3或6.2。
