# Support 共用護衛目標（2026-10-05）

## 計畫與決策

1. 延續5.5五位置Bot：Support已有跟隨與隊友治療，本輪補護衛攻擊目標。不新增英雄專屬handler、C++或Blueprint，不改正式Lua技能與catalog。
2. 以既有escort_player_id找當隊已披露、存活hero；Carry距Support最多550。只從距Support550內且距Carry350內的已披露存活敵方hero選擇，依距Carry、距Support、canonical ID穩定排序；不以HP或容器順序猜威脅。
3. 這是局部接近觀察，不讀敵方aggro／command／hidden pose，不宣稱敵人正在攻擊Carry或戰術安全。未找到有效escort／guard，沿原有攻擊／跟隨／公開路線回退；不對全圖追逐。
4. 新role_combat_focus統一Jungle局部支援與Support護衛，正式planner每位Bot只計算一次，普通AttackTarget與進攻技能共用。單體／接近point／AoE中心仍受各自rank、range、min_targets、成本與CD驗證，focus只是優先順序，不能授權非法目標或忽略效果門檻。
5. AllyHeal、自療、學習、續航與購裝原先優先順序保留；護衛不搶先作者heal-first政策、不直接改HP／Mana／命令，仍由正式PlayerInput與生成handler執行。

## 當前功能確認

- core support_guard_focus 2 passed：包含隊友／敵人存活、種類、隊伍、550／350界線、缺escort、反向順序與canonical tie；三種進攻intent共用focus且保留heal-first。
- 正式60Hz四玩家／正常Lua generated handler功能確認：1 passed。Support忽略距本人50的另一敵hero，對距Carry較近的400位置敵hero施放lumen_bolt，HP10000→9920、cost45加正常5×active dt再生、CD正常；移除Carry current披露回退近身敵人，未提交Carry新座標不改決策。後續AttackTarget仍選護衛目標，未注入傷害或替換handler。
- 初次正式確認失敗：fixture錯寫capacity200而正式Lua上限400，current158805 raw實際一致；修為沿出生maximum只設定current200後通過，不改authority容量刷新。root whitespace通過。
- 本批僅上述功能，不跑full suite／100場／UE／完整驗收、不build或stageDLL。完整5.5／6.2仍待，OpenSpec20/30不變；E177共享引擎基線保持未解。

## 錯誤與防錯

- 本輪有一次猜single_lane/bots目錄而回os error3；實際是runtime/native/moba_match/bots。改先rg --files找實際位置再讀檔，記E192。不新增工作流shell fallback。
- cargo短暫等待package cache lock是跨workspace並行編譯的正常等待，未刪lock或終止其他程序。既有template build-script三項dead_code warnings保留，不與新功能失敗混淆。
