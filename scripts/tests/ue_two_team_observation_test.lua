local source = debug.getinfo(1, "S").source:sub(2)
package.path = source:match("^(.*)[/\\]tests[/\\]") .. "/?.lua;" .. package.path
local observe = require("ue_two_team_observation").observe
local scoreboard = require('ue_two_team_observation').observe_scoreboard
local board = 'OM_SCOREBOARD_UI player=1 team=1 rows=2 tick=120 value=Scoreboard     Player     K / D / A|Team 1   P1   0 / 0 / 0|Team 2   P2   0 / 1 / 0'
assert(scoreboard(board,1,1).complete)
assert(not scoreboard(board,2,2).complete)
assert(not scoreboard(board:gsub('team=1','team=2'),1,1).complete)
assert(not scoreboard(board:gsub('tick=120','tick=119'),1,1).complete)
assert(not scoreboard(board:gsub('rows=2','rows=3'),1,1).complete)
assert(not scoreboard(board:gsub('P2','P1'),1,1).complete)
assert(not scoreboard(board:gsub('|Team 2.*',''),1,1).complete)
assert(not scoreboard(board:gsub('0 / 1 / 0','4294967296 / 1 / 0'),1,1).complete)
assert(not scoreboard(board:gsub('0 / 1 / 0','-1 / 1 / 0'),1,1).complete)
print('ue 1v1 scoreboard viewport observation: 9 scenarios passed')
local dead_board=board:gsub('OM_SCOREBOARD_UI','OM_SCOREBOARD_DEAD_UI')
assert(scoreboard(dead_board:gsub('player=1 team=1','player=2 team=2'),2,2,true).complete)
assert(not scoreboard(dead_board,1,1,true).complete)
assert(not scoreboard(board,1,1,true).complete)
assert(not scoreboard(dead_board:gsub('0 / 1 / 0','0 / 0 / 0'),2,2,true).complete)
print('ue dead scoreboard observation: 4 scenarios passed')
local objective = require('ue_two_team_observation').observe_objective_attack
local queued_attack = 'OM_MATCH_SMOKE objective player=1 action=attack input=8 target=4'
local attack_ack = '\nOM_MATCH_SMOKE objective_result player=1 input=8 status=0 tick=60'
assert(not objective(queued_attack, 1).complete)
assert(objective(queued_attack .. attack_ack, 1).complete)
assert(not objective(queued_attack .. attack_ack:gsub('status=0','status=4'), 1).complete)
assert(not objective(queued_attack .. attack_ack:gsub('input=8','input=9'), 1).complete)
assert(not objective(queued_attack .. attack_ack, 2).complete)
print('ue objective authority ACK: 5 scenarios passed')
local still = "OM_PRESENTATION seq=1 heroes=1 [5 k=1 o=1 c=1 (-1320,-1100)]"
local moved = "OM_PRESENTATION seq=2 heroes=1 [5 k=1 o=1 c=1 (-1319.5,-1099)]"
local control = "OM_PRESENTATION issued approach move player=1 target=(-40000,-40000)"
assert(not observe(still .. "\n" .. control, 1).moved)
local result = observe(still .. "\n" .. moved, 1)
assert(result.frames == 2 and result.own_only and result.moved)
assert(result.origin.x == -1320 and result.latest.x == -1319.5)
assert(not observe(still .. "\n" .. moved, 2).moved)
assert(not observe("OM_PRESENTATION seq=1 heroes=10 [5 k=1 o=1 c=1 (1,2)]", 1).own_only)
assert(not observe("OM_PRESENTATION seq=1 heroes=1 [5 k=1 o=1 c=1 (--,2)]", 1).moved)
local p2 = observe("OM_PRESENTATION seq=1 heroes=1 [5 k=1 o=2 c=1 (1320,1100)]\n" ..
  "OM_PRESENTATION seq=2 heroes=1 [5 k=1 o=2 c=1 (1319,1099)]", 2)
assert(p2.own_only and p2.moved)
print("ue_two_team_observation: 6 scenarios passed")
local observe_abilities = require("ue_two_team_observation").observe_abilities
local ability_lines = {}
for slot = 0, 3 do
  local id = slot + 10
  ability_lines[#ability_lines + 1] = string.format("OM_ABILITY_SMOKE queued player=1 slot=%d input=%d target=2", slot, id)
  ability_lines[#ability_lines + 1] = string.format("OM_ABILITY_SMOKE result player=1 slot=%d input=%d status=0 tick=42", slot, id)
  ability_lines[#ability_lines + 1] = string.format("OM_ABILITY_SMOKE cooldown player=1 slot=%d input=%d remaining=3.000 tick=43", slot, id)
end
local complete = table.concat(ability_lines, "\n")
assert(observe_abilities(complete, 1).complete)
assert(not observe_abilities(complete, 2).complete)
assert(not observe_abilities("OM_ABILITY_SMOKE complete player=1 applied=15 cooldown=15", 1).complete)
assert(not observe_abilities(complete:gsub("status=0", "status=2"), 1).complete)
assert(not observe_abilities(complete:gsub("remaining=3.000", "remaining=0.000"), 1).complete)
assert(not observe_abilities(complete:gsub("input=11 status", "input=99 status"), 1).complete)
assert(not observe_abilities(complete:gsub("input=11", "input=10"), 1).complete)
local rejected = complete:gsub("input=12 status=0", "input=12 status=4")
local retry = "\nOM_ABILITY_SMOKE queued player=1 slot=2 input=20 target=9\n" ..
  "OM_ABILITY_SMOKE result player=1 slot=2 input=20 status=0 tick=60\n"
assert(not observe_abilities(rejected .. retry, 1).complete) -- old ID cooldown is not evidence
assert(observe_abilities(rejected .. retry .. "OM_ABILITY_SMOKE cooldown player=1 slot=2 input=20 remaining=3.000 tick=61", 1).complete)
print("ue ability observation: 9 scenarios passed")
local parity_observe = require("ue_two_team_observation").observe_parity
local function decode_fixture(line)
  return {team_id = 1, verdict = line, expected = "h", external_runtime_pre_repair = "h",
    external_runtime_post_repair = "h", pre_repair_parity = true, post_repair_parity = true,
    external_runtime_frame_hash = "f", observer_frame_hash = "f", replica_tick = 120}
end
assert(parity_observe('PASS\n{"verdict":"UNVERIFIED"}\nPASS', decode_fixture)[1].passed == 1)
assert(not pcall(parity_observe, "FAIL\n", decode_fixture))
assert(not pcall(parity_observe, "PASS\n", function(line)
  local row = decode_fixture(line); row.external_runtime_pre_repair = "wrong"; return row
end))
print("ue three-way observation: 3 scenarios passed")
local observe_match = require("ue_two_team_observation").observe_match
local lifecycle = "OM_MATCH_STATE player=1 phase=1 alive=1 winner=0 hero=4 epoch=1 respawn=0.000 tick=10\n" ..
  "OM_MATCH_STATE player=1 phase=1 alive=0 winner=0 hero=0 epoch=0 respawn=5.000 tick=20\n" ..
  "OM_MATCH_STATE player=1 phase=1 alive=1 winner=0 hero=4 epoch=2 respawn=0.000 tick=30\n" ..
  "OM_MATCH_STATE player=1 phase=2 alive=1 winner=1 hero=4 epoch=2 respawn=0.000 tick=40"
assert(observe_match(lifecycle, 1).complete)
assert(not observe_match(lifecycle, 2).complete)
assert(not observe_match(lifecycle:gsub("epoch=2", "epoch=1"), 1).complete)
assert(not observe_match(lifecycle:gsub("respawn=5.000", "respawn=0.000"), 1).complete)
assert(not observe_match(lifecycle:gsub("alive=0", "alive=1"), 1).complete)
assert(not observe_match(lifecycle:gsub("tick=30", "tick=19"), 1).complete)
assert(not observe_match(lifecycle:gsub("phase=2", "phase=3"), 1).complete)
assert(not observe_match(lifecycle:gsub("winner=1", "winner=3"), 1).complete)
print("ue match lifecycle observation: 8 scenarios passed")
if arg[1] then
  local bootstrap = require("_bootstrap")
  local path, json = bootstrap.lib("path"), bootstrap.lib("json")
  local run = path.absolute(arg[1])
  local source_report = json.read(path.join(run, "unreal-ipc-smoke-report.json"))
  assert(source_report.success and #source_report.teams == 2, "source live smoke did not pass")
  local report = {success = false, kind = "saved-live-unreal-movement-observation", run = run, teams = {}}
  for team = 1, 2 do
    local prior = source_report.teams[team]
    local observation = observe(path.read(path.join(run, "logs", "ue-p" .. team .. ".stdout.log")), team)
    assert(prior.team_id == team and prior.own_only_observed and observation.own_only and observation.moved)
    assert(prior.move.current_x_raw ~= prior.move.origin_x_raw or prior.move.current_y_raw ~= prior.move.origin_y_raw)
    report.teams[team] = {team_id = team, unreal_movement = observation, move = prior.move}
  end
  report.success = true
  local output = path.join(run, "unreal-ipc-observation-report.json")
  json.write(output, report, true)
  print("saved live Unreal movement: PASS; " .. output)
end
