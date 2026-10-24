# Design

共用sim geometry提供有界整數capsule包含測試；沿用現有terrain座標限制與i128計算，邊界相切算命中，非法資料不算命中。

base_content共用swept dash執行器先校驗、依內容射程clamp，再透過既有advance_with_collision查詢整段可行性；受阻時在固定點距離上二分找到同一直線最遠合法端點，最多25次查詢，不借助未公開敵方碰撞。定身／完全阻擋／零位移失敗；在確定實際端點和所有命中後才產生位置、朝向及傷害outcome。候選用實際路徑包圍圓查詢，再以精確capsule過濾、canonical排序和去重。

fire_dash沿用逐rank內容：range／dash_width／damage_per_tick／dash_duration。傷害換算為per_tick * duration * 10，避免0.1量化成102/1024造成面板與實傷不符；每個路徑內敵人只命中一次。此批是瞬間、受碰撞限制的沿路衝刺，不宣稱已實作0.5秒逐tick移動／火焰殘留／持續引導。metadata改成既有Custom selector描述沿路命中，不再冒稱終點圓形。

OpenSpec CLI先前環境不可用，按已讀apply skill手動維護artifact；使用者授權自主選擇和修復，不另詢問例行選項。
