# 角色詳細資訊與 C 操作

盤點：Stats 在 profile 有 C 預設，但 action owner 的 implemented list、binding 與 handler 全缺；英雄頭像仍用引擎固定小字提示。這是正式操作缺口。

- [x] C 沿單一 owner／profile 主副鍵與精確 modifier切換；repeat、文字焦點、Esc／失焦／來源更換／退休驗證，不送遊戲命令。
- [x] 有界且按內容量測的可讀資訊面板，400／800與文字1／2；一般14／放大28，不縮字。
- [x] 真公開生命／魔力／等級／經驗／技能點／戰績與完整當前攻防資料：UnitStats → filtered ECS → ReplicaHost → schema4 → IPC11 → ABI29 → Unreal。頭像、名稱與C面板共用可讀資訊；buff沿正式catalog中文名與typed row，未知／非法／退休資料不沿用舊值。
- [x] 基礎 C 面板真Slate／改鍵／focus／生命期；ordinary build-v7 no-cache／refresh=false、native-v4 exact5 Success／0warning，103來源SHA一致，四張當次原圖逐張檢視。範圍接受 playable-mvp-player-stats-panel-accepted-v1；E1178–E1184保留，完整攻防資料與原4.5仍pending。

## 完整攻防資料的正式接線

目前 CProperty 只有 hp／mhp／msd／def_physic／def_magic，TAttack 包含攻擊基底、範圍與攻擊間隔；最終數值必須共用 UnitStats::from_refs（BuffStore），不能把 schema 基底直接冒稱含 buff 的最終值。SpecsDisclosedWorldStepper 持有已過濾 ECS，ReplicaHost::extract_presentation_source 已有從 stepper 補當前 summoner projection 的正式先例。

1. 從該 filtered ECS 的 Hero／PlayerOwner／ReplicaIdentity 及正式 UnitStats 查詢輸出型別化 presentation attributes，綁當前 replica id＋disclosure epoch。只查英雄，不逐 frame 掃全部萬筆單位；缺必要 component、重複 owner、已退休 identity 不輸出，不快取舊資料。
2. FilteredRenderSnapshot 增加本機 presentation metadata，經 ReplicaHost 明確填入；Moba HUD 只取當前自己的、與現有 body identity 完全匹配的屬性。此資料不進 deterministic world hash、不新增 authority 對局或自行合成遊戲結果。
3. HeroHudPresentation 新增 optional typed attributes；明確版本與舊資料 unavailable 語義。Renderer IPC／Rust FFI 若因此變動，正式遞增版本、更新所有 maintained layouts，由工具生成 cbindgen header，不手改生成檔。
4. Unreal 只做顯示與單位換算，沿共用詳細資訊／頭像 tooltip 更新；合法負防禦不能被當壞資料，不把 absence 當零，不由 item catalog 或英雄模板猜當前屬性。
5. Rust 驗真 production UnitStats 對 buff／移除／世代／owner 的投影與非法邊界，正式 proto／ABI／codegen；ordinary UE 建置與真 C UI 更新。這些完成前不聲稱原4.5完整詳細屬性完成。

2026-10-18 實作進度：正式 UnitStats 的六項 Q10 屬性已接 filtered ECS → ReplicaHost → optional schema4 → renderer IPC11 → ABI29 → Unreal typed payload。BuffStore 最終攻速計算由真攻擊路徑共用；公開世代切換會退休舊 ECS 與永久 modifier，避免新 body 繼承舊增益。cbindgen 正式生成與 staged header byte identity、MSVC layout smoke 已成功。rust-v1 的舊 ABI28 assertion 失敗保存 E1187；修正後 rust-v2 已完成 Core782／Client147（8既有忽略）／Bridge126（1既有忽略），all-target／no-default-features 與 Unreal 原生驗證仍在執行，尚不勾完整項目。

唯讀範圍使用共用 Slate metadata，沿當前焦點 ancestor 判斷，不掃整棵 widget tree；Stats／tooltip 的閱讀鍵共用捲動邏輯，Stats 的其他鍵會消耗。必須以真正 Slate Q／S／H 與 repeat、owner tick 後面板保留、無 gameplay input／error／camera zoom 改變驗證；不能僅靠 UI handler 回傳值聲稱隔離成功。

初始缺口已補：BuffListValue 保留為診斷相容介面；玩家資訊改用獨立中文 formatter，從正式內容名稱讀取，缺名稱顯示未命名狀態。資料只在當前 hero id＋generation 匹配時顯示，名稱單獨變更也更新；不把舊ID混入玩家面板。

## 本批接受

playable-mvp-hero-attributes-accepted-v1／summary.json：ordinary build-v6 NoEngineChanges／NoUBA／nocache19.94秒成功；native-v3 exact10 Success／0warning／trueexit0；139來源全程一致。800×600、400×300文字1／2四張當次C原圖逐張直接檢視，400文字2已無D提示蓋住閱讀面板，close與scroll均在有界區域，末尾buff可讀。Rust真Core782／Client147（8既有忽略）／Bridge126（1既有忽略）及五checks，33相關來源於本輪嚴格SHA重驗；ABI29 cbindgen正式生成／staged byte identity與MSVC layout smoke保留且header仍byte一致。E1185–E1195的失敗與修正證據保留。無新權威對局／ACK，非完整MVP驗收。

buff 文字接線：正式 OmVisualRegistry 已有 FBuffVisualEntry.DisplayName，不需再發明名稱表或變更 Rust ABI。由 OmGenerated 的 BuildOwnedBuffListSnapshot 將當前公開 catalog 名稱複製到 OmRuntime typed row，缺名稱用中文未知狀態；玩家詳細面板改用專用 formatter，診斷 ID getter 保持相容。保留64列／target＋generation／visual key 去重／剩餘時間檢查與清理，永久與倒數顯示中文，驗 source 記憶體生命期與缺資料。接線已實作，正在執行當次原生驗證，尚不作驗收完成聲明。

整併下一輪 UE：E1191 const 型別修正、typed buff DisplayName／共用過濾及玩家 formatter、玩家 buff cache 綁目前 hero id＋generation、名稱單獨變更也更新面板、未知名稱中文 fallback、小於0.1秒不誤報為0秒，以及小地圖缺資料中文。保留原診斷 getter與兩列既有 buff 測試，不以修改診斷預期掩蓋相容問題；增加真 catalog 名稱投影與玩家讀值／換世代清除斷言。預計當次 native 為原9項加 OwnedBuffListSnapshot 共10項，非完整 MVP 驗收。Rust33份相關來源保持不變，嚴格重驗既有 suite 的 SHA；C++ 變動則重新凍結全部來源、建置與原生驗證。

建置進度：source-v3 凍結139份來源；Rust／proto／ABI33份與 rust-v2 的真782／147／126與五check完全一致，rust-revalidated-v1只重驗SHA，不冒稱重跑測試。refresh build-v3 的SCW19.02秒、必要模組188.88秒、內部普通gate3.36秒皆成功；另 fresh ordinary build-v4 NoEngineChanges／NoUBA／nocache5.17秒成功，139來源不變。native-v1 已啟動10項與四種C面板原圖輸出，尚待實際結果及目視；無新增權威對局／ACK。
