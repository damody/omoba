# 兵線光環錯誤紀錄

沿用前批程序／watcher退休才重試的守門。新增功能先整批實作，集中驗收；每三分鐘檢查內層Rust進度與程序。

- A01：寫測試時初用不存在的Vec2::from_i32；來源檢查發現後，於建置前改為固定點Vec2::new的point helper。先讀實際型別API，避免猜測建構子。
- A02：讀傷害定義時誤猜native/damage.rs及outcome_processor.rs；以rg --files與符號搜尋定位到comp/damage.rs、game_processor.rs。不得猜測模組路徑。
- A03：實作檢查發現兵線DamageInstance發射與防禦結算未聚合BuffStore。同步改用既有UnitStats，並新增正式60Hz實際傷害／減傷接受，避免只驗證UI或聚合getter。
- A04：V1 Rust／UE與後端共同於template驗證編譯失敗E0382。新增owned Option使policy tuple不再Copy，insert後比較發生move；改為借用Option<&Aura>，不複製政策字串。三worker／watcher均退休且cargo／rustc退出後才修正；保留V1證據並改V2驗收，Lua三契約V1成功。
