# 持續選角服務：Rust JSON-line 入口

## 計畫與決定

1. 補上 6.2 前置選角入口，不再只依賴一次性的 `--lock-plan` 自動鎖定。
2. 共用 `SelectionService` 持有既有 Rust kernel；沒有 World、Lua VM、socket 或角色專屬規則。UI 可提交普通 JSON，不自行驗證英雄白名單。
3. `moba-config --selection-session FILE.json ADMITTED_PLAYER_ID` 由可信主機在啟動時綁定既有真人席位；訊息不能攜帶另一個 player ID。這不是網路認證。
4. 初始與後續回覆包含 compiled hero catalog、selection snapshot、綁定身分、錯誤與 request ID。每則回覆立即 flush。
5. 所有請求檢查 protocol_version=1、完整 catalog_data_hash、expected_revision；read／select／lock／finalize 共用同一持續狀態。只有明確 finalize 成功才輸出 plan；EOF 不自動鎖定或開局。
6. 嚴格 JSON schema 拒絕未知欄位／action。每行最多 16 KiB（包含換行）；超長 frame 回報後終止，避免尾端被當新命令。無換行的 EOF 殘段不執行。一般 JSON 錯誤不更動狀態，下一行仍可使用。
7. 一個程序只綁一位真人。其他真人未鎖定時 finalize 仍拒絕；尚未提供多連線共享大廳，不用多個獨立程序冒充共享狀態。

## 通訊例子

啟動後先讀取第一行，取得 catalog_data_hash 與 revision。後續範例中的 HASH 必須替換為服務實際回報值：

```json
{"protocol_version":1,"catalog_data_hash":"HASH","request_id":1,"expected_revision":0,"action":{"kind":"select","hero":"training_ranger"}}
```

`read` 不增加 revision；其他成功變更增加 revision。每次以最新回覆的 revision 再送下一個命令。`lock`、`finalize` 的 action 僅有 kind 欄位。完成後 read 仍可取得 finalized 狀態，但不重新輸出或再次完成配方。

## 當前確認與剩餘工作

- 首輪服務局部 3/3 通過；正式 compiled-content-only 的 moba-config cargo check 成功。新增第四項確認非法 JSON 後可繼續且不能替另一位真人鎖定，最後結果補於下方。
- 最終服務功能測試 4/4 成功；OpenSpec strict validation 與主 repo／omb whitespace check 成功。沒有執行全套驗收或對局。
- Unreal 選角 UI／程序管理與正式開局串接仍待；既有自動準備 launcher 保留，沒有偷偷改成阻塞等待輸入。
- E177 共享引擎建置限制仍存在；未 stage、未啟動 Unreal、未跑完整對局驗收。6.2 不勾選，整體仍 21/31。
