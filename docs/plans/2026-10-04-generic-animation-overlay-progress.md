# 通用動畫 overlay 投影

## 計畫與決策

- 接續 `build-unreal-rust-moba-framework` 的 2.2b／6.1；依使用者指示只確認本功能，最後再完整驗收。
- 問題一：bridge 用 sniper_mode／three_stage 名稱選 overlay，並以 hero_render.sniper_mode 再覆蓋。改為 catalog 載入時解析 Lua animation_overlay，實際 buff 清單決定適用性；衍生 flag 不再當額外真相來源。
- 問題二：Unreal 將所有非零 overlay ID 叫作 sniper_mode，走路一律 sniper_walk。改生成 OverlayName／OverlayWalk／OverlayStand 常數欄位，以共用 model 解析，不在每幀解 JSON、不新增角色 C++ 或 graph。
- 排序：最高有限 priority 優先；同 priority 使用最小穩定 numeric catalog ID，不受 buff 清單順序影響。允許負優先序；無 overlay 回普通動畫，未知 ID 不猜角色名稱。
- 可選 locomotion_variant_id 是 u32，0 表示保留原普通變體；既有 sniper Lua 明確宣告 2。walk／stand 名稱直接從原 locomotion map 生成；缺少該狀態映射時保留普通 locomotion fallback。
- 問題三：未知 hero catalog 不能回退到 Saika。移除該回退，未註冊英雄保持 ID=0，交給既有通用未知內容呈現。
- C ABI11、已存 Blueprint reflected API 與 legacy typed payload 不變；不是任意 animation action graph 的完整替代。

## 使用方式

在任意 buff 的 Lua 定義加入：

```lua
ue = {
  animation_overlay = {
    overlay = "custom_stance",
    priority = 90,
    locomotion_variant_id = 8,
    locomotion = { walk = "custom_walk", stand = "custom_idle" },
  },
}
```

overlay 名稱省略時採 buff ID；不依賴 Saika ID 或為該 buff 新增 C++。既有 action metadata 保留，這批僅遷移 overlay 選擇與 locomotion 名稱。

## 本功能確認

- codegen 兩個直接測試通過：非法 priority／map／variant／overlay 型別拒絕；任意 buff 的三個名稱確實產生在 native registry。
- bridge 兩個直接測試通過：優先序／穩定 tie／清單逆序／移除／無 buff 時忽略舊 flag；unknown hero 不冒充 catalog hero。
- 原快照 projection 測試以明確 Lua overlay metadata 更新 fixture，1/1 通過。
- 正式來源生成與 --check 通過：11 files／16 Lua inputs，content_hash `4243c8ad19f1f607`。
- 最後 build-only session 27886 exit 0、OmGameEditor Succeeded（C++ 已在前一次建置編譯，本次 up-to-date）；bridge 補增量編譯並 stage。最後 SHA-256 `c31089dfe1452bfa638e9f5ac70bf16a29115a9fa662d5d892f5068389a99ad3`，DLL 22:16:58 晚於最後 projection.rs 22:14:40。沒有在此後修改程式。
- 本專案 Editor 101752／MCP HTTP 30000；GenericAnimationOverlay 單輪 1/1 passed、error_count=0／warning_count=0。報告 `omfue/Saved/McpAutomation/GenericAnimationOverlay-20261004/report.json` 為 feature-confirmation，不覆寫完整驗收報告。測試確認共用 model 的任意 overlay／walk／stand 與缺值 fallback，不冒充 saved Blueprint graph／實際美術動畫驗收。
- 沒有跑完整 PIE／雙 UE／效能驗收；Editor 保留開啟。

## 尚未完成

- saved Blueprint 的 HandleSaikaActionEvent 引用、typed ABI 相容欄位與完整 legacy adapter 移除仍未完成；2.2b／6.1 不勾選。
- 完整技能／攻擊 action 動畫品質、三種英雄、整場對局與 LAN／效能留最後整合驗收。
