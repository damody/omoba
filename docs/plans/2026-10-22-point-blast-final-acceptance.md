# 2026-10-22 正式圓形爆發技能最終驗收

OpenSpec complete-point-blast-combat 5/5完成。補齊炎刃／炎襲的實際點選圓形戰鬥規則與對應瞄準半徑。

## 實作

base_content新增共用PointBlast resolver／executor，wrapper明確傳入AbilityId、數值鍵、damage kind／profile及可選closed stun／root／silence，不以英雄名稱特判。嚴格解析rank JSON；現在存在但型別／範圍不合法的extra拒絕，缺漏可用同rank生成常數且checked index不panic。限制damage0..1000000、range／radius1raw..10000、control1raw..60；正值低於固定點quantum拒絕。

來源必須alive且HP正值、存在合法位置，target必須point，source／center在共用有界幾何範圍；只限制落點中心的cast range，圓內敵人可延伸到施法者射程外。重用i128精確circle（零長度capsule），包含相切，排除一raw外、友軍、死亡／HP0與caster；候選canonical排序去重。全部預檢結束才發一次cue與每敵人一次damage／可選control，保留FIRE profile、canonical buff及既有瞬發語義。非法RErr交還既有host cast交易退回mana／CD／deferred outcomes，未宣稱一般process_outcomes整組原子回滾。

flame_blade改為合法point-circle、炎襲共用damage+stun；原四rank damage／range／mana／cooldown／radius／stun數值保持。原本None／Entity錯誤target或壞JSON會回ROk的路徑修正為失敗；炎刃preview改AreaEffect，炎襲preview增加stun StatusModifier。Lua描述如實說明指定地點、圓形、各一次傷害；模組歷史WERT註解改成實際QWER slot，未更動loadout。

共用typed PointCircle／radius_key發布手寫handler的同一radius鍵，兩生成器校驗完整rank、維度／range邊界及point instant active／ultimate；與executable effects互斥，避免雙重幾何。ability_requires_movement明確match SweptPath而非is_some，圓形技能不受root位移限制。Unreal沿既有current rank payload→owner circle／tooltip呈現炎刃100與炎襲500，不讀cosmetic特效半徑。無ABI／protobuf／CABI改動。

## 集中驗收

RustV1 **1950通過／10既有ignored／0失敗**；content36、sim72、template76、core919、codegen63、base284、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作成功。新增2個schema、1個arbitrary-ID codegen與7個base案例。

真adapter測試驗證exact circle／tangent／one-raw外／alive／HP0／ally／FIRE profile／canonical順序與三種control、非法source／budget／extreme／range、合法自身point、真兩handler的JSON／rank／extra／target拒絕與fallback、circle／stun metadata。正式Production60Hz涵蓋兩技能×四rank、中心最大射程與圓邊界、實際damage／stun、mana扣一次／CD、十tick不重複傷害、outside／Entity／missing資源回滾與root下非位移仍可施放。共享既有isolated／no-regen／dynamic slot fixture，未複製另一套對局設定。

四Lua契約成功。正式backend-v1與UE5.8 build-v1成功，UBT **48.71秒**；七DLL與import library SHA另驗。WorldV1 **54成功／0 Failed／0 NotRun**，warnings共4個，分類見本批errors及engine log。新增PointBlastMetadata真native案例確認兩技能各四rank半徑、零swept width、非位移policy、owner circle快取、tooltip／未學習清除。

兩張新PNG分別擷取真SOmHudToolTip，避免把雙技能長文塞在超出螢幕的單一視窗；其餘12圖作既有地圖／HUD／角色／沿路與位移提示回歸。共14張PNG在World worker／watcher退休後目視；圖片不當作真對局施法或ACK。兩張地圖各74影格／6片段。**3397來源／資產**前後集合、SHA／size／mtime一致；**86份證據**封存並獨立逐byte／SHA校驗。健康採樣**8次**、最大間隔**171秒**，7組worker／watcher及monitor已全部退出。

## 錯誤與剩餘

盤點路徑猜測與build前private-import自查記於2026-10-22-point-blast-errors.md，總登錄E1494。新幾何分類、真metadata／world-unit／MaxLevel與no-regen fixture沿用前批教訓；所有實際失敗及warning保留在log，不覆寫證據。

仍缺persistent zone／持續引導、定時衝刺／殘留、更多特殊幾何、位移召喚師／道具提示、完整combat replica接受、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效。MVP33/44及11項外部ACK／LAN／實際OS DPI／固定效能接受不變。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-point-blast/index.json。
