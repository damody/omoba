# 正式可見單位的通用移動動畫（2026-10-05）

## 本輪計畫與實作

1. 檢查formal IPC動畫來源：找到is_moving固定false、沒有private HeroCommand，以及attack時間不完整。
2. 從已披露位置接通通用移動動畫：完成，不新增第二份gameplay模擬。
3. 確認邊界與記錄問題：4項局部測試通過，最後完整驗收留待功能齊全。

## 決定與規則

- 正式快照轉換後英雄沒有private命令，hero_render.is_moving原固定false。ProjectionState新增disclosed_motion，以(entity ID, disclosure generation)保存當前有限座標／tick／observed moving；同身分較新tick位置變化才walk，較新tick沒有變化則stand。這是畫面位移觀測，不是權威命令、目的地或速度。
- 第一個樣本stand；相同tick且相同位置保留前次觀測，不能把control／duplicate當停止；相同tick卻不同位置或tick倒退重新建基準且stand，不連接矛盾旅程。paused明確stand。
- 無合法非零身分／世代、非有限位置或HP非正不保存樣本；完整快照缺席即prune，只保留當前live集合，沒有存hidden／remembered軌跡。世代不同不比較。
- 每個driver更新先處理顯式removed tuple，即使snapshot之後coalesce掉也不連接舊歷史；ViewRemoved全reset即使列表空；stop／restart清除。snapshot.removed_entity_ids也使對應觀測重新建基準。
- 僅presentation-only使用這份觀測；embedded／TD仍使用既有hero_render／hero_command，切換回embedded清觀測。Buff overlay priority／walk variant保持，沒有private payload、controller命令或玩法更新。
- 本輪不改Lua內容、native C++／Blueprint、C ABI13、IPC4或wire3，不stage、不部署、不維護omfx；既有原生consumer可接已生成的locomotion record。

## 實際缺口：正式攻擊動畫时间

- formal presentation_snapshot_to_frame沒有attack_phase_fx，且CommittedAttack只有13bytes counter／sequence／phase。TAttack.asd.v是基礎間隔；hero_tick使用Buff聚合final_attack_speed_mult後的effective_interval，不能直接以基礎數值重建真正時間。
- 前兩輪E233／E234已實作協商時脈與nativephaseconsumer、Lua命中點，但它們是有合法attack資料時的consumer，不等於formal IPC有完整權威攻擊來源。沒有偽造target、critical或時間來讓動畫看起來已接好。
- 下一步須建立權威攻擊解析結果的安全持續時間契約，完整baseline／committed更新／filtered runtime／IPC／bridge接線，明確處理版本與世代；在這完成前不勾6.1。
- 位移技能／teleport改變公開位置時，本功能只能觀察樣本間位移，無法辨識移動成因；不宣稱平滑velocity或dash動畫。獨立typed cue與未來權威動畫狀態可提供更精確語義，不以private state補推。

## 局部確認

- disclosed_motion_tracks_only_current_identity_and_observed_positions：首次／移動／重複tick／停止／paused／舊tick／新世代／NaN／缺席／retirement／死亡。
- disclosed_motion_does_not_override_embedded_animation_or_overlay_priority：公開移動套同Buff overlay與variant，embedded不被取代。
- disclosed_motion_history_respects_reset_removals_control_and_stop：RuntimeInner實際快照發布／control-only／explicit retirement／ViewRemoved與stop。
- disclosed_motion_from_ipc_positions_reaches_animation_without_private_commands：真正TeamPresentationSnapshot converter，對手live披露位置→stand→walk，沒有hero_command，隱藏後再披露不連接旅程。首次E0433漏import已修正後通過。
- 共4項不同測試通過。沒有全套、UE模組重建、MCP／PIE／截圖或完整對局；native本輪沒有變更，不重複已知E224啟動問題。
- OpenSpec strict與主repo／omfue whitespace檢查通過。
- OpenSpec仍21/31，6.1與正式攻擊來源維持未完成。
