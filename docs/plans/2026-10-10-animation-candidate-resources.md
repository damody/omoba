# 角色動畫候選：實際免費內容與限制

2026-10-10，於商店Undo來源writer執行期間獨立準備，只寫文件與`.ai-collab`素材候選，沒有UE匯入、建置或改動遊戲來源。

來源是[Quaternius官方動畫庫](https://quaternius.com/packs/universalanimationlibrary.html)及[作者itch.io頁面](https://quaternius.itch.io/universal-animation-library)。沿官方Download Now→No thanks, just take me to the downloads→Standard取得，沒有付款、帳號或條款提交。內附授權明確是CC0 1.0 Public Domain Dedication；Pro／Source未取得。官方宣傳的120多動畫包含其他版本，不能當作免費版實際數量。

實際ZIP為15,904,933bytes，SHA256：`cc73fc4e495b82958207316596317a3f40b9fa38065bde1027937452da537724`，12個成員。候選與原始授權在`.ai-collab/candidates/quaternius-animation-standard-v1/`。有界ZIP中央目錄、路徑、大小及原始複本SHA驗證通過。

實際有兩個Unreal-Godot GLB：`UAL1_Standard.glb`與`UAL1_Standard_RM.glb`，各43動畫、67nodes、65joint單一skin。已檢查GLB chunk邊界、內部buffer、動畫node／channel、輸入時間accessor邊界、有限非負且嚴格遞增的timestamps。證據在`.ai-collab/candidates/quaternius-animation-inspection-v1/inspection.json`，清單在`quaternius-animation-summary-v1.json`。這些CPU檢查不等於完整transform／骨架重定向或渲染驗證。

可用片段包括Idle_Loop、Walk_Loop、Jog_Fwd_Loop、Sprint_Loop、Sword_Attack、Punch_Jab、Spell_Simple_Enter／Idle_Loop／Shoot／Exit、Hit_Chest、Hit_Head、Death01與Dance_Loop。計畫優先採一般版配合權威位置驅動畫面，避免root motion再推動actor；這是實作方向，尚未UE驗證。[作者更新說明](https://quaternius.itch.io/universal-animation-library/devlog/1555893/added-root-motion-and-other-fixes)確認有兩種root motion匯出。

後續CPU檢查：`quaternius-animation-rig-comparison-v1.json`確認兩個Base Characters基底與兩個GLB的65joint名稱及nearest-joint階層相同；原process terminal未能從壓縮上下文恢復，後續Lua獨立解碼成功，限制記於candidate-inspection-output-recovery-note.md。另實際`inspect-animation-transform-outputs-v1.lua` exit0，證據`quaternius-animation-transform-v1/inspection.json`：每個GLB的43動畫／8385channels／1,318,850float components均通過output accessor邊界、input/output count與interpolation關係、finite transform及sampled rotation norm檢查，最大四元數norm誤差約0.000000158（門檻0.001）。不對cubic tangent要求rotation normalization。

實際局部rest TRS比較已完成exit0：`quaternius-local-rest-transforms-v1.json`。對兩個GLB，female有64／65 joints、male有62／65超過0.00001 component差異；最大translation差約0.077516／0.045536、quaternion component差約0.197852／0.146273。這些是原始glTF局部座標數值，尚不換算為UE單位或角度，也不含armature/root與inverse bind。名稱／階層一致不足以直接共用rest pose，後續需正式retarget及播放驗證。這是候選差異診斷，不是遊戲測試失敗。

仍待驗證：完整rest pose／比例／inverse bind相容性、UE5.8實際匯入與compression／retarget、原地移動方向及各動作播放、死亡重生／fog／source retirement／pool回收。未取得的動畫不能用空clip或單一角色別名冒充。尚未生產接受，也不能勾完OpenSpec6.4。

2026-10-11新增實際CPU矩陣診斷：固定Lua執行`inspect-candidate-bind-matrices-v1.lua`，fresh `quaternius-bind-matrices-v2/inspection.json`成功exit0。以完整staged-v2模型／依賴SHA與兩個原GLB SHA綁定輸入，四個golden數學檢查後檢查node世界TRS、finite／可逆affine inverse bind、accessor邊界與各skinned mesh預設姿勢閉合；四個候選各65joint，最大閉合誤差約0.000000639，小於0.0001，矩陣反矩陣殘差約4.44e-16。基底對兩個動畫版本均64／65個mesh-relative bind matrix超過0.0001，female最大component差0.228294494、male0.214012135；沒有缺失／額外joint。這是來源座標矩陣診斷，不換算UE比例／角度，也不假設任意glTF的預設姿勢必為bind pose。結果支持仍需正式retarget，沒有UE匯入、播放或視覺接受。

該次初跑v1失敗已保留在`quaternius-bind-matrices-v1/{inspection.json,errors.md}`：舊fullbody-inspection-v1只存JSON檢查副本，旁邊沒有binary buffer，不能把它當完整資產URI根。重跑改讀既有staged-v2 manifest的實際staged_path與依賴SHA，沒有修改原模型／動畫／候選檔或覆寫失敗證據。後續工具應依manifest定位完整模型，不猜目錄，也不把同名骨架視為bind相容。
