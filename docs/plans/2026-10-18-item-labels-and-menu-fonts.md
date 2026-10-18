# 物品名稱與 HUD 小按鈕字級

延續 complete-moba-controls-and-presentation，直接實作，不另開代理。

1. 把十一件裝備的玩家名稱改為中文，保留 catalog ID、配方、價格、效果與圖像來源。減傷符不能誤稱視野道具。
2. 擴充既有 authoring → HUD manifest 名稱同步，物品也從正式 Lua 資料取得名稱，不在 Unreal 寫第二份名稱表。
3. 商店與操作設定按鈕改用既有可讀字級 attribute，沿既有真 desired-size 量測適應文字縮放。
4. 重新執行內容／codegen 測試與正式產生／check，凍結產物後普通 UE 建置。以真 Slate 回歸查名稱、搜尋、文字縮放、小視窗與點擊；直接檢視當次圖片。

只接受本批通過的功能；完整 MVP 驗收留在所有功能收斂之後。不使用新權威對局或增加 quota。

目前實作：十一名稱已從 authoring 同步到 43-entry HUD manifest（十一更新）；獨立比對排除 name／displayName 後，玩法資料與所有圖像／provenance 完全一致。商店與設定按鈕使用 OmShopBodyFont(AppliedTextScale) 的動態 attribute，保持既有 desired-size 量測。

本批已範圍接受：`.ai-collab/playable-mvp-item-labels-accepted-v2/summary.json`。item-label-codegen-v1 的 56 Rust tests、all-target check、正式 generate／check 成功；item-label-rust-v1 的 core782／client147（8 ignored）／bridge126（1 ignored）與五項 checks 真成功。後續只修改兩個 C++ 測試來源，item-label-rust-revalidated-v5 比對全部144來源，確認 Rust／Lua／產物不變，沒有冒稱重跑 Rust。

最後 ordinary item-label-build-v6（NoEngineChanges／NoUBA／nocache）成功29.21秒，item-label-native-v4 exact10成功／0測試警告，item-label-source-v6／144 hashes於建置與native後一致。12張當次原圖逐張original檢視：兩個text2尺寸、400text2真wheel捲後商店入口、四百寬道具卡、正式十一項商店／配方／材料、大字商店與六格出售、1280×720已填資料HUD。fixture的 Cloth／Salve 是人工 payload，不混成正式物品名稱；黑背景是renderer fixture，沒有新authority game／ACK。當次 final invariant另存 final-v1，玩法與圖像／provenance仍完全一致。

E1200–E1207完整保留來源盤點、編譯型別、英文搜尋與讀回、字級上限、overflow可達性及報告寫入的失敗／檢查紀錄。400×300／text2的既有有界scroll由真Slate wheel證明商店入口完整可達且zoom不變；恢復頂端後其他HUD檢查通過。沒有改小字或刪裁切斷言，設定按鈕仍獨立可見。完整MVP／實體搖桿／新權威對局驗收仍在後續，不由本批代替。
