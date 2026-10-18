# 換機交接：MOBA／Unreal 前端

更新日期：2026-10-18（Asia/Taipei）。新電腦請先讀本文件，再讀 `AGENTS.md`。本次將已實作的主倉庫、後端 `omb`、Unreal `omfue` 與其內層 `BpGeneratorUltimate` 以中文提交至各自遠端 `master`；版本以本次主倉庫的 submodule gitlink 為準。

| 倉庫 | 本次功能提交 |
|---|---|
| `omb` | `f043858bd0d88d0f79d05d469b2bb224410aea99` |
| `omfue` | `44890d4d7222eea5f90c9a1e656e98a81294e9d0` |
| `omfue/Plugins/BpGeneratorUltimate` | `dfdbd34dabf1b70fd4c221baffeec0bb61942dd4` |

主倉庫版本請執行`git rev-parse HEAD`，避免在文件中寫入自身commit造成循環。

## 目前狀態

完整 MVP **尚未完成**。OpenSpec `complete-moba-controls-and-presentation` 共44項，**27完成／17待辦**；規格 ready 不是功能全數完成。詳細清單在 `openspec/changes/complete-moba-controls-and-presentation/tasks.md`，功能核對在 `docs/plans/2026-10-18-mvp-functional-task-audit.md`。

| 已接線的範圍 | 內容與接受界線 |
|---|---|
| 鏡頭／小地圖 | 左鍵定位與拖曳、滾輪、邊緣平移、Space／Y、隊友focus、視窗框及小地圖偏好；實際owned viewport事件已驗。 |
| 操作設定 | 可保存／遷移／恢復的快捷鍵、精確修飾鍵、A確認瞄準、QWER三施法模式、升級／自用、六格道具、D/F、回城；WASD、游標鍵與搖桿映射沿共用命令。搖桿使用合成軸，非實體控制器接受。 |
| 權威契約 | Rust側Stop／Hold、攻擊偏好、輸入模式、召喚師、ward、商店receipt／undo及隊伍溝通的共享資料／驗證已接線；需要新真對局ACK的父項仍待驗，不能只因按鈕存在就勾完。 |
| HUD／商店 | 七位正式英雄頭像、slot圖示／rank／冷卻／charges／tooltip、隊友卡／計分板／屬性／FPS與RTT、商店搜尋／分類／recipe／receipt、死亡／回城／結算與輸入focus。 |
| 溝通 | team／all chat、八語意ping／輪盤、快捷訊息、表情、標記、mute與公開資料隔離。 |
| 場景／角色 | 兩張公開三路map的地面、lane／river／jungle、阻擋輪廓、塔／基地、角色衣裝與裝備、共享team／selection標記、camera fit與光照生命期。完整6.2／6.4及世界畫面矩陣仍待收尾。 |
| 動畫／聲音 | 公開cue／HP／回城狀態驅動的原生動畫、池化VFX；原創UI／戰鬥／ambient音效，四段CC0英文播報，獨立master／music／effects／voice設定與字幕。 |
| 工具 | Lua 5.4工作流、擁有權程序清理、正式Engine bootstrap／NoEngineChanges建置、native／Blueprint驗收、實際submix PCM，以及可選的owned視窗MJPEG AVI錄影。 |
| BpGeneratorUltimate | UE5.8相容、MCP／PIE與工具修正；移除啟動時自動歡迎／新聞彈窗，手動入口保留。 |

## 最近一次完整驗收

可攜摘要：`docs/acceptance/2026-10-18-handoff/index.json`。原始接受報告與來源指紋、八張錄影原始JPEG及兩張已審世界PNG也保存在同目錄。

- ordinary NoEngineChanges／local／NoUBA／nocache建置成功，最後入口20秒。
- **197項完整原生成功**，12個既有warning保留；另明確執行2項editor專項，全部成功／零warning，不加算為198項完整原生。
- **458個實際視窗事件案例／222 wire intents／157 control stages／13個實際游標觀察**。wire admission不是權威ACK。
- **69個既有Blueprint**編譯／載入零error、零warning，資產SHA與mtime未變。
- **五份真實submix PCM**通過；沒有宣稱實體喇叭聽感已驗，也沒有把PCM當成AVI音軌。
- **1197份來源／資產及保護mtime**核對一致。441份已擷取Rust／ABI輸入相同，明示沿用之前真實通過的1168 Rust測試／五checks／九個既有ignored及ABI30；不是此次重新跑Rust。
- 錄影有4811張取樣影格／11個原尺寸片段，約138 MiB；本機原始經過797.450秒，封裝8fps播放約601.375秒，真實時間看JSONL。八張原始JPEG已直接看過。**這是黑底synthetic filtered-feed輸入fixture，不是正式整場世界錄影或美術接受。**

本次交接只整理版本與可攜性，不把既有驗收改稱新電腦重新通過。換機後必須重新建置與驗收。

## 新電腦取得版本

```bat
git clone --recurse-submodules --branch master https://github.com/damody/omoba.git
cd omoba
git submodule sync --recursive
git submodule update --init --recursive
git status --short
git submodule status --recursive
```

已有clone時先保存當地變更，再對主倉庫`git pull --ff-only`並更新submodule。不要各自把submodule拉到任意最新HEAD；主倉庫gitlink才是這次可重建組合。submodule處於detached HEAD是正常狀態。歷史`omfx`不編譯、不修復、不納入驗收，也不刪除。

## 工具鏈與建置

- Windows；Rust固定 **1.95.0**。host與`base_content.dll`必須同rustc，不能只升級其中一邊。
- 本機引擎是 **UE5.8.4**，位於`D:\UE5.8`，沒有隨Git上傳Engine或其build cache。新電腦需自行準備相容引擎。
- 本機成功的C++工具鏈：VS2022 Community、MSVC14.44（工具鏈14.44.35215／目錄14.44.35207）、Windows SDK10.0.22621.0。依實際可用工具鏈診斷，不修改Engine manifest／BuildId冒充相容。
- Lua固定用clone內`tools\lua\lua.exe`；不要新增PowerShell／Python／shell fallback。根目錄只維持既有四個薄bat。
- Unreal專案是`omfue\om.uproject`。DLL／EXE、Binaries、Intermediate、Saved、DDC、target與個人設定不搬移，應在新電腦重建。

在**實際clone根目錄**執行（引擎路徑依新機調整）：

```bat
rustup toolchain install 1.95.0
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-executor local --disable-action-cache --max-parallel-actions 1 --prefetch-shared-pch
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --verify-staged-only
```

第一次遇到正式EngineChange時，走同一Lua入口加`--refresh-engine`，完成後再跑一次不含refresh的普通建置。若存在`omfue/Saved/BuildTemp/source-engine-bootstrap.json`，必須完成正式bootstrap流程，**不手刪marker、不改Engine／UBT／共享PCH**。原機限制並行1是驗過的保守基準；快電腦可在確認RAM／磁碟餘裕後使用工具支援的`--max-parallel-actions 1..64`，不可因此放寬驗收門檻。

`run_ue.bat --td --editor --no-build --dx12`可開既有editor路徑。正式MOBA入口是`run_ue.bat --moba`，但它會啟動真對局，請先遵守下節的對局限制；本次交接未另開對局。

## 新機驗收與限制

```bat
tools\lua\lua.exe scripts\moba_renderer_fixture.lua build
tools\lua\lua.exe scripts\ue_local_acceptance.lua --ue-root D:/UE5.8 --run-id 新的唯一ID --native-rhi d3d12 --record-video
cargo test --manifest-path omb/Cargo.toml -p omobab
cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi
cargo test --manifest-path scripts/Cargo.toml -p base_content
```

Lua完整本機驗收不新增權威對局，但需要先有正式建置與本機renderer fixture執行檔。原始log／PCM／AVI會在新機產生，不應從舊機cache補假成功。其餘Rust／ABI驗證細節與五checks請參考保留的Rust報告及功能計畫。

本次交接另以四個倉庫的暫存區建立全新checkout（明確啟用`core.autocrlf=true`），**1472個素材／授權／receipt／匯入producer／固定Lua runtime輸入逐一SHA相同**，四個根目錄bat保持CRLF／無BOM。證據為`docs/acceptance/2026-10-18-handoff/fresh-checkout-sha256.json`；這是可攜性驗證，並非新機完整建置。

**既有權威對局批次已10/10耗盡。** 換電腦不會重設額度；不要另開match、改10/10計數、換入口繞過或用synthetic fixture冒充真對局／ACK／LAN／效能。若沒有合適執行條件，保留待驗。

## 剩餘17項：依原計畫繼續

1. **1.2／1.3**：既有遊戲入口的固定視點與actual controller／pawn／view target／focus／hit-test；native及原Blueprint action owner的PIE／重連矩陣。
2. **3.1／3.3／3.4／3.5**：右鍵、精確點擊／AttackMove、Stop／Hold、英雄限定與autoattack的真權威ACK及行為矩陣；不要重寫已完成接線來冒充新功能。
3. **3.9**：ward完整權威charges／生命期／team／fog與實際對局接受。
4. **4.1／4.3／4.4／4.7**：完整HUD／objective版面、指令成功與拒絕回饋、商店權威流程，以及字體／palette／縮放與多resolution／DPI接受。
5. **6.2／6.4／6.6**：所有正式角色／單位可辨識性、缺失動畫與VFX收尾；正式出生、lane、多人combat、fog、HUD、shop的固定視點before-after PNG與事件影片。現在的黑底輸入錄影不能替代正式世界影片。
6. **7.1／7.3／7.4**：全action跨native／原Blueprint的實際事件→有效camera／原ACK／visible UI；OS DPI／window mode／PIE／重連矩陣；有條件才做真match／LAN／固定效能。

建議先依`docs/plans/2026-10-18-mvp-functional-task-audit.md`與實際來源核對缺口，優先做能直接改善可玩性的世界、動畫與HUD部分；各項已實作但欠證的部分要明確區分，不因父項未勾就再造另一套架構。

## 延續工作規則

- 依使用者最新指示，**自行實作，不使用子代理或Grok**；不接續`.ai-collab`舊active job。先列計畫直接做，遇到問題自行判斷通用解，不反覆要求確認。
- 錯誤必須寫入`docs/plans/unreal-moba-error-register.md`；本次交接前最後編號E1299，新增前先檢查檔頭避免重號。
- build、Rust、native或asset consumer尚活著時，不改production／test來源。工具回傳session代表還在跑，等真正退出並核對來源後才修正。
- 改submodule時先在最內層commit／push，再讓上層更新gitlink；不可只推父repo而漏推子commit。
- 素材／授權與SHA守衛保留原始byte；已補`.gitattributes`避免資產與匯入producer的checkout行尾轉換。原始授權空白不隨意美化；既有UE兩處程式尾端空格、Rust三檔EOF空行及tasks.md尾空格也不為交接重寫已驗收來源，沒有宣稱全量diff-check零診斷。
- `.ai-collab`調查／草稿、log／trace／錄影／cache與`graphify-out`不搬移；已追蹤的歷史AI協作紀錄保留，可攜接受證據以`docs/acceptance/2026-10-18-handoff/`為準。內含原始報告的`D:\omoba`等路徑只代表舊機來源，不是新機執行指令。
- 沒有提交Engine、私有登入設定、API key、建置產物或個人對局profile；不需要Git LFS才能取得本批素材，新增單檔小於GitHub的100MiB限制。
