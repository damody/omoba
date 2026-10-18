# Unreal 共享 PCH 的可選唯讀預讀

普通本機建置可使用：

```bat
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-executor local --disable-action-cache --max-parallel-actions 1 --prefetch-shared-pch
```

適用於大型既有共享 PCH 位於慢速儲存、compiler 正在等待 mapped input I/O 的情況。選項預設關閉；新 clone 沒有 response/PCH 時正常繼續原建置，不下載或生成 PCH。

來源只取 project 及 plugin Intermediate/Build 內既有 MSVC response 的 /Fp，依 Engine/Source 解析相對路徑、正規化並限定 Engine/Intermediate/SharedPCH*.pch。最多4096 response、16層目錄、每份 response 1MiB、預讀總額8GiB；不跟隨 directory symlink。以1MiB分塊唯讀，保留每檔 bytes／elapsed，驗前後大小及mtime。這是 I/O 預讀，不能取代正式 stage SHA 或內容驗證。

計畫與實作：先純路徑／去重／預算／變動拒絕測試，再普通 workflow 實測，最後回到戰鬥設定 owned IPC 與 Blueprint 相容性。建置資源限制仍為每次呼叫參數，NoEngineChanges／NoUBA 保持原門檻。

實測：`.ai-collab/pch-prefetch-build-v1/summary.json` exit0，build.log記錄既有PCH 2,574,581,760 bytes／1,672ms、零budget skip；ordinary NoEngineChanges／NoUBA／nocache增量建置成功4.05秒。暖快取資料不能當作冷磁碟效能保證。15項純契約與既有ordinary resource-cap測試均通過；未修改引擎檔案。
