# Projectile source context errors

- P01：承接盤點時猜測ability_runtime/ability.rs，檔案不存在。後續依rg --files及實際宣告定位。
- P02：rg傳入native/outcome* Windows路徑萬用字元，觸發os error 123。後續改目錄與-g檔名filter。
- P03：讀取comp/tattack.rs猜測路徑失敗；實際TAttack在comp/tower.rs。已改用rg定位宣告。
- 決策：本批改進正式戰鬥規則，不重做已接受HUD；使用者要求通用解，禁止依塔名特判補效果。
- P04：再次猜測state_initializer.rs位置失敗，該讀取無實作影響。應只用既有StateInitializer公開入口，不猜測模組檔名。
- P05：core-v1編譯發現測試的origin欄位錯加到Outcome::ProjectileHit，而非ScriptEvent pattern（E0559／E0027）。確認worker／watcher退休後精確修正；保留v1失敗證據，不覆寫。
- P06：core-v2既有914項全過，新增3項中2項因fixture未安裝TowerTemplateRegistry失敗（915過／2失敗）。補齊正式傷害管線必要resource，未放寬正式程式檢查；原v2證據保留。
- P07：core-v3實際917項全過，但臨時wrapper未輸出launcher所要求的exit=0標記，receipt被判失敗。這是驗收包裝錯誤；保留原receipt，不冒稱成功；最終以正式Rust十工作summary及receipt接受。
- P08：V1集中回歸及建置全過後，檢查發現退休Summon的共用索敵仍要求source alive，會擋住已承諾彈體回呼。所有V1讀取來源的工作退休後修正；僅精確projectile source回呼跳過來源死亡／stun admission，普通AI及目標可見性／可攻擊性／LOS守門保留。新增退休召喚物及禁止目標案例，最終以V2接受。
- 決策：不可變來源快照使用Arc共享（serde rc），子彈／追加命中沿用同一snapshot allocation；新增ptr_eq斷言避免退回逐子彈深拷貝buff。V1內容測試182.61秒，未據此冒稱固定硬體效能接受。
- P09：V2新增召喚物案例失敗（core918過／1失敗），原因是fixture沿用Ally來源；現有Faction::is_hostile_to不將Ally→Enemy列為敵對，正式召喚物使用Player。修正fixture為Player，保留敵對規則；另涵蓋來源仍活著但已stun的承諾彈體。V2其他九工作成功但整體仍為失敗，保留summary／receipt／核心錯誤log。V3改成首個case失敗即停止後續工作，避免已知錯誤後浪費整輪時間。
- P10：WorldV3 51項成功／0 Failed／0 NotRun，但warnings由既有4個Untitled World actor清理警告增至5個。新增者為UE自動連線探測 https://www.google.com/generate_204 的3秒HTTP逾時，並非遊戲測試失敗；保留引擎log與report，不將5個全稱為既有警告，不為外部探測重跑已成功的遊戲測試。
- 最終：RustV3十工作成功，core919通過；WorldV3已退出後目視10張PNG。沒有放寬正式敵對／可見性／目標守門。
