# M3' 設計：v3 產生 Rust、每位英雄一個 DLL、熱加載

建立日期：2026-10-29。取代 [m3-ir-design.md](m3-ir-design.md)。決策見 README 的 D15、D16。

## 1. 為什麼改

使用者 2026-10-29：「用 rust 就是要極致的性能跟編譯器的檢查能力」、「一個英雄生成一個 dll，然後用熱加載，這樣編譯快，也可以像腳本一樣 runtime 驗證」。

| | 執行期 IR（舊方向） | 產生 Rust（新方向） |
|---|---|---|
| 效能 | 每個指令都要經過直譯迴圈、token stack、開放的 JSON map | 原生程式碼，可以 inline，沒有直譯成本 |
| 檢查 | 載入時才發現寫法不支援（ir-blockers 那 534 個） | 引用錯數值、狀態、屬性時 rustc 編不過；寫法不支援時產生器在建置期就失敗 |
| 改一位英雄 | 只改資料，不用編譯 | 重編那一位英雄的 DLL（目標是數秒到數十秒），熱加載 |
| runtime 驗證 | 重開 server | 熱加載後馬上跑該英雄的情境 |

不變的部分：v3 文件仍然是唯一的作者資料；數值單一來源（D5）；說明、網站、TooltipRenderer 都照舊讀 catalog 的 values 與 text；規則層（M2）；conformance 與重播比對。

## 2. 產生的程式碼

### 2.1 crate 配置

```text
scripts/
  script-abi/            host 與所有 DLL 共用的 ABI（不變，只放 abi_stable 友善型別）
  ability_kit/           產生的程式碼呼叫的共用函式庫（rlib，每個 DLL 靜態連結）
  base_content/          塔、兵、野怪、魔王等共用單位（保留）
  heroes/<hero>/         每位英雄一個 cdylib crate → hero_<id>.dll
    src/lib.rs           產生：manifest、技能註冊
    src/generated/       產生：values.rs、statuses.rs、<ability>.rs（不手改）
    src/hooks/           手寫：D7 的例外（只讀 generated 的常數）
  items/                 商店道具 → items.dll（道具數量多，之後可以依類別再拆）
```

- 全部是 `scripts/Cargo.toml` workspace 的成員，共用 `target/`，所以 `ability_kit` 只編一次。
- 產生器在 cargo build 之前寫入 `src/generated/`。檔頭標「由 v3_rust.lua 產生，不要手改」。**不進版控**（使用者 2026-10-29）：每次建置前產生，`.gitignore` 排除 `src/generated/`。

### 2.2 數值與型別

產生器從 v3 的 `values` 產生具型別的常數。數值在建置期由十進位字串轉成 Fixed64 raw，不經過浮點數：

```rust
// generated/values.rs（示意）
pub mod c01_ground_flame {
    use ability_kit::{Fixed64, Rank4};
    pub const COOLDOWN: Rank4 = Rank4::from_raw([/* 14, 13, 12, 11 */]);
    pub const DAMAGE:   Rank4 = Rank4::from_raw([/* 200, 300, 400, 500 */]);
    pub const STUN:     Rank4 = Rank4::from_raw([/* 0.5, 1, 1.5, 2 */]);
}
```

- 註解裡的數字只是這份文件的示意。實際產生的程式碼裡，不能有手寫的數字（value-single-source §5）。
- 狀態 id、cue id、子技能 id 產生成 `enum`。StatKey、DamageType、狀態旗標用 `ability_kit` 從規則層 vocab 匯出的 `enum`。
- 引用不存在的數值或狀態時，rustc 就會報錯；產生的 crate 開 `#![deny(warnings)]`。

### 2.3 技能

每招技能產生一個實作 `AbilityScript` 的 struct：v3 的 behavior 轉成直線的 Rust 程式，再呼叫 `ability_kit` 與 `GameWorldDyn`。

```rust
// generated/c01_ground_flame.rs（示意）
impl AbilityScript for GroundFlame {
    fn execute(&self, ctx: &mut CastCtx<'_>) -> Outcome {
        let r = ctx.rank();
        ctx.delay(values::DELAY.at(r), Step::Burst);   // 排程器在 host，狀態不放在 DLL
        Outcome::ok()
    }
    fn on_step(&self, ctx: &mut StepCtx<'_>, step: Step) { /* Burst：範圍選取 → 傷害 → 暈眩 */ }
}
```

- v3 的運算式轉成 Rust 運算式（Fixed64 運算），條件轉成 `if`，`for_each` 轉成走訪 `ability_kit` 選取結果的迴圈。選取一律依 entity id 排序。
- 延遲、重複、periodic 這類非同步動作，由 host 的排程器保存「下一步」（step 列舉加上參數），所以 DLL 不保存狀態（§4）。
- 產生器不支援某個寫法時，建置期就失敗並指出那一行 v3；產生器接受的欄位不可以默默略過，不支援就回報到 coverage。`v3_ir_blockers.lua` 由 `v3_codegen_coverage.lua` 取代，列出「產生器還不支援的寫法」與解鎖順序（沿用 ir-blockers 的排序：M5，然後 M6 第 1～9 批）。

### 2.4 共用與 hook 的分界（使用者 2026-10-30）

- **2 招以上類似 → 共用**：機制在 2 招以上（英雄技能與道具合計）出現類似的寫法，就做成 v3 詞彙，產生器翻譯，元件放 `ability_kit`。「類似」以機制判斷，不看參數：例如兩招都是「衝刺到點、路徑上觸發效果」就算同一種，即使速度、效果不同。
- **完全跟大家不同 → hook**：只有 1 招用到、而且無法用既有詞彙組合出來的，才寫 hook。v3 在該招標 `hook = "<方法名>"`，產生器為這招產生一個 trait（`on_cast`、`on_attack_hit`、`on_status_event` 等方法，與這招的 values、status、cue 常數），由手寫的 `src/hooks/<ability>.rs` 實作。沒實作的方法編不過，不再產生「施放時回報缺什麼」的 stub。
- **hook 的規則**：只能引用產生的常數與 `ability_kit`，不能寫數字，不能保存狀態（`static mut`、thread local）；檔頭寫明為什麼不能共用。v3 仍要寫這招的行為描述或預期結果，conformance 才能產生情境、比對 hook 的實作。
- **判定依據**：`v3_codegen_coverage.lua` 的「用到的招式數」。一種寫法只有 1 招用到，標為 hook 候選；之後若第 2 招也用到，就把 hook 改成共用，原本的 hook 刪掉。

### 2.5 共用元件 `ability_kit`

產生的程式碼只呼叫這一層，不直接散寫 world 呼叫，好讓產生器保持簡單、行為一致：

| 類別 | 內容 |
|---|---|
| 選取 | 圓形、扇形、直線、全圖；隊伍、單位類型、魔免／無敵預設排除（E226）；依 entity id 排序 |
| 效果 | 傷害（走規則層四段管線，帶 flags，例如 `no_amplify`）、治療、法力、施加／移除狀態、驅散 |
| 時間 | delay、repeat、periodic（排程器在 host） |
| 空間 | 投射物、dash、leap、擊退／拉扯、地面區域、樹木 |
| 單位 | 產生單位、召喚物、分身、光環 |
| 呈現 | cue（只送事件給呈現層，不影響結算） |

規則層與各種「每單位、每 tick」的系統（BuffStore、forced_motion、ability_projectile、reveal_zone、tether）留在 host。`ability_kit` 只是 DLL 這一側的薄包裝。

## 3. 每位英雄一個 DLL

### 3.1 載入

- 每個 DLL 匯出同一種 `Manifest`（`scripts/script-abi/src/manifest.rs`）：units、abilities，以及新增的 `id`（英雄 id）與 `content_hash`。
- host 依 launch manifest 載入一組 DLL，不再只掃描單一的 `base_content.dll`。這組 DLL 是 base_content、items 和本局選到的英雄；開發模式可以全部載入。
- **要先做的 spike**：abi_stable 的 `RootModule::load_from_file` 有 per-type 的 static 快取（`declare_root_module_statics!`），同一個 `Manifest_Ref` 型別可能只能載入第一個 DLL。現行 `omoba-core/src/scripting/loader.rs` 的 `load_scripts_dir` 雖然會走訪目錄裡每個 DLL，實際上只用過 base_content 一個。要先確認怎麼用 `RawLibrary`／`LibHeader` 載入多個同型別的 root module，以及 layout 檢查要怎麼做。
- 同一個 rustc 要求（AGENTS.md）不變：host 與所有 DLL 都用 `rust-toolchain.toml` 的 1.95.0。

### 3.2 launch 與版本

- `moba_role_launch` 現在把 base_content 複製到 `<run>/artifacts/scripts/`（E124），改成複製整組 DLL，並寫一份 `scripts-manifest.json`：每個 DLL 的檔名、英雄 id、content hash、檔案 hash。
- E125／E190 的防呆照舊適用：每個 DLL 內嵌 `OM_CONTENT_CATALOG_DATA_HASH` marker，與 authority 的 content hash 不一致就拒絕啟動；二進位比 dep-info 舊也拒絕。
- lockstep 握手與重播檔要記下整組 DLL 的 hash，authority 與 replica 不一致就拒絕開局（fail closed）。

### 3.3 編譯時間

- 改一位英雄只重編 `hero_<id>`。`ability_kit` 與 `script-abi` 沒改就不重編。
- release profile 現在是 `lto = "thin"`、`codegen-units = 1`，適合正式版。開發用的熱加載另外設一個 profile，例如 `dev-hot`：較低的最佳化、較多的 codegen-units，換取編譯速度。正式對局與效能量測一律用 release。
- 產生器只重寫內容有變的檔案，避免 cargo 因為時間戳記而全部重編。E206 的教訓也要遵守：被改到的 package 和依賴它的 package 都要一起重新產生。

## 4. 熱加載（開發用）

```text
改 v3 文件或 hook
  → tools\lua\lua.exe scripts\hero_hot_reload.lua --hero <id> [--run <run 目錄>]
      1. 只產生這位英雄的 Rust（v3_rust.lua）
      2. cargo build -p hero_<id> --profile dev-hot
      3. 複製成新檔名 hero_<id>.<generation>.dll 到 run 目錄（原檔不鎖，cargo 隨時能重建，同 E125）
      4. 經 debug-query 通知 server 與各 client runtime 載入新版
      5. server 在 tick 之間切換註冊表；重播紀錄寫下「第 N tick 換成 hash X」
      6. 跑這位英雄的 conformance 情境，輸出結果
```

- **狀態都在 host**：BuffStore、排程器的「下一步」、狀態變數、冷卻、充能都在 host，DLL 不保存狀態，所以換掉程式碼不會遺失狀態。產生的程式碼裡不能有 `static mut` 或 thread local，lint 會檢查。
- **不卸載**：abi_stable 不支援卸載 library，舊版留在記憶體裡。開發用 server 這樣可以接受；載入次數太多時重開即可。
- **replica 一起換**：authority 與所有 replica 都要在同一個 tick 換成同一個 hash，否則就 desync。split 本機開發時由 `hero_hot_reload.lua` 一起通知；正式對局不熱加載。
- **step 列舉的相容性**：新版的 `Step` 如果刪掉或改了排程中的步驟，切換時那些排程要取消並記錄下來，不要用新版的程式碼去解讀舊的參數。

## 5. 遷移（M3'）

| 步驟 | 內容 | 驗收 |
|---|---|---|
| ① DLL 拆分與載入 | spike：同一型別載入多個 root module；把 base_content 先拆出一位已上架英雄（例如稻姬）成 `hero_a02.dll`；launch 改成依 manifest 複製整組；握手與重播檔記下 DLL hash | 實機局 ≤10 分鐘正常結束；hash 不一致會被拒絕 |
| ② 熱加載 | `hero_hot_reload.lua`；server／replica 在 tick 之間切換；dev-hot profile | 改稻姬一個數值、熱加載之後，情境結果跟著變；換版前後，重播在換版 tick 之前的部分每 tick 都相同 |
| ③ 產生器 | `v3_rust.lua`：values、statuses、技能、manifest；`ability_kit` 第一批（M5 那 8 位英雄用到的寫法） | 產生的 crate 在 `deny(warnings)` 下能編譯；`v3_codegen_coverage.lua` 產生報告 |
| ④ 換掉 IR | B01、C17、A02，以及 B04 的兩招，改由產生的 Rust 執行。這幾招和 Rust handler 比對重播（每 tick 相同）之後，刪掉 `omoba-core/src/runtime/native/ir.rs`、`omoba-content-model/src/ir.rs`、`scripts/base_content/src/ir_switch.rs`、`tools/ir_probe`，以及 catalog 裡的 `ir` 欄位（`v3_compile.lua` 的 `EMIT_IR`） | 5 局實機重播每 tick 相同；`run_10000` 不退步；live 沒有 IR 直譯器 |

之後的 M5、M6、M7 都用這套流程：產生 Rust → 與 handler 或原作情境比對 → 上架。

### 5.1 IR 工作中可以沿用的部分

- **Lua 端**：`v3_compile.lua` 的驗證、型別檢查、衍生值，以及 `v3_ir.lua` 已經整理好的樹狀結構。產生器可以直接走訪這棵樹輸出 Rust，所以它是編譯器的內部表示，不會出貨。
- **Rust 端**：IR 直譯器裡呼叫 host 元件的那些對應（例如 `query_enemies_in_range`、`ally_area_restore`、`tether`、`reveal_zone`），以及揭露政策、法術增幅、選取排除魔免這些規則，改寫成 `ability_kit` 的函式。
- **測試**：conformance harness、情境產生器、`ir_replay_matches_rust_handlers_every_tick` 這類重播比對，把「IR 版」換成「產生的 Rust 版」就能沿用。

### 5.2 停止時還沒放回的 IR 工作

handover 第 3 節列出的 m3f（踏火無痕）、地裂與精準之光的 IR 切換，**不要再放回**。這幾招直接在 M3' ③ 之後改成產生的 Rust。影子目錄裡的資料修正（例如 handler 補上排除魔免）如果照原作是對的，可以單獨放回。

## 6. 風險

| 風險 | 對策 |
|---|---|
| abi_stable 同一型別不能載入多個 DLL | M3' ① 先做 spike。不行的話，每個 DLL 用各自的 root module 型別（由產生器產生），或者在 host 自己用 `RawLibrary` 載入 |
| 97 個 DLL 的載入時間與記憶體 | 只載入本局選到的英雄；量測開局時間 |
| 跨 FFI 呼叫的成本 | 「每單位、每 tick」的工作留在 host 批次處理；量測 `run_10000` |
| 熱加載造成 desync | 只在開發模式使用；同一個 tick 一起切換；hash 寫進重播檔；狀態都在 host |
| 產生器與手寫 hook 不同步 | hook 只能引用產生的常數；常數改名時 hook 就編不過 |
| 編譯時間仍然太長 | dev-hot profile；只重編改到的 crate；必要時把 `ability_kit` 拆小 |
