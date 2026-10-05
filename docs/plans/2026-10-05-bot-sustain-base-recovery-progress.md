# Bot 回城續航與權威基地恢復（2026-10-05）

## 計畫與決策

1. 檢查既有 Recall 發現只傳送、不恢復HP，因此單純低血回城會形成基地反覆回城。先补可選共用基地恢復，再接 Bot 決策，不讓 Bot 直接改位置／血量。
2. Lua moba_economy 新增 base_recovery_hp_per_second=120／base_recovery_radius=300；完整parser、產生常數、canonical data hash、compiled agreement與hot-reload gate同步。省略欄位為0，TD省略整個MOBA section仍維持舊預設。
3. SingleLaneConfig.base_recovery_enabled 預設false。新角色配方的可選sustain會明確啟用共用基地規則；沒有policy的旧Story、單路與既有fixtures不改。規則同時適用真人與Bot，並非Bot專用補血外掛。
4. 恢復只由authority post-combat commit使用Playing active delta，Recall完成後先核對己方存活基地、自己活體／正HP／非lethal_pending與距離，再以i128 fixed raw乘rate、上限mhp結算。warmup、pause、Finished、死亡、範圍外均不補血；不以回血復活已死亡生命。
5. 最終HP沿既有visible EquipmentStats／安全post-step投影，filtered world不持有MobaMatch也不再次計算恢復。沒有新前端玩法或角色C++。
6. BotSustainPolicy 嚴格宣告 recall_below_hp_per_mille=350、leave_base_at_hp_per_mille=850、threat_radius=1000，要求0<回城<離開<=1000及距離1..10000。一般角色配方加policy，單真人／混合原型配方沿用。
7. 決策只讀owner健康、當隊committed disclosure与公開路線home。低血且有已披露活體敵方／中立威脅→正式MoveTo撤向己方基地；附近無已披露威脅→正式Recall。隱藏敵人、記憶ghost、私有aggro不查。這是保守原型，不保證未知敵人無法中斷回城。
8. 已在Recall讀條時不送其他input；到基地且低於離開門檻時以正式MoveTo停止舊追逐後等待，達門檻才恢復角色／學習／施法。移動重送依owner現有命令去重，沿E160近距離stop規則，避免每tick追逐變動中的自己位置。

## 當前功能確認

- core 指定role_bot_sustain 2/2 passed：threshold精確邊界、死／同隊披露不算威脅、已披露敵人撤退、home hold／leave、非法policy；配方None維持disabled，Some原子啟用並編譯，非法門檻拒絕。
- base_content 指定base_recovery 3/3 passed：真實60Hz正式Recall channel／實際基地傳送／讀條不打斷／恢復再出發、pause不回血、maxHP上限／0HP不復活；disabled／warmup／範圍外均不補。
- 同組的短filtered測試20 ticks、雙隊共40 steps：完整canonical team hash逐tick與fresh authority bootstrap一致、零ComponentRepair。這不是全場filtered／KCP／Unreal或60FPS驗收。
- template-id runtime-lua-content規則測試1/1 passed：rate及radius修改進canonical hash，單邊compiled mismatch與dev hot reload拒絕。選base_content相依feature啟用workspace外template-id；伴隨base篩選0tests只算編譯，不算測試通過。
- 新混合單真人配方固定Lua prepare-only與正式Rust moba-config解碼成功：一真人九Bot、60Hz；沒有game／IPC sockets或Unreal程序。預檢與launch-plan現會保存base_recovery_enabled，不將prepared metadata當實際呈現證據。
- Unreal同一模板重新產生11檔／17個Lua inputs；generated content_hash=d197946331c9476f／identity=58136a29dddae4af保持不變，完整catalog_data_hash變為a3037e861b121026。前者不是完整玩法版本，後者才覆蓋本次恢復規則。
- root／omb／omfue whitespace檢查成功，未提交、推送或刪除既有變更。

## 仍待完成

Bot不含最佳化逃跑路線、精準補刀、完整隊友保護或商店策略。本批未重建OmGame或stage新DLL，未執行100場、LAN、Unreal完整對局或最後效能驗收。OpenSpec維持20/30，5.5不勾選；錯誤與修正集中E166。
