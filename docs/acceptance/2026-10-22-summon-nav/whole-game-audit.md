# 召喚落點與跟隨／導航盤點

接續全遊戲 17 系統盤點，單機優先補召喚空間行為。原召喚直接寫入要求座標；命令移動只有直線碰撞滑動，主人離開後沒有內容驅動返回。本輪採共用公開地形幾何，不使用英雄名稱特判，不引用隱藏敵人位置。

## 實作範圍

- Lua SummonStats／generated／runtime 增加 follow_start_distance、follow_stop_distance，0/0 禁用跟隨；拒絕非有限、負數、過大及 stop >= start 設定。雜賀內容設定 750／350，屬專案平衡值。
- 腳本接納生成前與正式生成時都檢查落點；公開 polygon 使用 fixed-point circle，友軍圓形體積包含批次先前召喚。要求點不可用時，按固定順序在每軸最多 512 範圍內找候選點，並檢查主人可達性；完全不可用則拒絕／退休 reservation，不把 spawn cue 當成功。
- 主動跟隨使用啟動／停止距離門檻，完整主人 Entity generation；明確 MoveTarget 優先直到完成。主人死亡或退休則取消自主返回，不自動接到新生命。原生導航 waypoint 快取 250ms；每一步仍做 swept-circle 檢查，不瞬移。
- 正式命令／返回沿共用 bounded static_next_waypoint；腳本追擊也接既有共用 public-terrain static_step_toward，保留 root／stun、buff 移速、pause 與期限退休規則。沒有新增每幀 UI 操作。
- navigation 狀態隨 SummonedUnit 納入既有 authority digest。停用跟隨的 legacy fixture 不會自動返回；公開披露 schema 不加入私有導航狀態。

## 驗證入口

新增 metadata 邊界、bounded placement／無合法位置、固定 60Hz 繞牆／步長／重播、命令覆寫／root／pause／owner generation／期限案例。正式 60Hz 雜賀技能在牆內要求陣形，驗證合法落點、主人移動後自主返回、過程不入牆與雙次重播一致。以最終驗收報告及封存 index 為通過依據。

## 仍缺

完整可見性複本生命期接受、更多召喚命令／隊形、主人重生後是否重新綁定的內容政策、分批生成在競爭落點的整組原子回滾、退休來源腳本 callback、通用施法排程／中斷、特殊技能幾何、objective 永久 stack／傳承／光環、營地分裂、塔皮／公告與美術。

原 MVP 33/44／11 外部 ACK／LAN／DPI／效能仍待驗；不新增網路對局、不重設配額，不以本輪單機通過宣稱全遊戲完成。
