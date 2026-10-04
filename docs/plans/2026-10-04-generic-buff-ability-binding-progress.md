# 通用 buff → 技能呈現事件綁定

## 本批決策與實作

- 接續 OpenSpec `build-unreal-rust-moba-framework` 的 2.2b；完整進度仍為 20/30，不把相容層遷移的一部分當作封關。
- 問題：bridge 的 `add_saika_buff_ability_event` 只認得兩個硬編碼 buff／技能 ID。新英雄即使取得通用 C++ 事件 API，仍無法使用同一個 buff lifecycle → ability cue 流程。
- 決定：Lua `ue.buff_visual.ability_binding` 宣告呈現綁定；codegen 產生 manifest 的型別化 `ability_binding`。bridge 使用同一個 Rust metadata 型別並建立 numeric buff ID 索引，`add_buff_ability_event` 不再判斷英雄／技能名稱。
- `mode = "toggle"`：added／refreshed／updated → ToggleOn，removed → ToggleOff。`mode = "transform"`：對應 TransformStart／TransformEnd。保留原 lifecycle 的 instance key、payload、tick、sequence 與 remaining duration；`multi_shot_count` 由 Lua 明確宣告，預設 0。
- 無綁定的 buff 不會憑名稱猜出技能事件。未知模式／未知欄位／負數 count／空技能／缺少技能引用／不一致 buff numeric ID／重複綁定拒絕；不退回 Saika 特例。這是呈現 metadata，不會改寫技能玩法或重新施法。
- 既有 sniper_mode 與 three_stage 的綁定移到 templates/buffs.lua，原能力 ID、事件種類和三連擊 count 維持相容。

## 新英雄使用方式

在該 buff 的 Lua 定義加入以下宣告；ability_id 必須是同一份有效 catalog 中已宣告、未 tombstone 的技能。

```lua
ue = {
  buff_visual = {
    ability_binding = {
      ability_id = "custom_stance",
      mode = "transform",
      multi_shot_count = 7,
    },
  },
}
```

沿用既有 Lua 建置入口重新生成及建置，不需新增角色 C++ 或 Blueprint graph。

## 本功能確認

- codegen 新增兩個直接測試：Lua metadata 生成／未知技能引用拒絕；非法 metadata 拒絕。2/2 通過。
- bridge 新增兩個直接測試：任意 custom_buff／custom_stance 在 toggle、transform 的四種 lifecycle 共八個組合；未綁定忽略、錯誤 catalog 拒絕。2/2 通過。
- 原 `snapshot_projection_publishes_catalog_aligned_buff_animation_and_script_cues` 更新 fixture 為明確 Lua 綁定，1/1 通過。
- 既有 codegen 25 個測試在新增直接測試前通過；沒有再跑全套 bridge／runtime／server／UE 驗收。
- 正式 Lua 執行 codegen 及只讀 --check 均 exit 0：11 files／16 Lua inputs，content_hash `ed9aaf23a003ec5d`。

## 尚未完成的邊界

- C ABI 11 佈局不變。`OmAbilityEvent.saika`／`OmSaikaAbilityProjection` 是既有相容欄位名稱，這批通用事件仍透過它承載既有 Unreal 通用 payload 所讀取的欄位；不宣稱已刪除 typed legacy API。
- bridge 動畫 overlay 的 sniper_mode／three_stage 分支、未知 hero fallback 及已保存 Blueprint 的 HandleSaikaActionEvent graph 尚未遷移；2.2b 保持未勾選。
- 本批已刷新生成來源與 manifest，但沒有 staging bridge DLL、重啟 Editor 或驗收新內容 hash 握手。最終整合建置必須同步 host／script DLL／generated catalog，不能把目前開啟 Editor 當新版本證據。
- 下一步：將動畫 overlay 選擇改接 Lua priority／binding；再以 MCP 依資產 graph 實際引用遷移 legacy handler，最後統一完整驗收。
