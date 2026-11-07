# 來源英雄索引摘要（D1／D2）

由 `tools\lua\lua.exe scripts\hero_source_index.lua index` 生成，不要手改；完整資料在 `source-catalog.lua`。

- 來源：https://github.com/damody/history_nobu @ `f966720b0956d002099d72aabec7ddc94b16d4bf`
- 只以文字讀取，未執行來源內任何腳本（`read_hero.lua`、`langtext.lua`、vscripts）。
- 每筆 ability／unit／tooltip 都帶 `path`、`line`、`start`／`stop`（原檔 byte offset，含 BOM）、`key_token`、`raw` 與 `raw_sha256`；`verify` 子命令逐筆回讀原檔比對。

## 檔案與英雄數

| 陣營 | `[ABC][0-9][0-9].txt` 主檔 | 預期 | `_2` 檔 | 其他 |
|---|---|---|---|---|
| A_Oda | 36 | 36 | 36 | 0 |
| B_Unified | 36 | 36 | 36 | 0 |
| C_Neutral | 25 | 25 | 25 | 1 |
| 合計 | 97 | 97 | 97 | |

主檔英雄數 97 與 36／36／25 一致。

`_2` 檔不是另一套技能組：它們是 `DOTAHeroes` 單位定義（模型、攻擊、`Ability1..N` 技能欄）。主檔才是 `DOTAAbilities` 技能定義。英雄數只算主檔；`_2` 與 `_old`／`_20` 技能另列版本，不混算。

- 英雄列：97；有單位定義（`_2`）：97；`langtext.lua` 有繁中名：97；其中網站名單註解掉（未上架）：22。
- 未上架：A09（蜂須賀政勝）、A10（平手政秀）、A16（鳥居元忠）、A18（稻葉一鐵）、A23（宇喜多秀家）、A33（本多正信）、A35（島左近）、A36（前野長康）、B03（齋藤朝信）、B07（足利義昭）、B12（山本勘助）、B18（湖衣姬）、B22（太原雪齋）、B23（伊勢姬）、B27（小島貞興）、B34（武田勝賴）、B36（直江兼續）、C03（島津義弘）、C12（大友宗麟）、C13（龍造寺隆信）、C16（果心居士）、C18（小早川隆景）。仍完整索引，選角時需注意資料可能未維護。

## 技能條目

ID 文法：`<英雄><核心>[_20][_old]`。`kind`：`skill`＝單一 W/E/R/T/D/F；`sub_ability`＝以槽位字母開頭的衍生招（例 `A28TW`、`A13W2`）；`auxiliary`＝其他輔助塊（`_HIDE`、`_DATA`、`B16MM*` 等）；`precache` 不分類。`variant`：`current`、`old`（`_old`）、`_20`（來源未說明含義，只照實標記）。

| kind | 數量 |
|---|---|
| auxiliary | 12 |
| precache | 97 |
| skill | 843 |
| sub_ability | 18 |

| variant | 數量 |
|---|---|
| _20 | 50 |
| current | 549 |
| old | 371 |

- 技能（skill＋sub_ability）共 861；有繁中描述 532、缺描述 329。
- 出現在 `_2` 的 `Ability1..N` 技能欄：430。
- 含 `RunScript` 呼叫來源缺少的 `scripts/vscripts/...`：2／861。這些技能的實際行為有一部分只存在於缺失的 Lua，KV 與描述只能說明意圖，不能冒稱已還原。

## 粗分類（D2）

依 KV（`AbilityBehavior`、動作塊、modifier 屬性）與繁中描述關鍵字標能力族；每個 tag 都附 evidence（`kv_behavior`／`kv_block`／`kv_modifier`／`tooltip_text`）。只出現在描述關鍵字的 tag 是弱證據，D3 必須逐招閱讀確認。

- 全部技能 861：classified 816、**unclassified 45**。
- 其中 current 版 444：classified 430、**unclassified 14**。
- auxiliary 塊 12：classified 12、unclassified 0。
- 證據來源分布（技能）：kv+text 387、kv_only 363、none 45、text_only 66。

| 能力族（README §5） | 全部技能 | current |
|---|---|---|
| `damage_heal_shield` 指定傷害／治療、資源、護盾 | 357 | 194 |
| `passive_attack_modifier` 普攻強化、被動、暴擊、吸血 | 340 | 187 |
| `shape_displacement` 圓形、扇形、直線、位移 | 349 | 221 |
| `channel_dot_area` 持續引導、DoT、地面區域 | 176 | 99 |
| `projectile` 飛行直線／追蹤投射物 | 95 | 56 |
| `multi_stage_toggle` 多段／重施放／蓄力／切換 | 26 | 14 |
| `summon` 召喚物 | 49 | 37 |
| `stealth_detect_control` 隱形／偵測／控制 | 262 | 159 |
| `push_pull_knockup` 推拉、擊飛、衝鋒、牆體互動 | 14 | 13 |
| `clone_multiform` 分身、多形態／多技能組 | 19 | 16 |
| `copy_rewind_terrain` 複製／偷技能、回溯、任意地形改造 | 7 | 6 |

current 版 unclassified：`A05E`（三河之勇）、`A33E`（鬼軀之步）、`A36EW`、`A36R`（苦楚秘釀）、`B01T`（三段）、`B14D`（暫夜）、`B22W`（自然屏障）、`B24T2`、`B36D`（履及）、`C04W`（神速之舞）、`C06EW`、`C07T2`（移動雷雲）、`C18R`（謀計．參刀剜心）、`C18T`（策謀之論．肆砲碾沙）。

## 來源 ID → 本地遊戲 ID

規則：match by zh-TW hero name; equal folder number alone never maps。完整 97 列在 `source-catalog.lua` 的 `id_mapping.entries`；以下只列非 `unmapped` 的列。

| 來源 ID | 來源名稱 | 狀態 | 本地 game_hero_id | 備註 |
|---|---|---|---|---|
| A02 | 稻姬 | `name_and_number_match` | `inahime` | identity match; kit version still has to be checked against the source variant (D3) |
| A34 | 阿松 | `name_and_number_match` | `omatsu` | identity match; kit version still has to be checked against the source variant (D3) |
| B01 | 雜賀孫市 | `name_and_number_match` | `saika_magoichi` | identity match; kit version still has to be checked against the source variant (D3) |
| B02 | 風魔小太郎 | `conflict_number_only` | — | **衝突**：本地 `scripts/base_content/src/heroes/B02_date_masamune`（伊達政宗）與來源同編號但不同人，禁止按編號配對 |
| B04 | 伊達政宗 | `name_match_number_differs` | `date_masamune` | same hero name, local folder number B02 differs from source id |
| B08 | 淺井長政 | `name_and_number_match` | `asai_nagamasa` | identity match; kit version still has to be checked against the source variant (D3) |
| B29 | 村上義清 | `name_and_number_match` | `murakami_yoshikiyo` | identity match; kit version still has to be checked against the source variant (D3) |
| C15 | 玉子 | `name_and_number_match` | `tamako` | identity match; kit version still has to be checked against the source variant (D3) |
| C17 | 阿市 | `name_and_number_match` | `oichi` | identity match; kit version still has to be checked against the source variant (D3) |

統計：`conflict_number_only` 1、`name_and_number_match` 7、`name_match_number_differs` 1、`unmapped` 88。

## 診斷與來源異常

- `game/dota_addons/nobu/scripts/npc/A_Oda/A13.txt` 的根 key 是 `DOTAHeroes`（預期 `DOTAAbilities`），內容仍是技能定義，照常索引。
- `game/dota_addons/nobu/scripts/npc/C_Neutral/C19_2.txt` 的根 key 是 `DOTAHeros`（預期 `DOTAHeroes`），照常索引。
- 帶 UTF-8 BOM 的檔案：20（offset 一律含 BOM）。
- `game/dota_addons/nobu/resource/addon_tchinese.txt` 有 81 個重複 token key；以第一次出現者為準。UTF-16 的 `addon_tchinese.txt` 未使用（與來源網站程式一致，取 `*2.txt`）。
- 解析診斷 `duplicate_ability_id`：`game/dota_addons/nobu/scripts/npc/A_Oda/A11.txt`:649 `A11E_old also in game/dota_addons/nobu/scripts/npc/A_Oda/A11.txt:177`
- 含未解析 `%placeholder`（AbilitySpecial 與頂層欄位都沒有同名變數；多為引擎內建值，如 `%attack_damage`）的條目：92；名單見 catalog 的 `unresolved_placeholders`。

