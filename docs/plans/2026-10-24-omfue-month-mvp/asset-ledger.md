# 美術資產點數 ledger

英雄 3D 模型（Tripo）與參考圖／材質（Grok）的每一次生成都記在這裡。新紀錄加在表格最下面。

規則（使用者指示）：

- Tripo 以省點數為原則：預設選最便宜的版本與選項；某一步省不掉（例如只有某版本能綁骨或匯出可用模型）才用較貴選項，並寫明原因。
- 不購買點數、訂閱或任何付費方案；不改帳號設定。點數用完就停下回報。
- Grok 不需省，以品質為準（只記錄，不設上限）。
- 不為了挑選多生候選；確定不能用才重生，並寫下原因。

## Tripo 定價（實際在網站上看到的數字）

2026-10-27 在 studio.tripo3d.ai「3D 工作台 → 模型 → 高精度模型」逐項切換選項，看生成按鈕上的點數（帳號 nobugg01，切換前 7455 點）。單張圖與多視角（前／左／右／後，訂閱專享）價格相同。

| 選項 | 點數 |
|---|---|
| 只生幾何（關閉紋理生成），H3.0 或 H2.5 | 15 |
| + 紋理 2K | +10（25） |
| + PBR | +5 |
| 紋理 4K（取代 2K） | +10 |
| 8K 貼圖 | +10 |
| 四邊面拓撲（取代三角面） | +5 |
| AI 模型 H3.1「最高质量」（取代 H3.0／H2.5） | +15 |
| 去光處理、面數控制（三角面 500–2,000,000） | 不加價 |

網站預設（H3.1、4K、PBR、8K 開）是 65 點。英雄採用 **25 點**：H3.0、紋理 2K、去光開、PBR 關、三角面 30,000、多視角（正＋背）。紋理不能省：遠鏡頭辨識靠配色。綁骨／動畫的點數見紀錄表。

注意：第三方評測提到 Tripo 免費方案的輸出不可商用、需 Professional 方案才可用於遊戲。實際授權條款要在網站上確認，記在下方「待決定」。

## 紀錄

| 日期 | 服務 | 資產 | 選項 | 消耗點數 | 剩餘點數 | 檔案 | 備註 |
|---|---|---|---|---|---|---|---|
| 2026-10-27 | Grok Build CLI（image_gen，訂閱制） | saika_magoichi 全身正面 A-pose 參考圖 | aspect 2:3，淺灰單色背景 | 不另計費 | — | scratchpad `tripo/grok/saika_magoichi-a1/saika_magoichi_front.png` | 一次可用，外觀與頭像／載入圖一致 |
| 2026-10-27 | Grok Build CLI | saika_magoichi 側面、背面 | 同上，以正面為外觀依據 | 不另計費 | — | `saika_magoichi-{side,back}-a1/` | 可用；側面手臂自然下垂（側視本來看不出 A-pose 外展） |
| 2026-10-27 | Grok Build CLI | 其餘 7 位正／側／背 21 張 | 同上 | 不另計費 | — | `<hero>-{front,side,back}-a1/` | 9 張退回重生（見下） |
| 2026-10-27 | Grok Build CLI | 重生 9 張 | prompt 加「背景滿版、無白邊」，側面加「雙手自然下垂、不朝前伸」 | 不另計費 | — | `<hero>-<view>-a2/` | 退回原因：oichi 正／背、murakami 正／背、date 正、omatsu 背、asai 背有白邊或內框；asai 側、date 側手臂朝前伸直。重生後全部可用 |
| 2026-10-27 | Tripo（studio，image-to-3D） | saika_magoichi 模型 | 多視角（只上傳正＋背：側面圖手臂下垂、槍托位置與正背不一致）、H3.0、2K、去光、PBR 關、三角面上限 30,000 | 25 | 7430 | 任務 `525d1fe9-8c4d-4695-97fa-25f68cec4d29` | 一次可用：29,258 面／18,345 頂點，步槍、紅頭巾、紅圍巾、黑紅具足都在 |
| 2026-10-27 | Tripo（自動綁定） | saika_magoichi 骨架 | 人形，骨骼預設 UE5 Mannequin（另有 ActorCore／Mixamo／VRM 1.0／Unity Humanoid） | 20 | 7410 | 同任務 | 骨頭名稱為 UE5 Mannequin（root、pelvis、spine_01…），FBX 61 個 LimbNode |
| 2026-10-27 | Tripo（匯出） | saika_magoichi FBX | FBX（Mixamo 版）、紋理 2K、匯出骨骼 | 0 | 7410 | `TripoV1/saika_magoichi/saika_magoichi.fbx`（下載為 zip：FBX＋`.fbm/` basecolor PNG，貼圖也嵌在 FBX 內） | 匯出不扣點；FBX 另有 Blender／3dsmax 版 |
| 2026-10-27 | Tripo（image-to-3D＋自動綁定＋匯出） | date_masamune | 同 saika：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；人形 UE5 Mannequin；FBX Mixamo 版 | 25＋20＋0 | — | 任務 `23c9cfe0-df0d-4f82-9504-eb48a3b84004` | 一次可用：29,230 面；眼罩、月牙前立、紺色具足 |
| 2026-10-27 | Tripo | omatsu | 同上 | 25＋20＋0 | — | `7067e150-01c7-407a-8e10-66de5948c03f` | 一次可用：28,776 面；淺藍和服、扇子 |
| 2026-10-27 | Tripo | oichi | 同上 | 25＋20＋0 | — | `e2562af8-1250-4154-8b40-72cf59f2ab9a` | 一次可用：28,638 面；粉白振袖、長髮。綁骨約 2 分鐘，比其他人久 |
| 2026-10-27 | Tripo | inahime | 同上 | 25＋20＋0 | — | `f0fae2b4-706f-4684-96f5-df54d4527b77` | 一次可用：28,277 面；綠衣、背弓（弓弦有重影，遠鏡頭看不出） |
| 2026-10-27 | Tripo | asai_nagamasa | 同上 | 25＋20＋0 | — | `ec69d301-d067-471d-a59f-a7c40b9ebb4f` | 一次可用：29,260 面；藍黑具足、白披風。第一次按生成沒送出（沒扣點），重新上傳後才成功 |
| 2026-10-27 | Tripo | murakami_yoshikiyo | 同上 | 25＋20＋0 | — | `18aa9d5a-0bd9-4fdc-9c7b-8996b8e827e3` | 一次可用：28,441 面；赤具足、背斧 |
| 2026-10-27 | Tripo | tamako | 同上 | 25＋20＋0 | 7095 | `90d1c6e2-2ed6-4061-9288-061b52d49770` | 一次可用：28,626 面；紫色短和服、背箭筒 |

| 2026-10-27 | 建置期上色（lua-host `png_recolor`，不用 Tripo／Grok） | oichi 變體 `sakura` | manifest `recolor`：色相 330–15° 且飽和度 ≤ 0.45、亮度 ≥ 0.5 的粉色，以及飽和度 ≤ 0.08、亮度 ≥ 0.7 的白色，都改成色相 345°（緋紅粉），飽和度 0.45–0.5 起跳 | 0 | 7095 | `/Game/Omoba/Characters/TripoVariantsV1/oichi/`（貼圖＋複製的材質）；原始貼圖與英雄資產不動 | 遠鏡頭下和阿松的白衣混淆。改了 2048² 中的 293 萬個像素，金色腰帶、黑髮、膚色都不動。前後對照：`.ai-collab/tripo-recolor/omatsu_oichi_before_after.png` |

| 2026-10-27 | Grok Build CLI | 塔、主堡、抑制塔參考圖（各 1 張） | aspect 1:1，45° 俯視三分之二視角，米白瓦片與旗幟（之後做陣營上色） | 不另計費 | — | `omfue/SourceArt/Structures/TripoStaticV1/<id>/reference/` | 一次可用 |
| 2026-10-27 | Tripo（image-to-3D，單張圖，不綁骨） | tower_yagura、castle_keep、shrine_gate | H3.0、2K、去光、PBR 關、三角面 30,000；FBX 匯出 | 3 × 25 = 75 | 5520 | 任務 `bf4870c5…`、`3fd7d7d9…`、`a07bb0f9…` | 一次可用：27,943／27,364／27,589 面。**開始前帳號是 5595，不是上次結束時的 7095**：有 1500 點被別處用掉（資產列表多了家具類模型），不是本 session 花的 |
| 2026-10-27 | 建置期上色 | 3 個建築 × 友方／敵方 | 米白（飽和度 ≤ 0.22、亮度 ≥ 0.62）改成藍（215°）或紅（0°） | 0 | 5520 | `/Game/Omoba/Structures/TripoStaticVariantsV1/` | 每個建築改了約 1/4 的像素 |

| 2026-10-27 | Grok Build CLI | 近戰兵、遠程兵、攻城兵 正＋背各 1 張 | 2:3 A-pose，米白布料（之後做陣營上色），武器背在身上 | 不另計費 | — | `TripoV1/minion_*/reference/` | 遠程兵背面第一次畫成 T-pose，重生 1 次（a2） |
| 2026-10-27 | Tripo（image-to-3D＋自動綁定＋匯出） | minion_ashigaru、minion_archer、minion_siege | 同英雄：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；UE5 Mannequin；FBX Mixamo 版 | 3 × 45 = 135 | 4890 | `b25fe845…`、`a09945e0…`、`cb057ce0…` | 一次可用。**開始前帳號是 5025，不是階段 1 結束時的 5520**：又有 495 點被別處用掉，不是本 session 花的 |
| 2026-10-27 | 建置期上色 | 3 種兵 × 友方／敵方 | 米白（飽和度 ≤ 0.22、亮度 ≥ 0.5）改成藍（215°）或紅（0°） | 0 | 4890 | `/Game/Omoba/Characters/TripoVariantsV1/minion_*/` | 超級兵直接用近戰兵放大（115 單位），沒有另外生成 |
| 2026-10-27 | Grok Build CLI | 河童、天狗、鬼 正＋背各 1 張 | 2:3 A-pose，中立配色（苔綠、赭黃、暗紫；天狗只有臉上的面具是紅的），武器背在身上 | 不另計費 | — | `TripoV1/jungle_*/reference/` | 一次可用 |
| 2026-10-27 | Tripo（image-to-3D＋自動綁定＋匯出） | jungle_kappa、jungle_tengu、jungle_oni | 同小兵：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；UE5 Mannequin；FBX Mixamo 版 | 3 × 45 = 135 | 4755 | `5afa88b6…`、`ade18bb4…`、`e30d1c8d…` | 一次可用，開始時 4890（和小兵階段結束時相同）。野怪不分陣營，不做上色變體 |
| 2026-10-27 | Grok Build CLI | 守衛石燈籠參考圖 1 張 | 1:1 三分之四俯角，深灰石材，紙窗和布條是米白（之後做陣營上色） | 不另計費 | — | `TripoStaticV1/ward_lantern/reference/` | 一次可用 |
| 2026-10-27 | Tripo（image-to-3D，單張圖，不綁骨） | ward_lantern（視野守衛） | 同建築：H3.0、2K、去光、PBR 關、三角面 30,000；FBX | 25 | 3230 | `22404334…` | 一次可用，28,188 面。原本守衛在 UE 是預設方塊。**開始前帳號是 3255，不是野怪階段結束時的 4755**：又有 1500 點被別處用掉，資產列表裡出現不是這項工作做的家具模型 |
| 2026-10-27 | Grok Build CLI | honda_tadakatsu（A07 本多忠勝）正、側、背參考圖 | 2:3 A-pose，黑具足、鹿角兜、念珠，長槍斜背在背上、雙手空出 | 不另計費 | — | `TripoV1/honda_tadakatsu/reference/` | 一次可用（a1） |
| 2026-10-27 | Tripo（image-to-3D＋自動綁定＋匯出） | honda_tadakatsu | 同英雄：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；人形 UE5 Mannequin；FBX Mixamo 版 | 25＋20＋0 | 3185 | `a7ec26fe-b0f9-4ed8-b96b-d1d178513907` | 一次可用：29,461 面；開始時 3230（與上次結束相同）。匯入 61 骨；retarget facing 180、參考姿勢誤差 4°、最大 18°（Pistol_Aim_Down） |
| 2026-10-27 | Grok Build CLI | honda_tadakatsu 頭像、載入圖、5 個技能圖示 | 頭像 1:1（1254 PNG）、載入圖 2:3、技能圖示 1:1 | 不另計費 | — | scratchpad `grok/portrait/honda_tadakatsu/`、`grok/out/honda_tadakatsu-a1/`、`grok/abil/a07_*/` | 退回 3 次：頭像 a1 畫中畫（人物在內框裡）；奮起 a1 有較淺的內框、a2 左右有暗色直條。重生後全部可用 |
| 2026-10-27 | Grok Build CLI | sanada_yukimura（B06 真田幸村）正、側、背參考圖 | 2:3 A-pose，朱紅具足、白毛皮披肩、白頭帶，十文字槍斜背在背上、雙手空出 | 不另計費 | — | `TripoV1/sanada_yukimura/reference/` | 一次可用（a1） |
| 2026-10-27 | Tripo（image-to-3D＋自動綁定＋匯出） | sanada_yukimura | 同英雄：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；人形 UE5 Mannequin；FBX Mixamo 版 | 25＋20＋0 | 3140 | `65587f92-d1dd-40ba-a9af-037986e7f260` | 一次可用：29,484 面；開始時 3185（與本多忠勝結束時相同）。背上的槍生成成兩支交叉（正背參考圖的槍方向不同），雙手空出、不影響動畫，沒有重生。匯入 61 骨；retarget facing 180、參考姿勢誤差 3°、最大 20°（Pistol_Aim_Down） |
| 2026-10-28 | Grok Build CLI | 8 位持武器英雄的空手正＋背參考圖（saika、date、murakami、asai、inahime、tamako、honda、sanada） | 以舊正面圖為外觀依據，去掉所有武器、刀鞘、箭筒；雙手張開空手、A-pose | 不另計費 | — | `TripoV1/<id>/reference/`（舊圖移到 `reference/armed_v1/`） | 一次可用 |
| 2026-10-28 | Grok Build CLI | 7 把武器參考圖（katana、matchlock、war_axe、yumi、hankyu、tonbokiri、jumonji） | 3:2 橫放側視，握把在左 | 不另計費 | — | `TripoStaticV1/weapon_*/reference/` | katana 第一次有白邊與內框、matchlock 畫成西式步槍，各重生 1 次 |
| 2026-10-28 | Tripo（image-to-3D＋自動綁定＋匯出） | 上述 8 位的空手身體 | 同英雄：多視角正＋背、H3.0、2K、去光、PBR 關、三角面 30,000；UE5 Mannequin；FBX | 8 × 45 = 360 | — | `9c7b65cb…`、`33458c14…`、`662a8bdc…`、`86c5f0fd…`、`062665cf…`、`7967f819…`、`b2569acf…`、`ef712a92…` | 使用者要求角色和武器分開生成（武器做進身體會讓 rig 和動畫壞掉）。8 位都一次可用、手上沒有東西 |
| 2026-10-28 | Tripo（image-to-3D，單張圖，不綁骨） | 7 把武器 | H3.0、2K、去光、PBR 關、三角面 10,000；FBX | 7 × 25 = 175 | 2525 | `f0664eb1…`（katana，伊達與淺井共用）、`b1925478…`、`844bbcc9…`、`c3f82414…`、`d724955f…`、`127d09d0…`、`f4446d5c…` | 一次可用。開始前帳號是 3060；這批合計 535 點（3060 → 2525） |
| 2026-10-29 | Grok Build CLI（image_gen，訂閱制） | M6 第 1 批：頭像 3（A01 德川家康、B27 小島貞興、C03 島津義弘）、技能圖示 19、特效 cue 參考圖 9 | 頭像與技能圖示 1:1；cue 參考圖 1:1、深灰底、只畫特效 | 不另計費 | — | `omfue/Content/Omoba/UI/gen-{hero,ability}-<id>.png`（256²）、`omfue/SourceArt/CueReferences/gen-cue-<id>.png`（512²）；出處與 prompt 在 `generated-art-v1.json` | 第一輪 31 張中 21 張可用。退回 10 張：內框／畫中畫 8（頭像 3、技能 5）、筆刷邊 1、代幣上像文字的符號 1。照 E092 改成正面描述滿版構圖、拿掉框的負面字，每張同時跑兩個候選（共 20 張），挑沒有框的那張；其中 4 張候選也有框或符文。這次為了少跑幾輪而同時生兩個候選，和上面「不為了挑選多生候選」不一致，第 2 批起改回一次一張 |
| 2026-10-29 | Grok Build CLI（image_gen） | M6 第 2 批：頭像 5（C09、B30、A24、A23、A35）、技能圖示 9、特效 cue 參考圖 4 | 同第 1 批 | 不另計費 | — | 同第 1 批（generated-art-v1.json 共 49 筆） | 第一輪 18 張中 16 張可用，一次只重生一張。c05_thunder_spear：a2 的水面漣漪畫成一塊平面又有框，a3 拿掉漣漪後可用。a16_blood_link 重生 4 次：兩個人影、兩顆心、8 字鏈都畫成內框；a5 把「深藍夜空」背景改成「暗紅黑雲霧填滿整張畫」才沒有框 |
| 2026-10-29 | Grok Build CLI（image_gen） | M6 第 3～9 批：頭像 18、技能圖示 102、特效 cue 參考圖 60 | 同第 1 批；技能圖示背景改成「暴風雲霧填滿整張畫」 | 不另計費 | — | 同第 1 批（generated-art-v1.json 共 229 筆；UI 156 張 21 MB、cue 參考圖 73 張 28 MB） | 第一輪 180 張中 150 張可用，一次只重生一張。連同重生共退回 34 次：內框或邊框 18、符文或像符號的紋樣 12、主題讀錯 3（臉、三角形、鐵砧）、Grok 未呼叫 image_gen 1。b25_finger_of_death、b15_multishot、c19_surprise_raid 各重生 2 次，改成「虛空或霧填滿整張畫」並拿掉次要物件後可用。每次退回的原因都記在索引的 rejected |
| 2026-10-30 | Grok Build CLI（image_gen） | 金袋參考圖 1 張（C06T 豪奪掉在地上的金袋，presentation identity `item_money`） | 1:1 三分之四俯角，棕色麻布袋、繩結、袋口和旁邊有金幣；背景鋪滿整張圖 | 不另計費 | — | `TripoStaticV1/gold_bag/reference/gold_bag_front.jpg`（sha256 `2cc9052f…`） | 第一張有內框，退回；第二張（背景鋪滿整張）可用。依主 session 指示不用 Tripo，這張只當造型參考 |
| 2026-10-30 | 引擎基本形狀（OmPropMeshCommandlet） | 金袋模型 `SM_gold_bag`，以及 3 個顏色材質 `MI_Prop_cloth`、`MI_Prop_rope`、`MI_Prop_gold` | `scripts/prop_meshes.lua` 定義 7 個零件：Sphere、Cylinder、Cone 組成袋身、袋口、繩結和金幣；材質是 `/Engine/BasicShapes/BasicShapeMaterial` 的 Color 實例 | 0 | — | `/Game/Omoba/Props/gold_bag/`、`/Game/Omoba/Props/Materials/` | 第一次是 constant 材質，渲染時還在編譯，看起來是預設格紋；而且法線重算後有稜角。改成 BasicShapeMaterial 實例並保留原法線後可用。world_units 的 `item_money` 使用它（高 30 單位） |
| 2026-10-30 | 引擎基本形狀（OmPropMeshCommandlet） | 變形外觀 `SM_hex_critter`（青蛙），以及 3 個顏色材質 `MI_Prop_frog`、`MI_Prop_eye`、`MI_Prop_pupil` | `scripts/prop_meshes.lua` 定義 10 個 Sphere 零件：身體、頭、眼睛、瞳孔、四隻腳 | 0 | — | `/Game/Omoba/Props/hex_critter/` | 一次可用。任何帶 v3 狀態 `hex` 的單位都換成這隻青蛙（OmStatusLook::ShowsHexForm），高約 70 cm 乘以單位的呈現比例 |
| 2026-10-30 | Grok Build CLI | hattori_hanzo（測試英雄服部半藏）載入圖 | 2:3（832x1248 JPEG，轉成 PNG） | 不另計費 | — | scratchpad `grok/loading/hattori_hanzo-a1/`；`omfue/Content/Omoba/UI/hattori-hanzo-loading-v1.png` | 一次可用：單一蒙面忍者，深藍頭巾只露眼睛，與 nobu 頭像一致；無文字、無邊框 |
| 2026-10-30 | Grok Build CLI | M6 第 1～4 批 33 位英雄的載入圖（kind=loading） | 2:3（832x1248 JPEG，轉成 PNG） | 不另計費 | — | scratchpad `grok/loading/<id>-a1/`；`omfue/Content/Omoba/UI/gen-loading-<id>.png`，`generated-art-v1.json` | 36 張中退回 3 張（風魔小太郎、明智秀滿、菊姬：畫成多人拼貼），重生成中 |
| 2026-10-30 | Grok Build CLI | M6 第 5～9 批 48 位英雄的載入圖＋第 1～4 批重生成 3 張 | 2:3（832x1248 JPEG，轉成 PNG） | 不另計費 | — | scratchpad `grok/loading/<id>-a1|a2/`；`gen-loading-<id>.png` | 50 張中退回 2 張（瑞龍院日秀畫成西式修女、望月千代女的分身畫成蒼白人物），重生成中；第 1～4 批的 3 張重生成後可用 |
| 2026-11-01 | Tripo（image-to-3D＋自動綁定＋匯出） | tokugawa_ieyasu（M6 第 1 批，tripo-order 第 1 位） | 單張正面圖（Grok）、H3.1、2K、去光、PBR 關、三角面、面數上限 10,000；人形綁骨 UE5 Mannequin；FBX Mixamo 2K | 25＋20＋0 | 25,075 | `3e2ba573-81a3-4c41-ae84-f6343fa0b868`；`TripoV1/tokugawa_ieyasu/` | 一次可用：9,715 tris、80 骨（H3.1 多了手指掌骨，auto characterization 會多出 LeftIndexMetacarpal 等骨鏈而對不上動畫庫；manifest 改寫明確的 21 條骨鏈，手指從 _01 開始）。開始前 25,120 點 |
| 2026-11-01 | Tripo | yamagata_masakage（第 2 位） | 同上 | 25＋20＋0 | 25,030 | `1adce7fd-7b50-4621-8774-5daebbdff1b4`；`TripoV1/yamagata_masakage/` | 一次可用：9,383 tris、80 骨；赤備具足，頭盔拿在左手 |
| 2026-11-01 | Tripo | okuma_tomohide（第 3 位） | 同上 | 25＋20＋0 | 24,985 | `dfa7ee32-fefd-455d-9f44-631342f47e2e`；`TripoV1/okuma_tomohide/` | 一次可用：9,345 tris、80 骨；黑具足、狼皮披肩 |
| 2026-11-01 | Tripo | sasaki_kojiro（第 4 位） | 同上 | 25＋20＋0 | 24,940 | `ec05329a-4a01-4979-ae03-977349866a5c`；`TripoV1/sasaki_kojiro（第/` | 一次可用：9,306 tris、80 骨；深藍羽織、長髮。綁骨進度條停住，重新整理頁面後綁骨已完成 |
| 2026-11-01 | Tripo | otani_yoshitsugu（第 5 位） | 同上 | 25＋20＋0 | 24,895 | `a660358f-342f-4e5e-bf98-afe4c9d7adc0`；`TripoV1/otani_yoshitsugu（第/` | 一次可用：9,766 tris、80 骨；藍頭巾、具足 |
| 2026-11-01 | Tripo | maeda_toshiie（第 6 位） | 同上 | 25＋20＋0 | 24,850 | `adcab301-d9af-4d27-92c2-02cbd6a9f8d3`；`TripoV1/maeda_toshiie（第/` | 一次可用：9,612 tris、80 骨；赤具足、長馬尾 |
| 2026-11-01 | Tripo | nohime（第 7 位） | 同上 | 25＋20＋0 | 24,805 | `6faec3a0-bbbc-4507-862a-2d8f0fa11301`；`TripoV1/nohime（第/` | 一次可用：9,551 tris、80 骨；紅色蝶紋打掛 |
| 2026-11-01 | Tripo | kojima_sadaoki（第 8 位） | 同上 | 25＋20＋0 | 24,760 | `749c3ffb-a8ad-456e-864e-294226dd85de`；`TripoV1/kojima_sadaoki（第/` | 一次可用：9,782 tris、80 骨；暗紅具足、背刀 |
| 2026-11-01 | Tripo | shimazu_yoshihiro（第 9 位） | 同上 | 25＋20＋0 | 24,715 | `8b0d037d-cf7f-4078-b5d9-c5d057438951`；`TripoV1/shimazu_yoshihiro（第/` | 一次可用：9,586 tris、80 骨；黑具足、背大錘 |
| 2026-11-01 | Tripo | maeda_keiji（第 10 位） | 同上 | 25＋20＋0 | 24,670 | `ccb5bcac-c4ac-48dd-96f8-8cc06394147a`；`TripoV1/maeda_keiji（第/` | 一次可用：9,431 tris、80 骨；橘色具足、紅髮、花紋披風 |
| 2026-11-01 | Grok Build CLI（image_gen） | 場景元件參考圖 7 張（崖壁直段、崖壁轉角、竹、闊葉樹、本陣泉水、商店、大將塔） | 1:1，三分之四俯角，「low-poly、形狀簡單、細節少」，背景鋪滿淺灰 | 不另計費 | — | scratchpad `grok/scene/<id>/`；`TripoStaticV1/<id>/reference/` | 7 張一次可用 |
| 2026-11-01 | Tripo（image-to-3D，單張圖，不綁骨） | cliff_straight | H3.1、2K、去光、PBR 關、三角面、面數上限 1,500；FBX 2K | 25 | 24,645 | `1b351cf9-1728-4066-8611-770ea7cc6f89`；`TripoStaticV1/cliff_straight/` | 1,360 tris。第一次匯出的貼圖名稱帶連字號（`low-poly_grassy…`），UE 匯入時被拒；匯出時把檔名改成 `cliff_straight` 重新匯出（0 點）後可用 |
| 2026-11-01 | Tripo | cliff_corner | 同上，上限 1,500 | 25 | 24,620 | `74867975-0e9a-4b1e-a937-f29e989dfd98` | 1,416 tris |
| 2026-11-01 | Tripo | bamboo_tree | 同上，上限 800 | 25 | 24,595 | `6dbda8c0-18fa-4dc9-b970-bbf9e1b71446` | 774 tris |
| 2026-11-01 | Tripo | broadleaf_tree | 同上，上限 800 | 25 | 24,570 | `2a72b4e4-3067-407d-bacd-d101ada03174` | 694 tris |
| 2026-11-01 | Tripo | general_tower（大將塔） | 同上，上限 5,000 | 25 | 24,545 | `86d5f473-253e-4fcc-b276-584b38272058` | 4,616 tris |
| 2026-11-01 | Tripo | fountain_honjin（本陣泉水） | 同上，上限 8,000 | 25 | 24,520 | `7967e9e8-898c-4f5b-b473-dde4b558180c` | 7,171 tris |
| 2026-11-01 | Tripo | shop_stall（商店） | 同上，上限 8,000 | 25 | 24,495 | `c2f679cc-aea9-49a1-babc-93178743d39a` | 7,313 tris |

7 位是先連續送出生成、再逐一綁骨，所以點數是交錯扣的，中間的剩餘點數不逐列記。7 位合計 315 點（7410 → 7095），與「7 × 45」相符。

參考圖目前放在 session scratchpad（`C:\Users\User\AppData\Local\Temp\claude\D--omoba\0d0543d3-d3f5-47ba-ad1a-fbfdc56e2338\scratchpad\tripo\grok\`），每位英雄採用的版本：saika a1；omatsu 正側 a1／背 a2；oichi 正背 a2／側 a1；inahime、tamako 全 a1；asai 正 a1／側背 a2；murakami 正背 a2／側 a1；date 正側 a2／背 a1。已複製進 `omfue/SourceArt/Characters/TripoV1/<hero_id>/reference/<hero_id>_<view>.png`，旁邊的 `.prompt.txt` 是最終版 prompt（a1 圖用的版本少了「背景滿版」與側面手臂兩句）。

### saika_magoichi 試點結果（2026-10-27）

- 點數：生成 25＋綁定 20＋匯出 0＝**45 點／英雄**（7455 → 7410）。其餘 7 位照同樣設定估 315 點。
- 匯入：61 骨、高 99.6 cm、2K basecolor 嵌入 FBX；UE auto characterization 直接認得 UE5 Mannequin 骨架（21 條 chain，不必手動指定），43 支動畫都直立。
- 尺寸由 runtime 依網格邊界正規化到英雄 120 world units，不必手調。
- 第一版 retarget 的手臂穿過身體（Tripo 朝 +Y，動畫庫朝 −Y，見 errors.md E122）。修正後重新 retarget，不需要重新生成，沒有花點數。
- 證據（修正後）：側面截圖 `.ai-collab/tripo-saika-sanity-1/idle_move_attack.png`；遊戲內 `.ai-collab/tripo-saika-sanity-1/ingame_far_camera.png`、影片 `target/spotlight-tripo-fixed/saika_magoichi/`。

### 8 位全部完成（2026-10-27）

- 總點數：8 × 45 = **360 點**（7455 → 7095）。每位一次就可用，沒有重生。
- 全部是 61 骨、約 100 cm。retarget 都自動做了 facing 180 修正，參考姿勢誤差 3–5°。`Om.Runtime.NativeHeroPoseSanity` 8/8 通過（omatsu 的 idle 改成 Idle_Loop 之後）。
- 證據：
  - 側面 idle／跑步／attack：`.ai-collab/tripo-pose-2/all_heroes_idle_move_attack.png`（每位的單張在 `.ai-collab/tripo-pose-2/<hero>/png/`）。
  - 8 人同框的 MOBA 鏡頭：`.ai-collab/tripo-pose-2/ingame_lineup_moba_camera.png`；影片在 `target/spotlight-lineup/saika_magoichi/`。

### 總結（2026-10-27 最終：英雄、三個非英雄階段與守衛完成後）

| 項目 | 資產 | Tripo 點數 |
|---|---|---|
| 英雄 | 8 位（生成＋綁骨） | 360 |
| 階段 1 建築 | 塔、主堡、抑制塔（只生成，不綁骨） | 75 |
| 階段 2 小兵 | 近戰兵、遠程兵、攻城兵 | 135 |
| 階段 3 野怪 | 河童、天狗、鬼 | 135 |
| 守衛 | 石燈籠（只生成，不綁骨） | 25 |
| **本工作合計** | 18 個模型，全部一次可用，沒有重生 | **730** |

- 帳號從 7455 降到 3230，少了 4225 點。其中 730 點是上表；其餘 3495 點（1500＋495＋1500）在階段之間被別處用掉，不是這項工作花的（見上表備註）。
- 陣營色與阿市的配色都在建置期上色，沒有花點數。Grok 是訂閱制，不另計費。
- 在 runtime 中的對應：
  - 英雄：每位使用自己的 Tripo 身體。
  - 建築（`TripoStaticV1`）有友方／敵方兩套變體。
  - 小兵有友方／敵方兩套變體。
  - 野怪不分陣營：天狗是一般營地與南北守衛，鬼是 epic，河童是營地小怪。
  - 視野守衛（moba_ward）是石燈籠，顏色依擁有者英雄的隊伍；原本是預設方塊（errors.md E151）。
- 尚未換成 Tripo 的：訓練用英雄（training_vanguard、training_luminary）仍是 CC0 模型；它們現在是 test-only，正式對局不會出現。TD 與訓練模式的 30 個類別仍沒有模型，不在 build 期「單位不可沒有模型」檢查的範圍內。

## Mixamo 動作

來源都是 mixamo.com（Adobe，帳號 damody），不花點數。下載設定：FBX Binary、Without Skin、30fps、Keyframe Reduction none。原始檔在 `target/mixamo-input/`（不進版控）；轉到動畫庫骨架後，由 `scripts/mixamo_clips.lua` 產生 `omfue/Content/Omoba/Characters/AddedClipsV1/`。Mixamo 的授權條款尚未在網站上逐條確認，記在下方「待決定」。

| 日期 | Mixamo 動作（檔名） | 卡片說明 | clip | 用途 |
|---|---|---|---|---|
| 2026-11-02 | T-Pose | 轉骨架用的基準姿勢 | — | `mixamo_to_library.py` 的參考 |
| 2026-11-02 | Great Sword Slash | Great Sword Slash 系列（C 下載） | Mx_Sword_Slash | 刀 |
| 2026-11-02 | Stable Sword Inward Slash | Inward Slashing With A Sword Standing Stable | Mx_Sword_Inward | 刀普攻 |
| 2026-11-02 | Stable Sword Outward Slash | Outward Slashing With A Sword Standing Stable | Mx_Sword_Outward | 刀普攻、暴擊 |
| 2026-11-02 | Withdrawing Sword | 拔刀（C 下載） | Mx_Sword_Draw | 居合 |
| 2026-11-02 | Standing Melee Attack Downward | Downward Attack With Axe | Mx_Axe_Down | 斧下劈 |
| 2026-11-02 | Standing Melee Attack Horizontal | Right To Left Attack With Axe | Mx_Sweep | 斧、槍橫掃 |
| 2026-11-02 | Great Sword Slash（存為 Great Sword Downward Slash） | Great Sword Downward Slash | Mx_Axe_Smash | 砸地 |
| 2026-11-02 | Bayonet Stab | Advancing And Stabbing With Rifle Bayonet Or Spear | Mx_Spear_Thrust | 槍突刺（轉 39 度、手腕傾 50 度）|
| 2026-11-02 | Thrust Slash | Thrust And Slash With Blade | — | 試過沒有採用：接觸時槍身是豎的 |
| 2026-11-02 | Upward Thrust | Upward Weapon Thrust | Mx_Spear_Upward | 槍上挑 |
| 2026-11-02 | Firing Rifle | Firing A Rifle While Standing | Mx_Rifle_Fire | 鐵砲射擊 |
| 2026-11-02 | Standing Taunt Battlecry | Battlecry With Axe | Mx_Battlecry | 吶喊、Buff |
| 2026-11-02 | Charge | Pointing Onward Charge | Mx_Charge | 衝鋒起手（只取前 100 幀） |
| 2026-11-02 | Standing Melee Attack Kick Ver. 1 | Kick With Right Foot Holding Axe | Mx_Kick | 踢 |
| 2026-11-02 | Throw | Throwing An Object From A Standard Pose | Mx_Throw | 投擲 |
| 2026-11-02 | Two Hand Spell Casting | Casting A Spell With Two Hands | Mx_Cast_Two | 雙手施法（只取前 75 幀） |
| 2026-11-02 | Standing 2H Magic Attack 03 | Two Handed Casting Spell Forwards Sustained | Mx_Channel | 引導（只取前 90 幀） |
| 2026-11-02 | Standing React Death Backward | Death Falling Backwards | Mx_Death_Back | 第二種死亡 |

## 特效模型（10 位演示英雄，預算 3,000 點）

使用者 2026-11-02 給 3,000 點，為這 10 位的技能特效建模；累計到 2,700 回報 main，到 3,000 停止。累計以這張表為準，不用帳號餘額的差額（帳號有其他人或其他工作在用）。

- 開工前帳號餘額（2026-11-03 02:40，帳號畫面）：**24,355 點**。
- 設定：單張圖、不綁骨、去光、PBR 關、三角面；小件面數上限 1,500，單件不超過 5,000；靜態網格。

| 日期 | 工具 | 模型 | 用途（招式） | 點數 | 累計 | 帳號餘額 | 任務 ID | 結果 |
|---|---|---|---|---|---|---|---|---|
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_war_hammer | 島津 R 不動戰錘（c03t_hammer） | 25 | 25 | 24330 | `f1995d71-040b-4603-afde-4c7e0a78a3f2` | 1,399 tris；試點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_swallow | sasaki W（c22e_swallow） | 25 | 50 | 24305 | `b8e175c9-7bb0-4add-a799-1a10ad39b583` | 1406 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_rock_chunk | yamagata Q（b20w_rock） | 25 | 75 | 24280 | `da5db5e9-0dc7-4908-8a1e-e53cb794f499` | 1414 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_throw_axe | kojima W（b27e_axe） | 25 | 100 | 24255 | `2a3a479b-b146-4ae4-9009-1b4d6da3cd66` | 1374 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_fire_bomb | nohime R（a26t_bomb） | 25 | 125 | 24230 | `3f56f2c1-e1a3-4772-a79c-1d729c9a6275` | 1412 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_cannon_shell | nohime Q（a26w_flak） | 25 | 150 | 24205 | `5249e1aa-c1a2-416f-8d4d-a505a9a3bc1e` | 1430 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_rocket | nohime W（a26e_rocket） | 25 | 175 | 24180 | `f5780828-c790-4759-a9a6-033280e8a5aa` | 1350 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_arcane_arrow | otani C（a12f_arrow） | 25 | 200 | 24155 | `5f69627b-1e4d-4fa7-91b0-0497484d9a4a` | 1332 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_fissure_blade | kojima R（b27t_slam） | 25 | 225 | 24130 | `090d2df0-72e2-4b25-998e-d80558ab1cd6` | 1410 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_earth_crown | yamagata W（b20e_thunderclap） | 25 | 250 | 24105 | `d2e51e27-74b8-43fd-9a94-485908138f39` | 1378 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_thunder_spear | keiji W（c05e_bolt） | 25 | 275 | 24080 | `ec2d9c19-a40f-4f7c-ae81-50dbe385acd4` | 1416 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_fire_wings | okuma R（b26t_wings） | 25 | 300 | 24055 | `50913490-8d72-41e8-926c-7e5231d92828` | 1500 上限 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_shock_dome | okuma Q（b26w_shockwave） | 25 | 325 | 24030 | `41a17c6c-f481-471b-a9e8-1fc8cca7ef36` | 1500 上限 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_thunder_sword | tokugawa E（a01r_lightning） | 25 | 350 | 24005 | `08956e34-7f8c-435f-99d9-7f0306edadf3` | 1500 上限 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_war_banner | tokugawa R（a01t_vo） | 25 | 375 | 23980 | `1f48d21b-7d2d-48f7-ad1c-bbfd43a83a05` | 1388 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_spear_head | toshiie Q（a14w_spear_cast） | 25 | 400 | 23955 | `9ebeb1a1-6fe3-4ec3-b802-dd273889ce3d` | 1208 面，靜態，匯出 FBX 0 點 |
| 2026-11-03 | Tripo（單張圖、H3.1、512） | fx_iron_bell | shimazu Q（c03w_vo） | 25 | 425 | 23930 | `46852dae-614d-4b69-a129-6a07e144e8fa` | 1347 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_rope_net | tokugawa W（a01e_net_land） | 25 | 450 | 20535 | `8ad243fc-ba0f-4366-96e0-79c316bd474f` | 1470 面的網格撐不起網孔，成品是破碎實心片，未採用 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_tower_shield | kojima E（b27r_model_grow） | 25 | 475 | 20510 | `52335c43-fda4-482c-805d-4779d3979b43` | 1340 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_oni_mask | okuma W（b26e_voice） | 25 | 500 | 20485 | `d3f409a8-b015-4908-8056-583c40558ce3` | 1396 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_wind_spiral | keiji Q（c05w_vo） | 25 | 525 | 20460 | `b33e71c4-378a-4d49-9cf8-73210438f29c` | 1416 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_fury_spear | toshiie R（a14t_title） | 25 | 550 | 20435 | `f1a19e3c-bb68-4e00-b4e9-dd931a0fc3f6` | 1500 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | weapon_flintlock | 濃姬雙手槍（左右共用） | 25 | 575 | 20410 | `a67be971-fe4f-4a43-ade1-270c8f4d8d9f` | 1460 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_earth_spikes | yamagata W（試做） | 25 | 600 | 20115 | `c3e306dd-0d81-4eba-9b32-a36b8d820897` | 1374 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_powder_keg | nohime W（試做） | 25 | 625 | 20090 | `b1d27aa3-bbb1-474a-8b27-ded6673255e0` | 1393 面，靜態，匯出 FBX 0 點 |
| 2026-11-04 | Tripo（單張圖、H3.1、512） | fx_stone_shards | kojima W（試做） | 25 | 650 | 20065 | `793b922e-0903-4c9c-b7c5-a70e5ac7e246` | 1408 面，靜態，匯出 FBX 0 點 |

2026-11-04 帳號使用記錄另有文字生成模型：14:54～14:57 約 9 筆（每筆 30 點，共 270；3 筆由資產時間推得）與 22:17 一筆 30 點。線 C 只用單圖生成，這些標為非本工作線，不計入上表累計。
| 2026-11-05 | Tripo（單張圖、H3.1、512） | fx_medicine_inro | tokugawa Q（試做） | 25 | 675 | 20010 | `efbcb0d3-69b4-4162-9229-c50acc91b14e` | 1392 面，靜態，匯出 FBX 0 點 |
| 2026-11-05 | Tripo（單張圖、H3.1、512） | fx_armor_plate | yamagata E（試做） | 25 | 700 | 19985 | `d2540e3c-6e34-4c64-bfdf-349df6a10089` | 1314 面，靜態，匯出 FBX 0 點 |

## 待決定

- Mixamo 動作的授權條款：一般理解是可用於遊戲，但還沒在網站上逐條確認。
- 下載的 Tripo 原始模型（FBX／GLB）是否進版控。角色放在 `omfue/SourceArt/Characters/TripoV1/<id>/`，建築放在 `omfue/SourceArt/Structures/TripoStaticV1/<id>/`，都還沒 commit。`omfue/` 照使用者要求不納入一般 commit。
- Tripo 生成物的授權（免費方案是否可商用）。網站上沒有看到相關條款，仍待確認。
- Tripo 的「模型私密性」維持網站預設「仅分享」，18 個模型都在帳號的資產列表裡。要不要改成私密，屬於帳號設定，由使用者決定。
- 帳號點數在階段之間共少了 3495 點，原因不明。最後一次在資產列表看到不是這項工作做的家具模型，表示有其他人或其他工作在用同一個帳號，需要使用者確認。
- 原始下載（`C:\Users\User\Downloads\<id>.zip`，18 個）沒有刪除。收件用的解壓副本在本 session 的 scratchpad `tripo-dl/<hero>/`，保留期限不保證；已收件的 FBX 和 manifest（含下載檔的 sha256）在 `TripoV1/<hero>/`。
- 8 張 Grok 側面參考圖都沒有用在 Tripo 生成：多視角只上傳正＋背，因為 Grok 側面圖的手臂和武器位置常和正背不一致。
