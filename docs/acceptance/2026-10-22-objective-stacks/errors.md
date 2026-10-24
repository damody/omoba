# 史詩目標疊層錯誤紀錄

- S01：盤點時將ability_runtime誤認為單檔，並使用Windows rg不支援的目錄wildcard。正式目錄為native/ability_runtime/，後續先以rg --files定位，再以-g過濾。
- S02：追查既有Unreal顯示介面時猜測OmPresentationTypes.h名稱，實際不存在。此查詢無修改；保留錯誤，後續檔案入口必須先列目錄／rg --files，不由型別名稱推測標頭路徑。
- S03：新增測試透過World的內部可變性操作資源，binding不需mut；核心編譯提示unused_mut。移除後最終來源改以V2整批接受，V1證據保留。後續集中建置前應檢查新增測試的mut是否真的呼叫&mut World介面，避免為無效mut反覆重建。
- S04：V2在V1生命期測試尚使用同一路徑base_content測試EXE時重新連結，Windows LNK1104檔案鎖定。V1已全數成功；V2與相依source守門失敗紀錄保留。最終RustV3加入通用前置退休守門：每個前批receipt必須retired且owner／watcher PID＋creation-token均已退出，才可開始新批。最終來源比對等V3成功；沿用V2已成功的UE／後端、V2來源快照與World，不以失敗V2 Rust作接受。禁止邊跑同target測試EXE邊重建它。

集中驗收問題與修正追加於此；不刪除失敗證據、不降低既有斷言。

- S05：擴充證據封存時，跨檔apply_patch末段使用非完整來源行作context，驗證失敗且整份未寫入。改為分檔精確patch；原始紀錄／正式來源未受影響。
