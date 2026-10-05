# 正式配方 LAN host／remote-client 入口（2026-10-05）

## 本輪計畫與完成內容

1. 分開server監聽、client目的地與本機席位。既有localhost／所有真人預設保持；host可明確指定 --server-bind，--local-player可重複選擇本機真人。
2. --connect建立remote-client模式，必須提供host最終JSON與至少一個本機真人；禁止--hero／--interactive-selection或設定遠端server bind。配方、Bot、隊伍與規則不因本機篩選改變。
3. remote plan沒有server欄位，啟動與健康檢查不假造遠端PID；只啟動每席位runtime與presentation-only Unreal，失敗反向逐一清理本輪owned程序。runtime仍必須有精確player/team ready。
4. 使用既有正式Rust配方／設定預檢、compiled-content-only、部署guard、60Hz與localhost IPC。沒有runtime Lua、英雄專用C++／Blueprint或新的transport協定。

## 使用方式

以下位址是文件範例，執行時須換成host實際LAN介面IP。需先由host完成選角／產生最終JSON，將相同配方交给可信remote玩家；兩台使用相同內容版本與story設定。

```text
tools/lua/lua.exe scripts/run_moba_role_ue.lua --recipe FINAL.json --server-bind 192.168.1.10 --local-player 1
tools/lua/lua.exe scripts/run_moba_role_ue.lua --recipe FINAL.json --connect 192.168.1.10 --local-player 6
```

第一台啟動唯一server與player1的runtime／Unreal；第二台只啟動player6的runtime／Unreal。FINAL.json必須將這兩席位宣告為bot=false，不可選Bot或未知ID；--port預設57061，兩台須一致。多本機玩家重複--local-player；IPC埠仍以本機席位順序分配，兩台使用相同localhost埠不會相互衝突。

--prepare-only只保存設定／launch-plan而不啟動程序。--no-build仍檢查部署，不繞過內容或版本guard。remote的runtime建置不額外建omobab bin，但沿既有共同前端／內容建置入口，不能宣稱完全不建任何後端相依。

## 局部功能確認

- `scripts/test_moba_network_launch.lua` 新7/7：非法位址／remote重新選角／重複ID拒絕；真人席位篩選與不改來源；真正Rust設定預檢保存host與remote plan；mock成功、准入失敗、runtime退出及cleanup失敗。
- native預檢使用文件保留位址192.0.2.10，仅驗設定，不綁定該位址或開socket。兩份完整配方一致，host兩真人八Bot／本機一真人；remote僅player6／team2、沒有server欄位、IPC127.0.0.1:57062。
- 相鄰 `scripts/test_moba_role_launch.lua` 7/7成功，保留本機／同隊多人／全Botprepare／原owned cleanup行為；help入口成功。沒有執行真正server／runtime／Unreal程序，mock不當作LAN或畫面驗收。
- 首輪TOML空白斷言與Windows rg glob失誤記E246。沒有生成内容hash、Rust ABI、C ABI14、IPC4或selective wire5變更，不需本輪重跑native全套。

## 邊界與待完成

此入口只實作既有authority准入下的LAN程序配置，不是新的帳號認證／加密transport。普通最終JSON不是簽章證明；remote提供席位不授予server以外的英雄／隊伍控制權，准入仍由server處理。

未自動修改防火牆、設定路由或開放非localhost監聽。兩台實際LAN、斷線與renderer重連、遠端共用選角交付、完整UI／勝負／效能仍待最後整合。E224的Unreal engine/project BuildId界線未放寬。完整6.4保持未勾選，21/31不變，不stage／commit／push，不維護omfx。
