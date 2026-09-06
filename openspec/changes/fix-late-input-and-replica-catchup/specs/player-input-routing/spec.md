## MODIFIED Requirements

### Requirement: TD_STRESS move input remains responsive at 400+ creeps

In TD_STRESS、FOG_2TEAM_DEMO或等價高負載測試中，local `MoveTo` input SHALL被server接受、在一秒grace內retarget或明確拒絕。短暫client replica stall MUST NOT造成後續所有MoveTo永久失效。若輸入被接受或retarget，對應 `input_id` SHALL出現在生效tick的authoritative frame與client applied input metadata。

#### Scenario: MoveTo under stress avoids permanent rejection
- **WHEN**client replica因高負載暫時落後10到120 tick
- **AND**玩家送出 `MoveTo`
- **THEN**omb接受或retarget該輸入到下一個authoritative tick
- **AND**英雄執行該移動，不會因一次backlog後永久停止接受新MoveTo

#### Scenario: late input remains explicit
- **WHEN**input在一秒grace內被retarget或超過grace被拒絕
- **THEN**omb log包含player id、input id、target tick、current/effective tick與late-by tick
- **AND**結果可與transport starvation或client-side pending backlog區分

#### Scenario: 超過 grace 維持 fail closed
- **WHEN**座標input落後超過一秒
- **THEN**omb拒絕該input且不改變authoritative world
- **AND**server不使用無上限retarget掩蓋失控client
