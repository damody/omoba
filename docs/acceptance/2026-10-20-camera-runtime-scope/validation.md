# 鏡頭Runtime生命週期接受

2026-10-20。本輪補齊直接Runtime Stop／Start仍保留舊Space temporary follow及其他camera held state的缺口。當前功能專項與整套分段本機驗收成功；OpenSpec仍33/44、11父項待驗。沒有新增對局、重設10/10、commit或push。

## 共用修正

AOmRtsCameraPawn在PostInitializeComponents及既有輸入owner刷新維護Runtime scope訂閱，EndPlay／BeginDestroy解除。停止、成功建立新Runtime或換subsystem時沿共用ReleaseHeldCameraModes取消pan、drag、scout、temporary follow；Runtime退休不恢復舊視點，另清除minimap inspect、舊hero／四個隊友slot、viewport focus觀察與pointer樣本。保留profile revision、同一input component、zoom與鎖定偏好；profile編輯／一般release仍沿原本恢復視點規則，冪等Start保留有效held state。

Om.Runtime.CameraRuntimeScope使用真port-zero Runtime、有效local controller／camera與實際InputStack，修正前red-v2唯一失敗為Runtime stop immediately retires temporary camera follow。green驗直接Stop即刻退休、新快照不能復活舊Space、舊release不回跳、新press移動actual camera、冪等Start保留follow、重複Stop安全、按住方向鍵時Stop後actual camera不再移動、minimap inspect跨重啟取消且新快照恢復保存的lock preference。此fixture不產生新權威ACK。

red-v1在profile apply前因fixture缺camera manager失敗，不算有效缺口重現；補InitializeFor及Resolve斷言後才取得red-v2。第一次green十九項中最後一個位置斷言使用11000／12000超出既有10000正邊界，正式clamp正確，改用邊界內6000／7000，不放寬正式guard。全部失敗報告保留，詳錯誤紀錄E1390–E1393。

## 當前驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local／無action cache建置 | 正式scope修正UBT51.31秒；最終fixture版本6.09秒，stage SHA一致 |
| Lua契約 | fresh 140 owned＋69 local acceptance成功 |
| 世界／鏡頭專項 | 19成功，四筆既有RuntimePresentationReset preview-world warning |
| 完整native | fresh 206成功；既有warnings保留 |
| 既有Blueprint | 69，零compiler error／warning／load failure |
| 原創實際音訊 | 五PCM通過 |
| 分段完整輸入清單 | 458案例／222 wire／157 stages／13游標觀察 |
| foundation與tail | 各294／120／103及164／102／54，個別原860／900秒期限、90 proof frames、8fps與完整清理 |
| 來源／資產 | 3335檔前後SHA／size／mtime及檔案集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust接受明示沿用2026-10-19新機證據 |

兩段原始native／wire／control history／UE index／cleanup與七binary cohort均由正式strict union重新驗證，兩段AVI再次通過RIFF、索引、timeline、幀數、SHA與8fps檢查。兩公開map另有144幀／12片段；本輪六HUD與兩map原圖已逐張檢視。圖片只作呈現回歸，新的scope取消行為由actual input-stack／actor斷言證明。所有publisher／editor／profile清理正常，三分鐘監測至root lifetime退出。

本輪execution_mode=phased、single_session_full_success=false；不將跨session清單覆蓋冒稱單session連續效能、原gameplay ACK、真LAN或實際100%／150% OS DPI配對。前次世界scope與失敗single-session包保持原樣，11父項缺證不勾選。
