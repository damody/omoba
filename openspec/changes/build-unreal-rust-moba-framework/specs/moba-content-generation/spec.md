## Purpose

定義 Lua、Rust 與 Unreal 生成內容之間的穩定契約，使新增英雄、技能、物品與呈現資產時不必手寫角色專屬 Unreal 程式。

## ADDED Requirements

### Requirement: 正式對局只執行編譯內容
系統 SHALL 僅在建置／生成階段求值 Lua 內容，正式 MOBA 執行元件使用生成並編譯的 Rust／Unreal C++，不得啟用 runtime Lua 內容載入或熱更新。開局工具可將 Lua 配方轉為一般 JSON／TOML 設定，遊戲不執行該配方。

#### Scenario: 正式建置誤啟用 Lua
- **WHEN** 正式 compiled-content-only 建置同時啟用 runtime-lua-content
- **THEN** 建置明確失敗，不容許 Cargo feature 合併悄悄帶入執行期 Lua

#### Scenario: 不部署 Lua 作者檔
- **WHEN** 使用已建置的正式遊戲元件與生成的對局設定
- **THEN** 英雄／技能／地圖資料與行為由編譯內容取得，不依賴部署 scripts/lua_data

### Requirement: 單一內容來源
系統 SHALL 從經驗證的內容定義產生 Rust 與 Unreal 所需的穩定 ID、資料與註冊資訊，並拒絕重複或失效的引用。

#### Scenario: 主動物品編譯資料
- **WHEN** 作者在 moba_items 宣告合法 active 與 cooldown
- **THEN** Rust 生成入口 SHALL 驗證五種 native kind 閉集、有限且不低於量化下限的量／時間／比例／冷卻與未知欄位，產生型別化 Fixed64 常數並由共用物品 registry 轉接 native 執行器；被動預設欄位省略不得改動既有 canonical 商品資料，不使用 runtime Lua 或獨立手寫 JSON 效果副本

#### Scenario: 通用敵方控制效果
- **WHEN** 作者宣告 control_enemy 與每級 duration_key
- **THEN** 兩個生成入口 SHALL 只接受 stun／root／silence 閉集、完整有限時間與合法敵方單位射程；編譯執行器 SHALL 在整份效果預檢後提交標準控制狀態，定身不得阻止合法施法、沉默不得凍結移動或普攻時鐘，舊 stun_enemy 保持相容

#### Scenario: 無效技能引用
- **WHEN** 英雄引用不存在的技能 ID
- **THEN** 生成失敗並指出英雄與技能 ID

### Requirement: 英雄模板生成
系統 SHALL 依英雄內容定義產生所需的 Unreal C++ 類別與資產配方，且重跑相同輸入不得產生差異。

#### Scenario: 通用動畫宣告與型別驗證
- **WHEN** 任意英雄提供動畫片段及可選的 Unreal 動畫 metadata
- **THEN** 特殊角色狀態 SHALL 僅由 Lua 顯式宣告，不由片段名稱推斷 Saika 狀態；Rust 與 Unreal 生成入口 SHALL 共用映射、idle list、播放倍率與 fallback policy 驗證，非法宣告在輸出前指出內容 ID 和欄位，不靜默降級為預設值。

#### Scenario: 新增英雄
- **WHEN** 開發者提供完整 Lua 內容、必要的 Rust 行為與美術資產
- **THEN** 生成器產生可編譯的英雄註冊與呈現資料，無須手寫角色 C++ 或 Blueprint graph

#### Scenario: 動畫時間生成一致
- **WHEN** 英雄片段宣告來源時脈與可選 impact_tick
- **THEN** Rust與Unreal建置使用共用片段驗證及ticks-to-seconds換算；拒絕非有限、無來源、非正時脈、非法片段區間與循環片段命中點，未宣告命中點保持未知

### Requirement: 內容版本一致
系統 SHALL 驗證腳本 DLL、生成資料與 Unreal 內容的版本及內容雜湊，遇到必要內容錯配時拒絕開局。

#### Scenario: 舊版腳本 DLL
- **WHEN** 載入的腳本 DLL 與本次生成內容不一致
- **THEN** 系統顯示明確錯配原因並停止啟動該對局
