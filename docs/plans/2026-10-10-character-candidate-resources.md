# 角色素材候選與實際檢查（2026-10-10）

這是角色畫面實作的素材前置紀錄，尚未匯入 Unreal，也不是完整英雄或動畫驗收。

來源為 Quaternius 的 Universal Base Characters 免費 Standard 套件，下載包128,968,391 bytes，SHA-256為 `fdbf1804c90dfc1ea03e992bff7da2dfd1a79318e13270a660180f9308455f40`。實際解出的 License_Standard.txt為806 bytes，記載免費子集CC0；未取得付費Source套件。原始套件共有128個成員，但只有女／男兩個完整身體身分，其餘包括不同格式及髮型等零件，不能當作六名完整英雄。

兩個Godot／UE格式glTF均為3 meshes、69 nodes、65 joints、沒有內嵌動畫。固定Lua檢查了二進位buffer／accessor界限、頂點座標、所有三角形索引、關節索引及權重（合計容差0.02）；兩個模型均沒有非法索引／權重或非有限座標。這些檢查不包含UE比例、材質、法線方向、動畫重定向或實際渲染。

原始glTF的貼圖引用不完整：兩個身體均引用包內不存在的 `T_Eye_Normal_png.png`，男性另引用不存在的 `T_Hair_1_Normal_png.png`。初次檢查停止，fresh完整檢查如實記錄success=false，未把工具正常結束當成資產成功。

獨立候選副本已完成三筆明確記錄的同目錄 `_png.png` → `.png` 檔名對應。每個來源／副本及貼圖／buffer均記錄SHA-256，副本的引用可全部本地解析，原壓縮包與原始模型保持不變。這是有依據的檔名推論，尚不能證明材質法線方向正確；沒有補造動畫或直接覆寫正式資產。

證據位於工作區 `.ai-collab`：

- `candidates/quaternius-base-standard-v3/manifest.json`：原始成員、雜湊與授權。
- `candidates/quaternius-fullbody-inspection-v1/inspection.json`：實際模型／骨架／動畫盤點。
- `candidates/quaternius-fullbody-dependencies-v1/errors.md`、`candidates/quaternius-fullbody-dependencies-v2/validation.json`：原始缺失與完整檢查。
- `candidates/quaternius-fullbody-vertex-data-v1.json`：實際CPU資料檢查。
- `candidates/quaternius-fullbody-staged-v1/errors.md`、`candidates/quaternius-fullbody-staged-v2/manifest.json`：初次失敗及獨立副本對應。
- `quaternius-fullbody-dependency-review.md`：前後限制與修正紀錄。

避免重犯的工具錯誤：ZIP成員名含方括號時，Windows tar的pattern會解讀它們，必須先以反斜線正確跳脫，並用 `tar -tf` 確認唯一精確成員；不要沿用已失敗的方括號pattern替代法。使用 `tar -C` 前必須先建立實際解出目錄。每次重試使用新證據目錄，保留原失敗，不以空資料夾或exit0冒稱資產成功。

後續仍需實際UE材質／比例／骨架／動畫驗證，以及可辨識的英雄、兵、塔與基地呈現。正式前端須使用可覆寫的每模板呈現規則，不能把所有英雄都別名到同一角色。此候選未完成，不勾計畫6.2，也不替代最後完整畫面驗收。
