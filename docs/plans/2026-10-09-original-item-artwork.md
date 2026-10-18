# 原創道具圖示準備紀錄

已使用內建 image_gen 產生11張獨立方形道具圖，每張輸出都已逐張目視。固定 Lua 將原始 PNG binary-identical 複製至 `.ai-collab/candidates/item-icons-v1/`，保留產生工具原檔；`manifest.json` 保存完整 prompts、來源、實際1254×1254尺寸、bytes、SHA256及catalog_id1..11。模型名稱未由工具提供。此階段是素材準備，尚未 Unreal import 或實際HUD／商店接線。

對應正式 authored IDs：moba_sword、moba_armor、moba_boots、moba_greatsword、moba_guard_charm、moba_stride_charm、moba_mana_charm、moba_ward_charm、moba_strike_charm、moba_training_flare、moba_training_hex。Ward Charm依現有減傷效果畫成琥珀防護符，不能代表未實作的視野ward。現有道具catalog未提供charges，不用圖示虛構可用功能。

後續有界工作封包 `.ai-collab/moba-original-item-icons-task.md` 已準備但未派送。將維護正式可重跑import／typed registry／六格卡與商店顯示，驗證實際Texture2D、NPOT UI設定、來源雜湊及第二次不重存；保留disabled原因、快捷鍵、冷卻與tooltip。待前一writer與實際功能檢查完成才派送，完整驗收最後。
