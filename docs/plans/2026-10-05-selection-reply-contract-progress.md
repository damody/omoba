# 通用原生選角回覆與介面生命週期

## 計畫與實作決定

1. 將原生選角 JSON admission 從 Slate widget 抽為共用 FOmSelectionReplyModel；先形成完整候選資料，全部通過才提交。沒有英雄白名單、角色專屬 C++／Blueprint、runtime Lua 或第二份選角規則。
2. 依現有 Rust SelectionReply／HeroSelectionSnapshot 契約驗證 protocol／schema、精確 revision、request correlation、hash／目錄與座位身分。目錄非空且 ID 唯一、座位英雄在 catalog；所有 player ID 正整數 u32／唯一，team 1／2，admitted seat 為真人。ready 與全體 locked 相符、finalized 必須 ready。
3. revision 不經 double：canonical ASCII u64 token，按長度與字典序排除回退，保持相同 revision 的錯誤／read 回覆合法；不從命令種類猜 revision 增量。
4. 成功接受後保留快照供 RebuildWidget 使用；Stop 清快照、列表與協定狀態，Start 重設狀態訊息／smoke stage。介面重建不重新解析／送命令／產生結果檔。
5. 新增獨立 Om.Runtime.SelectionReplyContract 原生 automation，涵蓋 arbitrary hero、分數／溢位／零／重複玩家、非法 team、unknown hero、Bot admission、矛盾 ready、schema、non-object catalog、duplicate hero、u64 上限／精確性、舊 revision／相同 revision、錯誤 correlation、hash 改變與失敗原子性。

## 當前確認

- 僅執行 OmRuntime／OmGenerated／OmEditor scoped compile，不啟動整體 Editor／遊戲／LAN 驗收。
- scoped native 首輪成功：9 actions／11.57 秒，Result: Succeeded。新解析器／widget／獨立原生測試及兩個模組連結均成功；沒有編譯失敗。日誌：`omfue/Saved/Logs/native-selection-reply-contract-modules-20261005.log`。
- 原生 assertion 僅已編譯、未執行；現有 E224 引擎／project BuildId 不相符限制仍保留，不繞過或重試整體 Editor。這是目前功能編譯與接線確認，不宣稱按鈕／像素／完整對局驗收通過。
- 不部署／stage binary、不改生成器版本、Lua catalog hash、C ABI 或 IPC framing；不維護 omfx。
- OpenSpec strict validation 與主 repo／omfue whitespace check 均成功。選角 widget 與新 helper／test 是既有未追蹤原生來源的一部分，普通 git diff 不顯示它們；以保存來源及實際 scoped compile 確認，沒有 stage 或 commit。
- 問題与防錯紀錄 E242。OpenSpec 完整 6.2 仍未完成，21/31 保持；LAN 多真人選角與選角到正式對局仍需後续實作／最後驗收。
