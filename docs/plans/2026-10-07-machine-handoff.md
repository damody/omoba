# 換電腦接續紀錄（2026-10-07）

本次只整理與上傳，不恢復遊戲開發、不開新場次。先前程式與文件已在主庫master（2e142ea3）及正式submodule遠端；本次補上交接資料、生成產物忽略規則及可跨機器clone的Unreal HTTPS URL。不刪本機檔案、不上傳DLL／EXE／target／dump／log／capture／私密sentinel、IDE生成檔、MCP測試資產或無關Publisher文件。

## 新電腦取得原始碼

建議保留原路徑 D:\code\omoba，避免不必要的舊路徑差異。先安裝Git與Rust工具鏈、Unreal及所需Visual Studio C++／Windows SDK；Rust版本固定1.95.0，host與script DLL必須一致。Lua固定工具鏈已入Git，不另加PowerShell／Python fallback。Grok帳號／CLI與登入由新機器自行設定，不透過Git複製憑證。

```bat
git clone --recurse-submodules https://github.com/damody/omoba.git D:\code\omoba
cd /d D:\code\omoba
git submodule status
rustup show active-toolchain
```

若是既有clone：先確認沒有未保存變更，再git pull --ff-only，接著git submodule sync --recursive與git submodule update --init --recursive。不要用reset --hard。omfx只是歷史submodule，不編譯／不維護。omfue是唯一正式前端，BpGeneratorUltimate在其Plugins內。

不要搬舊機器Binaries、Intermediate、Saved、target、DLL或Unreal BuildId；新機器完整重建。Unreal本機EngineAssociation GUID可能不同，使用明確--ue-root指向實際引擎（原機D:\UE5.8），不要手改manifest偽造相容。

```bat
tools\lua\lua.exe scripts\build_ue_moba.lua --build-only --ue-root D:\UE5.8
tools\lua\lua.exe scripts\run_moba_pie_ue.lua --help
```

build入口建置restart、編譯並部署script DLL／bridge與OmGame；不是正式release後端/runtime的完整建置證明。恢復對局前依正式launcher／stage契約補齊其release產物；--no-build只允許已有一致產物，不能在乾淨clone直接當作初始化。首次重建／MCP連線應先做，最後才進入新場次。

## 開發停止點與接續優先順序

- OpenSpec build-unreal-rust-moba-framework仍27/31。待4.4、6.2、6.4、6.5；不可為換機而勾完成。
- Lua只在工具／build-time生成Rust與Unreal C++，沒有gameplay Lua runtime；單人正式路徑一個外部Rust replica，不讓Unreal再跑第二個模擬。
- 已修Bot零input correlation購物回執漏frame：先consume／verify FIFO後省略無UI回執，7項局部回歸通過。修正版v5長局越過旧約4分鐘中斷點；不是完整驗收。
- 原同世界選角→gameplay、真人買／賣（Transaction #2 settled(code0)）、攻擊移動、Q升級L2、死亡重生已有實際畫面／後端證據。使用者暫停後v5 success=false、cleanup_verified=true，原三程序已清理；自然勝負UI尚未完成。
- 下一步先定位真人R lumen_mend tick83725：輸入與AbilityCast accepted，但DLL execute返回RErr，不能宣稱治療／CD成功。已提交dispatch詳細RErr log；局部四技能headless cast/settlement測試1通過，未重現原場問題，也未部署該log至已停止的v5。
- 不要無改碼重跑效能：固定十二門檻結果仍FAIL（server／client／UE尖峰），不可放寬／挑樣本；畫面3FPS與simulation60Hz需分開，不冒稱渲染60fps。
- 真雙機LAN仍缺兩台實機證據。換機後可能具備條件，但不能把同機loopback當作LAN。
- 前一執行批實際8場／最多10；不要因換機或分批重置計數規避上限，下一次若續同批最多再2場。純fixture／程式編譯不算真場次。

詳細歷史：2026-10-06-moba-completion-execution.md、2026-10-06-same-world-pie-handoff-progress.md、防錯紀錄E347／E348；.ai-collab/state.json與review.md保存Grok歸屬／取消／成本。舊任務packet可能包含歷史狀態，以最新paused checkpoint及本交接為準。Grok runtime session／bridge資料不入Git；新機器應用新job，不假裝resume本機已不存在的worker。

## 上傳安全與操作錯誤

omfue原SSH遠端Host key verification failed。已使用同一GitHub倉庫HTTPS確認master=88f8023，不關閉SSH驗證、不搬ssh key／token、不修改全域Git安全設定。為新機器可clone，.gitmodules採HTTPS；submodule新增忽略規則先commit／push成功，主庫才更新gitlink。既有所有已追蹤程式與進度遠端原本已同步，本次不重新提交他人歷史。

一次列出全部raw evidence造成過量輸出，之後只取副檔名／數量摘要，絕不讀私密sentinel或大dump內容。原始證據與Publisher檔留原機，不被刪除；需要原capture診斷時另作安全離線移轉，Git只存已審查摘要。
