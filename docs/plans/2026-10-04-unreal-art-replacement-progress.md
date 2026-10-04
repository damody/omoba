# Unreal 美術替換流程驗收

## 計畫與決策

1. 60Hz單路對局已通過，下一步驗6.3：日常只換美術來源／Lua配置，生成器與MCP負責Unreal設定，不手寫角色C++／Blueprint graph。
2. 使用既有Saika原生模板，暫時Lua `ue.native_visual={scale=0.9,yaw_deg=-45}`，重跑完整生成／編譯。原版render.scale0.012在UE為1.2；測完移除override並重建原版。玩法數值、技能、Rust handler不變。
3. 同一已由ledger持有的來源atlas換成既有default portrait PNG做受控測試：256x256→160x160→256x256。這不是新正式美術，只是具有不同尺寸／像素的驗收素材；不修改原圖像素或新增英雄程式。
4. 全來源二進位備份＋SHA，MCP讀回Texture2D尺寸／sRGB、依既有材質引用實際PIE渲染、三張PNG人工檢視；每階段配方執行兩次，第二次必須無import／material-binding改動且22套件SHA不變。
5. 比對資產替換期間全部OmRuntime Source C++／headers及Content/RustBP套件SHA，確認工作流沒有改角色程式或graph。Lua變更造成的生成C++差異是生成器輸出，不是人工編寫角色程式；測試framework僅改掉寫死1.2的斷言。

## 實作

- `scripts/tests/ue_art_swap_editor_test.lua`：固定owned recipe source，所有preflight通過才替換，來源backup留在target/art-swap-runs。無論MCP／PIE失敗都嘗試還原；若第三方在測試中改來源，拒絕覆寫並保留backup，不假裝已還原。新run另保存protected-before／after manifests。
- `scripts/tests/ue_art_swap_acceptance.lua`：獨立重讀原始source／backup／替代source／三張PNG的binary signature與SHA，核对raw readback、PIE／配置與idempotence結果，輸出可版控compact證據。
- 原共用Editor test以生成CDO NativeMeshTransform核對实际mesh；PIE從同一Lua來源求expected scale，保留真正數值斷言。沒有修改角色runtime actor C++或Blueprint graph。

## 已驗證

- codegen24項測試通過；variant fullbuild2110 exit0／UBT9.40秒，Editor49928同session兩輪13/13通過。Lua override0.9進actual PIE，畫面有native model、材質與動畫。
- run `art-swap-1791066725` success=true／source_restored=true／code_and_blueprints_unchanged=true。
- baseline／replacement／restored：256／160／256 square，sRGB=true；各native_mesh_rendered=true、actual scale0.9。替換及還原各first import2 jobs；三階段第二次import0、binding mutation0、10 verified jobs／22 saved packages逐SHA不變。
- 三張2353x1099 actual PIE PNG已直接檢視：替換後模型body色彩變更、還原後回復。PNG不同hash不是唯一證據，尺寸readback與實際mesh rendered亦通過。
- 原source SHA `b3ccc7216607256aa627abf065cf4fea0a2355f948a0abee318ea16a73f72cfa`，替代 `39b902dbf424b603049e97fc9f8e742cddca912a1322f40b0f1e5edd52245e97`；最後source與original backup逐byte相同。
- 保存verifier獨立exit0，證據 `openspec/changes/build-unreal-rust-moba-framework/evidence/art-replacement/art-swap-1791066725.json`。
- 臨時Lua override已移除；原版fullbuild57914 exit0／UBT5.77秒，content_hash重新為de9c7fcfc98d6479，最後stage SHA cc76ef2d8328126a7ca255de84fa49daf72272af426ba9a0b40f8ceb8871d709。原版Editor85496同session兩輪13/13、PIE18379含截圖／actual scale1.2／mesh rendered通過；codegen --check與recipe planner通過。
- Save／QUIT後固定Lua process.wait確認Editor85496退出。獨立60Hz release雙端run1791067086在原240秒期限內success=true／cleanup_verified=true；雙四槽4/4、自己的HUD與位移、死亡重生10／9、原attack input8各status0 tick3663／3661、實際基地winner2。Finished12837（213.9秒）／UI12838，兩張1280x720 Defeat／Victory與頂端60Hz已直接檢視，保存verifier獨立exit0。
- 最後三方hash216／215 PASS rows（各108 unique ticks）至12960、0 FAIL；UNVERIFIED25902／25900不算PASS，UI後122ticks通過。五PID94372／3556／88660／78668／63348查無，沒有留下測試程序。證據evidence/unreal-match-lifecycle/interactive-ue-1791067086.json。
- 美術PIE使用既有legacy standalone，不拿截圖的120Hz作正式MOBA／120Hz效能證據；正式回歸只有60Hz，且不宣稱穩定GPU60FPS。

## 重現

固定Lua按順序 `scripts/build_ue_moba.lua --full`、`scripts/tests/ue_art_swap_editor_test.lua`；再以 `scripts/tests/ue_art_swap_acceptance.lua target/art-swap-runs/art-swap-<run>`核對。若測Lua override，在heroes.lua設定上列native_visual後完整重建；最後移除override、MCP Save／QUIT、確認PID退出後再重建原版。不要以Live Coding或舊DLL替代重新生成。

## 完成範圍與限制

還原保證指原始PNG來源逐byte與Lua內容／Unreal貼圖尺寸及呈現恢復；UE reimport可能改寫套件metadata／GUID，不宣稱整個還原uasset與最初套件byte完全相同。真正冪等門檻是同一階段第二次配方執行時套件SHA不變，三階段皆已驗證。

6.3的代表性美術／Lua配置替換流程已驗收，整體18/30。這不代表任意不相容骨架的retarget、自動校正UV、全部角色動畫品質或完整遊戲已完成；這些仍屬6.1等剩餘範圍。模型／Skeleton本次保持相容不替換，替代portrait只作atlas驗收，正式原始美術均已還原。所有錯誤與防重犯措施在E102。
