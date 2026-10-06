# 6.1 原生呈現功能封關（核對已保存證據，不重跑驗收）

## 完成範圍

OpenSpec 6.1 的通用動畫、技能 cue、視野／地圖呈現與非必要美術 fallback 已有正式實作及執行確認。本次只重新核對已存在的報告與目前來源，沒有再啟動 Editor、PIE 或對局。

- `omfue/Saved/McpAutomation/FinalNative-20261006-fixtures-fixed/report.json`：同 Editor 兩輪，各42 passed／0 failed／0 skipped／0 not_run／0 running。
- 實際執行案例包括 NativeHeroPresentation、AnimationStateSmoke、GenericAnimationOverlay、ProjectileCueStyle、AbilityCueStyle、AbilityCastCue、ProjectileCueHistory、WorldBridgeSyntheticFrameSmoke、RememberedGhostPresentation、CollisionTerrainPresentation、FogGeometryKey、FrameGeometryRanges 及所有 MinimapTerrain/Memory/Teams/FogBoundary/FogGrid 案例。這些不是「僅編譯」狀態。
- `omfue/Saved/McpAutomation/FinalPie-20261006-includes-fixed/pie-smoke-report.json`：success=true、native_mesh_rendered=true、remembered_ghost_rendered=true、記憶instance數1→0，pie-after確認pie_running=false。native英雄／記憶截圖已在原執行回合檢視，不以actor存在等同畫面渲染。
- 通用fallback與未知內容／動畫缺失政策有native斷言；缺素材不改內容身分、權威玩法或生成資料，不需角色專用C++/Blueprint graph。

## 明確不涵蓋

這項是呈現功能封關，不等於所有素材品質、全部Skeleton相容、完整選角到自然結算、兩台LAN、正式60Hz效能／GPU或硬60FPS。PIE報告所示120 FPS不是正式60Hz效能證據。報告仍含fixture World has no context、非法frame拒絕及RHI資源預算警告，不宣稱零warning。

6.2、6.4、6.5保持未完成；4.3的真UE重連與六類TCP事件已有分層證據，但完整整合範圍仍另核對，不以本項擴大結論。OpenSpec由24/31更新25/31。
