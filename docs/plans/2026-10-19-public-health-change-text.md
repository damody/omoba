# 下一批：公開生命變化數字

專項完成時的狀態：正式SynchronizeCombatFrame已接上世界級lazy／64-slot／每owner四slot文字池，使用Engine UnlitText、公開HP delta與明確正負號。scope／generation／fog／pause／maxHP／respawn／容量回收均已專項驗。HealthTextFocusedV9兩項Success／零warning與2036真glyph pixels、當輪opaque PNG已直接目視；FocusedV12再驗選角／VFX／生命文字／商店四項全Success／零warning。4.3含authority ACK等未驗範圍，父項不勾完。

最終完整本機驗收已通過，見 `2026-10-19-ue58-final-acceptance.md` 與 `docs/acceptance/2026-10-19-new-machine/index.json`；198 native、69 Blueprint、458 viewport、Rust／ABI／五checks與3325份指紋均已完成，接受界線與既有warning／ignored完整保留。

2026-10-19初始來源盤點（歷史）：4.3仍未完成。`OmUnitActor`已有公開HP驅動的生命週期／音訊、`UOmCombatVisualSubsystem`已有世界級有界VFX池，但當時搜尋不到damage/heal/floating combat number實作；由此確認本輪缺口。

先等目前UE bootstrap及renderer fixture建置真正退出，再修改production／test來源；使用者要求當前功能實作確認後才集中完整驗收，不啟動新權威對局。

## 直接實作方向

1. 擴充既有`UOmCombatVisualSubsystem`，提供世界級、有界、lazy配置與可重用的文字slot，避免每個entity每frame建立UI／Actor。沿既有OwnerCounts、cue去重、scope、pause、fog、identity與pool回收，不另建另一套世界呈現owner。
2. 在正式`AOmUnitActor::SynchronizeCombatFrame`共用路徑消費合法公開HP變化（native與Blueprint都會經過），顯示正／負數字。這是公開HP增減，不捏造未揭露的傷害、overkill、技能／施法者或治療來源。首個frame、epoch／generation更換、reveal、respawn只種baseline；MaxHp改變也只重種，避免把升級／裝備調整當治療。拒絕非finite、舊tick／sequence與不同identity。
3. 隱藏、pool、source退休和scope reset立即清掉舊數字；paused不老化；死亡frame可以呈現當次合法HP損失，respawn不顯示巨大假治療。明確正負符號，不只靠紅綠色。使用既有nameplate位置／visual scale，文字朝向本世界的local camera，不掃描全世界猜玩家。
4. 針對正式generated hero／真文字component驗首次／受傷／恢復、duplicate、舊frame／epoch、死亡／重生、hidden/reveal、容量與重用、pause與來源退休；確認無per-entity新增空元件。另存實際render PNG並直接目視，不能只檢查字串或helper。
5. 普通NoEngineChanges建置與必要專項確認當前功能成功；最後集中跑完整native／Blueprint／owned viewport、Rust／ABI及來源／資產保護檢查。4.3還含完整command ACK／feedback，不因局部數字成功勾完整父項。

## 已確認來源

- `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmUnitActor.cpp`：`SynchronizeCombatFrame`、`SetActorHiddenInGame`、`ResetPresentationState`、nameplate與音訊HP baseline。
- 同module `Public/OmCombatVisualSubsystem.h`、`Private/OmCombatVisualSubsystem.cpp`：容量512、OwnerCounts、RetireOwner、ResetScope、paused/fog檢查與世界級render host。
- `OmGenerated/Private/OmWorldBridgeActor.cpp`：正式bridge來源位於OmGenerated，不能再誤讀OmRuntime/Private同名檔。
- `OmEditor/Private/OmNativeCombatVisualTests.cpp`：既有有界VFX與生命週期測試，可沿其world/context與實際資源建立自足專項。
