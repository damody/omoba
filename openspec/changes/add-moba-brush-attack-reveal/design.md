# 設計

MobaMatch保存依canonical entity與觀察陣營排序的有限期grant，兩秒模擬時間，普攻admission刷新，死亡／退休／過期清理，重播digest包含私有期限。原生普攻、script attack cue及lane AI共用記錄函式。非普攻技能不自動揭露。

不可用ForceShow，它會繞過牆、距離與stealth。新增RevealBrush規則，只略過brush條件；Wave B及即時unit item使用同一判定。Fog只照亮已合法揭露攻擊者所在格，不創造VisionSource或揭露整塊草叢，其他單位仍保留自己的遮蔽。強制隱藏與ServerOnly優先。

兩秒為明確遊戲規則，不宣稱複製特定LOL版本數值。採記憶體World與synthetic UE／IPC驗收；外部MVP 11項保留。OpenSpec CLI未安裝，手動維護artifact，不聲稱CLI validate成功。

技能tooltip沿已公開payload顯示目標類型、ally／enemy需求、施放距離與point落點半徑，使用WorldUnitsToCm換算。未學習、未知形狀、非point替換、非法浮點及退休不得保留舊數值。圖示、名稱與文字slot共用builder及既有可捲動tooltip host，不新增per-entity每frame UI。


野怪及附屬怪直接AI普攻也使用共用state參數函式；中立攻擊者對受擊的非中立team授權。測試用正式record_moba_jungle_damage仇恨入口，不直接修改private camp.target。兩秒期限以具名Fixed64常數表示。

