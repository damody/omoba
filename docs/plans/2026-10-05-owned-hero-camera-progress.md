# MOBA 自由鏡頭與自己英雄追蹤（2026-10-05）

## 本輪計畫與完成實作

1. 接續6.2的實際操作缺口，不繼續擴充零碎控制效果。既有 FocusCameraOnOwnedHero 每份完整呈現快照都 SetFocusLocation，導致邊缘捲動被下一份快照拉回；缺自己英雄時還會借第一個可見英雄。
2. 共用原生 AOmRtsCameraPawn 保存安全 owner focus 與自由／鎖定／暫時追蹤模式。開局只定位一次，預設自由；Space press／release 暫時追蹤，Y 切換持續鎖定，縮放不被切換重設。顯式追蹤時停止邊緣捲動，避免兩種控制互相拉扯。
3. WorldBridge只選 configured local player 的存活Hero，位置只從合法呈現frame取得；移除另一英雄fallback與朝世界原點的偏移。缺席／死亡／非法位置使目標不可用且不跳原點；鎖定偏好保留，新自己英雄披露後恢復。沒有把相機操作送至 gameplay 或新增第二World。
4. 以 entity key 精確綁定目前生命，control-only RemoveEntity退休舊目標即清除；若同frame已換新生命，舊key移除不清新目標。完整ReleaseAllActors／Stop／EndPlay清除目標並允許新場次首次定位；一般control-only沒有退休仍保留合法目標。
5. 使用通用原生輸入與相機API，不新增英雄C++、Blueprint graph、美術相依或runtime Lua。保留現有UI pointer guard；沒有新增UI輸入認證或宣稱文字欄位／失焦行為已完整驗收。

## 當前建置確認

- 第一次限定OmRuntime＋OmGenerated＋OmEditor建置：10 actions、12.26秒、Result Succeeded。
- 精確生命周期接線後最終限定建置：9 actions、9.99秒、Result Succeeded。全程保留NoEngineChanges，不修改引擎／BuildId，也沒有重試完整restart。
- 新相機automation斷言涵蓋自由快照不拉回、鎖定立即定位與更新、缺owner不借別人／不跳原點、重生恢復、解除保持位置；本輪僅編譯成功，未執行Editor／PIE／真人Space與Y／完整畫面驗收。
- 日誌：omfue/Saved/Logs/native-owned-camera-modules-20261005.log與native-owned-camera-final-modules-20261005.log。不把編譯當實際按鍵成功；最後整合仍須執行這些斷言與互動確認。
- 不改Lua作者內容、生成hash、Rust、ABI／IPC／wire，不stage DLL、不維護omfx、不commit／push。完整6.2及21/31保持待完成。

## 錯誤與決定

- 調查猜 scripts/moba_interactive_selection.lua 得os error2，inventory確認實際為moba_shared_selection.lua；rg把OmPlayerController*放Windows路徑得os error123，應對存在目錄用-g。又誤猜OmGenerated插件根目錄，inventory確認OmGenerated是OmRuntime插件下的module。這些都是路徑調查錯誤，並非編譯失敗；未知路徑必須先rg --files，不重複猜測。記錄E254。
- 多項合併輸出超限後縮小；不把截斷當完整證據。沒有實際C++／UHT建置失敗。
- 相機只對自己存活英雄建立目標，不新增隊友／敵人追蹤或從actor世界補隱藏位置。首次fit維持原zoom與bounds框架；完整公開地圖相機邊界／像素與操作驗收仍留最後，不冒充正式導航邊界。
