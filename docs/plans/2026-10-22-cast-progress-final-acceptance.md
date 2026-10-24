# 2026-10-22 權威施法進度與結果提示最終驗收

OpenSpec `complete-moba-cast-progress` **5/5完成**，接續全遊戲17系統盤點，優先單機可驗證內容。

## 實際實作

正式MOBA每英雄保存一筆Preparing／Canceled／Released、固定點remaining／total與遞增instance。到期後依真實handler交易結果記錄成功／取消，不從冷卻推測。明確取消、生命期失效、釋放再驗證失敗清理進度，死亡／非playing／沒有英雄輸出零，狀態納入authority digest。

PostStep沿既有Team HUD metrics輸出slot／state／remaining／total／instance，沒有施法目標或位置。client-runtime只取指定team／player／slot，拒絕缺欄、重複、非法時間／狀態／instance；typed optional protobuf保持舊snapshot無進度的相容性，同步checked-in生成碼並做codec roundtrip。

Rust與C header同步ABI31，40-byte ability row經正式bridge到Unreal owner payload。技能欄增加權威進度條、準備時間與短暫取消／已施放文字。同instance不能時間倒退、取消後復活或舊終態覆寫；新instance重新開始，空frame清理。UI不推進權威時間，終態文字1.25秒僅為表現。

## 最後集中驗收

RustV2 **1713通過／10既有ignored／0失敗**：template61、core859、codegen57、base255、backend190/1、bridge128/1、client148/8、ABI15；另generation成功，9工作exit0／退休。

新增核心明確結果／instance／退休清理，正式60Hz技能真實PostStep準備／遞減／Stop取消／再次施法／成功釋放／死亡清零；client測隔離、缺欄、重複、非法tuple及protobuf codec；bridge測typed HUD跨ABI31／空frame。UE CastProgressLifecycle檢查比例、文字與舊狀態防護，既有CastWindupTooltip一併回歸。

V1的client新測試因i64 metric與u32 protobuf狀態直接比較而無法編譯，其他八工作exit0；明確checked conversion後，等V1 worker／watcher退出才重跑全套V2。保留V1所有原始log及被撤回的V1來源檢查，不把中間結果當接受。最後變更僅Rust測試比較型別，UE build／World所測正式程式碼未變；V2來源集合包含此修正。

UE5.8 **build-v1成功（UBT 86.25秒）**，後端建置與Lua三契約成功。WorldV1 **35成功**（34 Success＋1 SuccessWithWarnings／4既有preview-world清理warnings；0 Failed／NotRun）。兩map各74影格／6片段，兩張輸出已目視，屬公開場景fixture；未以閒置畫面冒稱真實連線owner ACK或動態施法條的視覺接受。

**3349來源／資產**集合、SHA、size、mtime在驗收前後一致；最終比對等待Rust及capture worker／watcher退休。current build／staging／DLL與import library另驗。**66份原始證據**逐byte／SHA封存獨立複核。三分鐘監測 **8次／最大181秒**，另讀進度與engine log確認；**10個workflow worker＋10個watcher**及全程monitor退出。

## 尚未完成

真實連線owner ACK下的動態施法畫面、角色施法動畫／可見敵方提示、更多中斷／扣款政策、持續引導／脈衝、非法施法意圖與舊命令規則、特殊技能幾何；完整召喚複本生命期、競爭落點整組原子回滾、退休callback、objective永久stack／傳承／小兵光環、營地分裂、塔皮／公告與美術。

原MVP維持 **33/44／11外部待驗**。未新增網路對局／重設配額／commit／push，omfx不維護。209 native／458 IPC／69 Blueprint／五PCM未重跑不混算。錯誤P01–P04與盤點已更新；封存索引為`docs/acceptance/2026-10-22-cast-progress/index.json`。
