# 2026-10-22 召喚物正式呈現錯誤紀錄

- P01：盤點多次猜測不存在的render_snapshot.rs、summoned_unit.rs、saika_magoichi.rs、NativeCreepPresentationTests.cpp與team_replica目錄；實際來源分別為snapshot.rs、comp/unit.rs、heroes/B01_saika_magoichi、OmNativeAnimatedUnitTests.cpp與runtime平面檔。無來源變更，已依rg --files／實際宣告定位。後續禁止從名稱猜檔案。
- 工具：OpenSpec CLI既有環境未安裝，沿手動spec-driven artifacts實作。
- 防錯：召喚物與小兵有同名catalog，必須使用明確公開類別，不能依ID存在、owner或Faction推定；不向filtered複本加入私有SummonedUnit／AI。
- T01：RustV1新增snapshot分類使用Unit／UnitType，原檔採顯式import而非comp glob，漏列造成4個編譯錯誤；core、base與其他依賴case失敗。模板／codegen成功不能當整批成功。等待RustV1 owner／watcher退休後補顯式import並完整重跑RustV2；保留原始log／receipt。
- P02：Saved World helper以Lua pattern插入CLI旗標時未把所有hyphen當literal，留下多餘引號。World尚未啟動；改用精確字串修正並在啟動前做Lua語法檢查，既有test IDs／旗標保持。
- T02：RustV2 core908成功／2失敗。既有fractional snapshot精簡World未註冊Unit，新增read_storage強制依賴造成panic；生產extractor應維持可用於無Unit的World，改以has_value<MaskedStorage<Unit>>檢查後才讀取，無storage自然非Summon，不放寬其他身分判斷。
- T03：新snapshot fixture使用StateInitializer基本World，缺TowerUpgradeRegistry，full extractor因此panic；依既有extraction_world契約補registry，不以空值吞掉正式必要資源。整個RustV2 owner／watcher退休後才修，保留原始失敗證據。
- T04：新正式60Hz fixture將敵人直接移到近處後未提交WaveB公開可見性，召喚AI正確拒絕從私有位置猜索敵，因此100tick內未開火。沿既有summon正式測試補run_committed_visibility_wave_b後才觀察projectile／動畫，不放寬production visibility。
- T05：WorldV1在既有NativeCreepPresentation回歸測試崩潰。把新有正式外觀的召喚物替換成AOmAnimatedUnitActor來測空設定，但該基底標記UCLASS(Abstract)，SpawnActor回傳null。修復應使用具體generated summon並明確清空可選NativeMeshAsset，沿既有缺省外觀測試的反射契約，且先TestNotNull再使用；不移除抽象語意。
- T06：RustV3正式召喚射擊後assert attack_visual_state.is_some失敗。該欄只來自MobaMatch管理的野怪／兵線狀態，腳本召喚普攻經ScriptAttackPhaseCue→snapshot.attack_phase_fx，Bridge已有通用FX fallback。測試應接受真正的公開attack_phase_fx並比對ID+generation，而非要求錯誤的呈現渠道；不新增重複動畫狀態或放寬正式戰鬥規則。
- P03：追查T06時再次使用不存在的runtime/simulation_driver.rs、comp/unit.rs及PS不展開的summons*路徑；無來源變更。已重新rg --files定位runtime/native/comp與summons/saika_gunner.rs，延續P01防錯。
- P04：Saved修復工具使用path.write(...,false)企圖修改既有來源，被防覆寫保護拒絕；拒絕發生在第一個來源寫入前，沒有部分變更。隨後RustV4啟動因腳本尚未產生而失敗，owner／watcher已退休。修復明確使用overwrite=true並移至RustV5，保留V4失敗receipt，不覆寫證據。
- P05：WorldV2尚在執行時過早打開第二張map PNG，當時尚未產出而讀取失敗；第一張overview與新召喚物已可讀。以最終World成功及檔案存在為視覺驗收前置條件，全部產出後再完成七張目視，不把缺圖當已驗。
- P06：證據封存假設所有watcher都至少產出一筆health，RustV4 runner腳本缺失而立即退出，watcher尚未取樣便退休，沒有health.jsonl；封存停止且未產生成功index。明確核對該失敗receipt的owner／monitor皆退休，僅對RustV4缺省health作已知例外；重入時既有證據須逐byte相同，錯誤MD更新另明確覆寫，不刪除失敗證據。
