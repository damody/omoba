# MOBA 經濟安全結算進度

後續 receipt／死亡期間 owner metadata／持續性 IPC 與 TCP 重連已於 `2026-10-04-moba-shop-receipt-ipc-progress.md` 補強；下列未完成項目為本輪當時狀態，C ABI／UE 商店與收入／回城仍未完成。

## 計畫與決定

1. 接通正式權威 shop kernel 後，補 owner-team Gold／Inventory／ItemEffects baseline 與逐 tick 結算。
2. 敵方只接收可見最終數值；測試出生、Reveal、fresh bootstrap／重生及完整 hash，不用 ComponentRepair。
3. 保留網路商店關閉，等 receipt／IPC／UE UI、catalog 版本及實際流程驗收完成才開放。

使用 `build-unreal-rust-moba-framework`（spec-driven，17/30）；本輪是 5.3 的部分實作，不勾選完整經濟／介面任務。

## 已實作

- Lua 物品新增明確 `catalog_id`，0 保留空格，拒絕重複；調整宣告順序不改 numeric id。生成模型／catalog JSON／完整 data hash 同步更新，舊 staged DLL 不代表新內容。
- `CommittedEconomy` kind 21：81-byte typed 結算，Gold、六格 ID／cooldown、十個 ItemEffects bits 與 dirty。保留既有 f32 component 精確表示，不用任意 JSON 作事件 payload；拒絕長度、未知 ID、非法 cooldown、非有限／負 bonus、負 balance、空格帶 cooldown、非法 bool。
- Gold／ItemEffects 加入 MOBA hero baseline；projector 在 Reveal 與 expected view 使用相同 owner-team 篩選，enemy 移除 Gold、Inventory、ItemEffects。fresh bootstrap／rebase 從這份已篩選 view 生成，不分享完整世界。
- Economy fact 只給擁有者隊伍，projector 另核對已存在私有 schema；receiver 必須已有三個私有 schema，完整驗證成功後才一起提交。重複套用冪等，不重複扣款。
- Shop accepted-input 仍可對應權威輸入，但 filtered stepper 不本地 buy／sell，只採結算。技能 input 與 owner cooldown 仍正常重演；filtered world 不持有 MobaMatch。filtered 單路第一次取得 lane clock 時安裝共用 generated ItemRegistry。
- `CommittedEquipmentStats` kind 22：40-byte 最終 HP／max HP／speed／armor／attack，只發布可見 actor。裝備的私有資訊不進敵方 view。
- 在 component export 後套用 typed settlement，再同步 Specs；未再獲允許的私有 component 也從 Specs 移除，避免後續 export 重建已移除 bytes。
- 權威 shop 另拒絕 compiled catalog 沒有的 item ID；kernel 的整數價格、原子交易及舊 TD JSON 路徑不變。

owner-team 代表同一 team-filtered stream，不是單一玩家密封資料。在未來多人隊伍中，隊友可取得這份經濟資料；renderer adapter 仍須按自己的 player ID 選 HUD。英雄死亡期間這份 entity 已退休，死亡 HUD 的金錢保留仍待額外 owner metadata／IPC 契約。

## 問題、修正與證據

- 首次新增 helper 的 patch 誤放進 movement scope，E0425；修正為獨立 settlement helper，唯一函式上下文定位。詳見 error register E087。
- 新商店 hash 回歸在 tick 3 失敗，逐 schema 診斷確認敵方 max HP 沒有裝甲加成。將公開 settlement 補完整 HP／max HP；沒有放寬 hash 或追加修補。
- 新雙隊商店回歸通過：兩英雄確實互相可見，兩隊各買裝備、重複材料合成差價、餘額不足拒絕、出售、下一 tick stats 重算；逐 tick 完整 canonical hash 一致、零 ComponentRepair、enemy input／私有 schema 不外洩。
- 最終餘額 p1=525、p2=550；fresh bootstrap 私有 schema 仍正確；穿靴死亡後 40 ticks 新 generation 重生，Gold／格位保留、速度加成不重複，雙隊 hash 持續一致。此為 headless fresh bootstrap，不是實際 TCP renderer 重連。
- core 314 pass，含 bounded codec、private capability、冪等、非法記錄原子拒絕；template-ids 27 lib＋23／8 integration pass。
- base_content 70 pass，含 15Hz 完整 filtered lifecycle（1618 ticks／3234 steps）與 120Hz（10698 ticks／21394 steps），死亡／重生／終局凍結完整 hash 一致。
- omobab 140 pass／1 ignored，client-runtime 41 lib＋3 bin pass，Unreal bridge 41 pass／1 ignored。既有 warnings 保留，ignored 不算驗收。
- script-abi 13 pass；新 codec rustfmt 檢查與 scoped git diff --check 通過。私有狀態同時在 disclosed map 與 filtered Specs 被檢查，不只驗證呈現輸出。

## 尚未完成

未新增 renderer shop intent、交易結果 receipt、Gold／六格 persistent IPC／C ABI／UE 商店介面；secure V2 仍拒絕買／賣。收入／擊殺助攻／回城、三路、五位置 Bot、完整 UI／LAN／效能仍待實作。

本輪沒有 rebuild／stage／重啟 Unreal，也沒有 commit／push／cleanup。下一次真實 UE 驗收必須完整重建 DLL／consumers、重新生成 catalog、執行 stage gate，不能用前輪 success 或 skip-build 代表目前源碼。
