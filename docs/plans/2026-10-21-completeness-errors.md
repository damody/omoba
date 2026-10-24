# 營地與施放提示擴充錯誤紀錄

## C01：工具與 patch 定位

本機沒有 openspec CLI；沿用 repository 的 spec-driven artifacts 手動維護，不宣稱 CLI 驗證已通過。第一份 moba_match patch 尾端 hunk 排在檔案較早的位置，apply_patch 正確拒絕整份；依檔案順序拆成兩次後套用，拒絕版本不算已實作。後續 patch 先讀實際 context，並依來源順序排列。

## C02：第一次完整內容回歸 242／246，四項失敗

保留 completeness-base-full-v1.log，整輪不接受。舊硬重置 fixture 現在有兩秒 patience，不能再斷言下一個 tick 就返回；將該項明示為仍支援的零 patience／零 soft reset 舊策略，新軟重置由獨立真傷害測試驗證。基地兩項仍在第一路拆完後對基地送 lethal 並預期免傷，改為驗一路全部塔死亡退休才開門，且其他路塔仍在。正式三種子 camp replay 在回營路上過早下 AttackTarget，角色停在 leash 外，軟重置依法忽略外部傷害；回返 MoveTo 不再被範圍外攻擊命令打斷，只有回到營地範圍內才重新攻擊，原3000 tick與每 frame 雙副本 hash／零repair斷言保留。

## C03：附屬怪 ID 追加地圖後位移

檢查發現若把附屬怪排在全體普通營地／兵種之後，新地圖的普通單位會插到既有附屬怪之前。採獨立40000起保留區，普通Creep身分仍自32771起且禁止跨區；測試涵蓋兩張有不同及共用附屬怪的map、追加map不改任何既有ID及跨角色衝突。不是按怪物名稱硬編runtime分支。

## C04：地圖牆群數量的舊固定斷言

加入鏡射野區牆群後共有14個terrain rect；authoring及出生／lane／營地本體防碰撞驗證皆通過，template-v3僅失敗於舊固定數量2。將該份正式內容契約更新為14；其他刻意只有2塊地形的獨立轉換fixture保留原值。未修改碰撞門檻以接受地圖。

## C05：第二輪完整內容回歸 244／246，fixture 位置落在新牆上

保留 base-full-v2 failed。硬重置舊策略把英雄放在營地右側250，正好位於新牆x850邊界，無法追到非法落點；改到同距離可行走的上側。siege yield fixture用tower_pos減水平1400作退讓位置，落在野區牆內；改沿基地至實際中路塔的方向取樣，仍要求真MoveTo精確退到兵後、無兵hold與承諾frame後才恢復，沒有放寬導航／時間／距離斷言。UE正式建置78.84秒已成功，這兩項是Rust cfg(test) fixture修正，不修改產品規則。

## C06：短場景工作流已退出才綁監測

world-v2在監測綁定前已完成，validate_owned正確拒絕不再存在的PID；沒有追錯程序，也不冒稱產生了v2三分鐘樣本。該輪未滿三分鐘，期間已人工讀取engine／case進度，最終workflow exit0、26項成功；world-v1及各長測試另有開始／完成監測。後續完整local會在啟動後立即綁定root及foundation／tail進度。

## C07：尚在寫入的 AVI 為獨佔分享模式

想從本輪連續錄影擷取原始MJPEG影格時，io.open回報Permission denied，原因是UE writer尚持有目前片段，不是需要使用者提權。沒有中止或干預驗收；等片段正常關閉再以新的輸出目錄擷取，原始AVI不修改。第一次輸出目錄保持未完成證據，不列為接受影格。

## C08：Lua path.write第四參數才是binary

第一輪已關閉AVI的frame擷取用了path.write(file,bytes,true)，第三參數其實是overwrite，Windows文字模式改動了JPEG換行，檢視成灰色破圖。原始AVI未受影響；V2擷取輸出不接受。改成write(file,bytes,false,true)，並驗實際檔案size及binary read逐byte相同，再輸出新的V3。新封存工具對原始JSON副本也明確binary read/write，避免在副本轉換CRLF，SHA／size以真實bytes記錄。

## C09：提示畫面有顯示但中文字過小

local-v1完整206 native／兩段操作／69Blueprint與PCM成功，但原始AVI frame600顯示提示字級10經目前Slate縮放後約7 physical px，不足作最終可讀畫面接受。沿既有FOmHudReadableText的physical floor，預設至少12physical px，大字設定跟著增加；底板／行高依字體量測、長文字換行，放在落點上方並限制玩家視口。新增.5／.6667／1／1.5／2 paint scale與大字的physical尺寸斷言；新建置及完整local-v2獨立接受，不把v1當最終UI結果。查找共用文字工具時曾猜不存在的OmHudMetrics.h，後以實際symbol盤點找到OmHudViewportSpace.h；後續只讀盤點結果。
