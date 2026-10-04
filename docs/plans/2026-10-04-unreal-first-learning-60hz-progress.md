# 雙 Unreal 首次學習：60Hz 驗收增量

## 計畫與決策

- 沿 OpenSpec `build-unreal-rust-moba-framework` 的 5.3／6.2 繼續，19/30 全項不提前勾選。
- [x] 新增獨立 `OMOBA_UE_FIRST_LEARN_SMOKE=1`；與舊 rank upgrade 互斥，沿既有 server-owned bindings 使用純 Lua `training_apprentice`。
- [x] 共用 Unreal Ctrl+Q 正式綁定驗 rank0→1，四槽初始0、技能點出生1＋正常升級。沒有新英雄 C++／Blueprint graph，不直接寫 world 或信任 transport ACK。
- [x] runtime internal FIRST_LEARN／UPGRADE／COMBAT 均強制0，避免誤算 Unreal 操作；共用非Shipping native smoke 只呼叫唯一正式 Ctrl+Q delegate 一次，不宣稱 OS 鍵盤注入。
- [x] Lua觀測器增加first-learning參數與正反向回歸，舊rank1→2保留。保存verifier依明確first_learning契約分流，raw protobuf逐snapshot對authority ranks／XP／SP，精確UI前後tick、原upgrade只accept一次、private input與四槽CD0核對。
- [x] runtime 61 passed／8 opt-in ignored；Lua upgrade observation與process graceful stop回歸通過。舊run1791100483保存capture再驗2954／2949 snapshots，原input2一次通過。
- [x] 固定Lua build-only：OmGameEditor完整編譯／C ABI9不變，bridge SHA `6d8382d466fc3a4d935f8e6b19625825dc8652e0bad791aa0f655b3a26aefe29` stage通過，Lua hash仍 `3b296ff5bdbcd7d9`。
- [x] 修正共用WorldBridge的cast／learning ID混用：cast ID在rank0仍為0，learning ID保留合法catalog。未知技能／HUD未ready仍拒絕，沒有直接改rank或繞過正式輸入。
- [x] 最終full建置／stage SHA `f84a14e78ec4d7c9ca37bfa39549fd892e3982e7cc5a161bc04d92e59a2a9ca3`、MCP11資產gate、同Editor兩輪19/19通過；新增rank0實際Slate文字與apprentice_lance first-level6門檻回歸。owned Editor81376已正常退出並獨立inspect不存在。
- [x] 真實release雙Unreal60Hz首次學習對局、保存證據重驗、五PID退出。
- [x] 檢視前後四張實際backbufferPNG，區分HUD資料驗收與E121尚未封關的裁字問題；後者仍存在，沒有宣稱完整UI像素封關。

## 問題處理

E124記錄了錯誤patch anchor導致E0425、Windows字面glob與猜測不存在檔名。已修正scope後重跑runtime，查程式位置使用rg既有目錄，不建立新workflow fallback。

第一輪run1791103763已開始，同profile scripts release LTO編譯中；沒有把無即時console輸出判定為卡死。只讀process確認rustc1.95仍在编譯該owned Cargo子程序，繼續等待，不啟動重複對局。

後續第一輪正式delegate各一次卻queued0，未把callback當提交成功。曾懷疑pointer／HUD輸入消耗，但第二輪1791104221診斷explicit0／pointer0仍失敗，已撤回那個production變更。真正程式阻擋為rank0清除可施法ID，升級入口卻複用同欄位；新增獨立OwnedAbilityUpgradeIds。兩輪原始失敗證據與report保留，十個owned PID另驗退出，不在同個失敗run重送繞過測試。

第三輪1791104474玩法與raw驗收成功：5,914 live snapshots exact ranks／XP／SP、原input2各一次；保存verifier因名稱desired120／117／119超出內寬116拒絕。共用技能格寬改成實際name desired＋12 padding、min128，既有短名保留最小寬。這是新內容的確定layout不足，不是E121的已充分layout卻偶發文字截短；沒有降低verifier門檻。

## 最終真實證據

最終adaptive width版本再次full建置與MCP11BP gate（compile-1791104879）通過；Editor105396同session兩輪19/19通過，正常退出並獨立inspect不存在。codegen shipped --check、OpenSpec strict與相關main／omfue diff --check通過；stage仍f84a14…，沒有留下owned Editor／game程序。未commit／push或清理使用者既有工作樹。

Run：`target/interactive-runs/interactive-ue-1791104682`，release server＋兩個runtime＋兩個Unreal `-game`，network60Hz／D3D11；沒有runtime自動升級，route=`unreal-bound-CtrlQ-delegate`。

- p1：before tick2342，level3／XP80／SP3／rank0；正式input2 observed2344；after tick2348，level3／XP105／SP2／rank1。XP在操作期間正常增加25，沒有凍結或偽造。
- p2：before tick2818，level4／XP11／SP4／rank0；正式input2 observed2820；after tick2824，level4／XP11／SP3／rank1。
- 每队 KEY callback／submitted／result／complete 都各一次；原upgrade input2在raw accepted inputs精確一次，未分享敵方private input。
- raw／IPC live snapshots：p1 2,971、p2 2,961，共 **5,932**。四槽rank與level／XP／SP逐筆核對原始authority exact tick；只有Q0→1，其他槽保持0，全部CD0。前後native HUD日志tick精確匹配snapshot。
- launcher三方50／50 PASS rows至3000；保存verifier去重後每隊25個unique checkpoint，post-upgrade p1六／p2兩個，last3000且零repair／observer-external frame hash一致，沒有把重複PASS算不同tick。
- 五owned PID52792／84372／48896／21308／8096，在cleanup後另用固定Lua process.inspect全部確認不存在。
- `unreal-upgrade-verification-report.json` success=true；記錄原始log／PID／checkpoint／四PNG與四raw capture SHA256，驗證前後不變。舊rank1→2 run1791100483也用新版保存verifier重驗success。
- 四張1280×720實際backbuffer已檢視：兩隊Q從L0到L1／SP各扣一；仍有部分英雄／技能名、計分板、文字的局部截短，E121維持未封關。不是同tick GPU fence，也不能從FPS60單圖聲稱穩定60FPS。

## 重驗入口

```text
tools/lua/lua.exe scripts/verify_ue_upgrade_run.lua interactive-ue-1791104682
```

重開對局明確設定 `OMOBA_UE_FIRST_LEARN_SMOKE=1`、`OMOBA_UE_UPGRADE_SMOKE=0`、`OMOBA_UE_STEP_FPS=60`、有界 `OMOBA_UE_SMOKE_SECONDS=60`，執行 `tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane`。一般遊戲不啟用smoke flag，launcher不覆寫主game.toml。修改C++後先使用既有build入口，skip UE build仍有stage gate。

## 下一步

首次學習正式delegate網路增量封關；不是完整選角、四技能按鍵施法、三英雄Bot／三路／兩台LAN或性能基線。下一個功能驗收可接R首次學習後正式按鍵施法，沿同一組cast／learning ID分離，驗原cast／正冷卻與效果；已排除的E121假設不得再盲重試。

## 驗收界線

這輪只學Q，不自動施放傷害招；因此核對CD保持0，不冒充學後技能傷害或正冷卻驗收。上輪runtime真實KCP已驗學後自補施法／正冷卻，但那不是Unreal按鍵來源，不能合併冒稱同一條操作路徑。

Unreal可能在對局開始後才連上，因此首次學習允許正常兵線XP已提升hero level，要求學前SP等於level（出生一點＋每級一點），學後精確扣一且包含期间正常level delta；不得硬改等級來湊fixture。

原始證據保留在target/interactive-runs，不提交二進位／capture／log。全文錯誤見docs/plans/unreal-moba-error-register.md E124；完整三路／Bot／LAN／UI與效能大項仍待後續。
