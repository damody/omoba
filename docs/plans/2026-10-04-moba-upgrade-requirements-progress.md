# 技能升級門檻實作紀錄

## 本輪計畫與決定

- 新增共用 AbilityLevel.required_hero_level，缺值預設1；值須1..25且單調不下降，rank數必須符合max_level。Rust與Unreal生成器共用驗證，runtime MOBA開局核對compiled門檻，不接受單邊修改。
- 保留四招初始rank1；training_luminary的lumen_lance宣告[1,6,11,16]作明確資料範例，非根據R鍵或ultimate字串猜規則。這不是完整英雄聯盟初始技能配置。
- Rust只在MOBA扣點前判斷下一rank門檻，TD舊行為不變。Unreal由生成visual registry讀同一門檻，既有bCanUpgrade／Tooltip欄位表達可升級及原因，不增加角色graph或遊戲腳本ABI。
- 本輪門檻增量已完成下列驗證，不代表整套框架完成；本輪錯誤與決策見E119。OpenSpec保持19/30，5.3／6.2未勾選。

## 已驗證與範圍

- shared model7、template66（含compiled agreement／禁止hot reload）、codegen24、core333、base94、server154、runtime60、bridge52與Fyrox check通過；最後新增開局rank1防呆後core333、base94再次通過，DLL與bridge已重新建置部署。
- 正式60Hz PlayerInput升級後第一招傷害80→125，冷卻採用第二級6.5秒且不靠HUD猜測；6／11／16等級邊界之前拒絕且Hero完全不變，恰達門檻扣一SP升一rank。60Hz雙隊40tick／80steps包含lvl6升lumen_lance至rank2、敵方input私有及零repair完整hash。
- 真實KCP60Hz1791093512使用新Lua，3902 snapshots（1381／1274／1247）原wire rank／SP／XP／Gold／計分板核對通過；同request雙送各原input3只接受一次。兩隊各10、player3獨立9 checkpoints至1200，各2post-kill parity；四PID90260／5564／88660／62888另外inspect退出。
- OmGameEditor完整建置成功、MCP11BP compile1791093495通過；同Editor95892兩輪17/17原生測試，含NativeAbilityProgression與GameplayInputSurface。串行PIE smoke完成並退出PIE；不是升級對局畫面。最後開局防呆重建後Editor79996、11BP compile1791094033再次通過；同一新Editor再跑兩輪17/17通過。
- 生成hash ac6ff592a15c8b5e、11files／15Lua inputs --check通過；最終開局防呆重建後bridge與stage SHA-256精確相同：e968182ac36c1025bfb0952b23e5b1940ac6ea9de450abdbed7fdd490ebc6990。OpenSpec strict與本輪來源／生成檔whitespace檢查通過；沒有新增根目錄wrapper。
- UI共用函式讀生成registry，bCanUpgrade與Tooltip表示門檻／點數／inactive／max／unknown；native bar顯示Ctrl+key [+]及tooltip。沒有每英雄C++、Blueprint graph或新增FFI script ABI欄位。
- 現有vertical slice依然四招初始rank1。若rank1宣告required_hero_level>出生level，開局明確拒絕，不能繞過門檻；延後第一次學習／rank0 loadout未完成。本輪没有實體Ctrl按鍵、升級後HUD像素或網路升級技能傷害驗收，不能以Editor純函式測試代替；整個5.3／6.2保持未勾選。
- 最終Editor79996串行PIE smoke也完成截圖與stop，再查PIE已停止；保留Editor供繼續開發，無待完成的本輪測試程序。下一個驗收優先是正式UE綁定Ctrl+技能升級→authority扣SP／rank→HUD回傳，而不是擴到120Hz。
