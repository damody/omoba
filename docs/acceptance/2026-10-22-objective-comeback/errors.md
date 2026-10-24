# 2026-10-22 目標經濟補償錯誤紀錄

- C01：初始盤點查詢猜測 native/mod.rs，實際模組為 native.rs；改用 rg --files 確定位置。
- C02：精確編輯 helper 預期一個 tower_plating synchronize 實際有兩個（inactive finished 與結算 finished）；assert 在寫該檔前拒絕。保留已完成前段，只執行明確後段，兩處均需清除補償狀態，沒有整份重跑或重複插入。
- C03：查詢猜測 OmGeneratedBuffCatalog.h／runtime.rs 不存在；依 rg --files 和既有測試確認 OmRegistry.h／native.rs。測試 helper place_hero 實際接受 x/y 整數，改用既有 Pos 寫入；編輯時發現 C++ 列舉行多出 + 字元已修正，均在啟動任何編譯前處理。
- C04：V1 template/codegen/generate/ABI成功；core/base/backend/bridge/client因 core 未宣告直接使用的 omoba-content-model 而 E0433，九工作均退出後修正。template 的該依賴在 compiled-only 模式是 optional，不能依賴間接重匯出；改明確加入 core native dependency，沿既有本機 crate 重用完全相同的規則與驗證，不新增外部套件，也不改 ABI crate。重跑 V2 全組；原始V1完整保留。
- C05：V2 core902全通過，base新60Hz fixture呼叫 disarm_attack(tower) 對不存在的 TAttack unwrap；塔攻擊由既有腳本／Unit路徑提供，不能套英雄專用測試 helper。UE與後端已成功，先完成現有全批測試／原生程序退出後，只移除該fixture無效呼叫，再重跑完整Rust；不修改正式塔規則去迎合fixture。失敗原始紀錄保留。
- 設計防錯：錢包不可當總經濟；累計只記實得的正向 authority 獎勵，排除初始金、購物、出售與退款。獎勵統一接 credit_moba_player，飽和金額只計實付。重生錢包必須通過既有 admitted predicate。
- 設計防錯：補償資格於 tick 起始鎖定；同 tick 目標按 tick／entity generation identity 排序，總額有限。耗盡不在仍落後時自動補滿；差距回到退出門檻後才能再次啟用。
