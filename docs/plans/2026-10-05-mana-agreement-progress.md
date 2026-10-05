# Mana 規則協商與正式配置

## 決定與實作

- `mana_transport` 共用 protocol version 1 與完整 `CONTENT_CATALOG_DATA_HASH` 驗證。version 0＋空 hash 表示未宣告；部分宣告、未知版本、缺失／過期 hash 拒絕，不降級或猜測。
- Proto JoinRequest 追加 14／15、TeamGameStart 追加 27／28 的 Mana 版本／規則 hash，既有 tag 不變。正式 build.rs 更新 tracked fallback，不手寫 generated Rust。
- 新 KCP selective client 宣告支援並驗證回覆。legacy native／web request 明確傳零／空值，沒有假裝支援新規則。
- server `MATCH_MANA_ENABLED` 預設 false；只有 secure_v2_required 的 single_lane／three_lane 可以啟用。一般與 role-plan 配置都轉入共用 `SingleLaneConfig::mana_enabled`，launch preflight 回報該值；沒有修改實際 game.toml 預設或啟動新對局。
- server 在呼叫玩家註冊／改動 session 前驗證 Mana 宣告；啟用對局拒絕未宣告支援的加入者。有效宣告仍要求 selective V2、supported V2、secure fog、非零 player ID、player role，並沿既有 authenticated team registration 邊界。不接受 observer／legacy 冒充能力。
- 正常啟用對局的初次與重新加入 bootstrap 都回覆版本／hash；停用對局回覆零／空值，新 client 宣告支援不代表 server 已啟用。
- 修正未啟用且沒有 managed pool 的對局不發 `CommittedMana` fact26；明確啟用或測試中的 managed pool 才結算。避免以 disabled None 新事件破壞舊端。

## 當前功能確認

- core `mana_agreement`：1/1，合法、未宣告、部分／未知／過期宣告。
- server `mana_agreement`：2/2，配置預設／模式限制／轉入對局，以及註冊前能力／player 身分條件。
- base_content `mana_agreement`：1/1，正式 60Hz 停用／啟用各三 ticks，停用零新 fact，啟用每 tick 兩個合法 managed pool facts。
- root、omb、omfx whitespace diff 檢查通過。編譯期間既有 td_rounds warning 與 server 的 protoc fallback warning 保留；fallback 已由 core 的 vendored protoc 同步，本輪 server 編譯成功。

## 未完成界線

本批是協商／配置／正式 ECS 輸出檢查，不是實際 socket 混合版本／重連／LAN 驗收，也不是 Unreal Mana HUD 成功。沒有重啟 Editor、停機／覆寫共享引擎、DLL stage 或長對局。E177 的 native ABI12／共享引擎建置問題仍存在。一般啟動預設仍停用，需後續短網路接線確認與可用 native 編譯基線才把 launcher 配方改為預設啟用。

Bot 法力預算／補魔策略、Buff／裝備再生、explicit script 資源操作與完整框架仍待；OpenSpec 維持20/30，不勾選4.1／5.5／6.2／6.4。
