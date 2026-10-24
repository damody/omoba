## ADDED Requirements

### Requirement: team限定的標記與快捷鍵
系統 SHALL 支援G一般、V警戒、Alt或Ctrl＋左鍵八方向ping輪盤，包含撤退／推進／前往／全面進攻／協助／需要視野／敵人消失／敵方視野，以及minimap與world合法座標；內容以team-scoped權威／transport送達，有rate limit／mute設定。輪盤拖曳鬆開提交一次、右鍵取消，與施法／小地圖gesture依有效scope仲裁。

系統 SHALL 支援Alt-click自己可用的技能／道具cooldown或合法公開scoreboard資訊發出隊伍快捷訊息，禁止從隱藏敵人資料補消息內容。

#### Scenario: team隔離
- **WHEN** 玩家在minimap或world送ping
- **THEN** 同team收到一次合法標記與音效，敵team不收到；未公開target不補hidden entity資訊

#### Scenario: 公開目標投票
- **WHEN** 玩家對允許的公開大型objective送出投票ping
- **THEN** 同隊收到有界且可到期的投票與結果；target生命周期／重複投票／rate limit由權威處理，不能透露hidden objective狀態

### Requirement: 對局文字輸入與焦點
系統 SHALL 提供Enter team chat、Shift+Enter all chat（可設定禁用）、歷史顯示、mute與Esc關閉，並遵守input focus仲裁。快捷訊息與表情使用本專案原創內容。

#### Scenario: 文字輸入不觸發gameplay
- **WHEN** 玩家輸入包含QWER／Space／A的聊天文字並送出
- **THEN** 沒有施法／camera／AttackMove，文字送至所選有效channel；focus釋放後操作恢复，斷線時顯示失敗原因

#### Scenario: Runtime重建立即退休溝通狀態
- **WHEN** 直接停止或成功建立新Runtime，包含兩個UI tick之間的Stop／Start
- **THEN** 同步取消草稿與ping／emote gesture，清除舊歷史／objectives面板、標記、音訊去重與mute／結果token；不提交訊息、不在新Runtime恢復舊狀態，只在草稿持有焦點時歸還自己的viewport；保留偏好及可重用widget／owner，冪等Start保留有效live狀態
