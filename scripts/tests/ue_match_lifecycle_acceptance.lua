-- Revalidate and retain a compact report from a real, already-cleaned UE run.
local source = debug.getinfo(1, "S").source:sub(2)
package.path = source:match("^(.*)[/\\]tests[/\\]") .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local path, json = b.lib("path"), b.lib("json")
local observations = require("ue_two_team_observation")
local requested_run = assert(arg[1], "usage: ue_match_lifecycle_acceptance.lua RUN_DIRECTORY")
local run = path.absolute(requested_run)
assert(#arg == 1, "expected exactly one run directory")
local prior = json.read(path.join(run, "unreal-ipc-smoke-report.json"))
assert(prior.success and prior.gameplay_mode == "single_lane" and #prior.teams == 2,
  "real single-lane two-team smoke did not pass")
local parity = observations.observe_parity(path.read(path.join(run, "server", "three-way-checkpoints.jsonl")), json.decode)
local report = {success = false, kind = "saved-live-unreal-match-lifecycle",
  result_ui_smoke = prior.result_ui_smoke or false, tick_rate_hz = prior.tick_rate_hz,
  run = run, terminal_parity_delay_ticks = 120, teams = {}}
for team = 1, 2 do
  local text = path.read(path.join(run, "logs", "ue-p" .. team .. ".stdout.log"))
  local abilities = observations.observe_abilities(text, team)
  local match = observations.observe_match(text, team)
  local move = observations.observe(text, team)
  assert(prior.teams[team].team_id == team and abilities.complete and match.complete
    and move.own_only and move.moved, "incomplete real UE lifecycle")
  assert(match.winner_team == prior.match_winner_team, "winner differs from source report")
  assert(parity[team].passed >= 6 and parity[team].last_tick >= match.finished_tick + 120,
    "missing post-finish three-way hash")
  for _, slot in ipairs(abilities.slots) do
    assert(parity[team].last_tick >= slot.result.tick, "missing post-cast hash")
  end
  local result_ui
  if prior.result_ui_smoke then
    assert(observations.observe_objective_attack(text, team).complete, 'missing authoritative objective attack ACK')
    result_ui = observations.observe_match_result_ui(text, team, team)
    assert(result_ui.complete and parity[team].last_tick >= result_ui.tick + 120,
      "missing authoritative native result UI or post-UI three-way hash")
    local screenshot = path.join(run, 'result-ui', 'team-' .. team .. '.png')
    assert(path.is_file(screenshot) and path.read(screenshot, true):sub(1, 8) == '\137PNG\r\n\26\n',
      "missing or invalid native result UI PNG")
    result_ui.screenshot = screenshot
    result_ui.screenshot_sha256 = b.lib('hash').sha256(screenshot)
  end
  report.teams[team] = {team_id = team, abilities = abilities, match_lifecycle = match,
    movement = move, match_result_ui = result_ui, three_way_parity = parity[team]}
  if prior.result_ui_smoke then
    report.teams[team].objective_attack = observations.observe_objective_attack(text, team)
  end
end
report.winner_team = prior.match_winner_team
report.success = true
local destination = path.join(b.root, "openspec", "changes", "build-unreal-rust-moba-framework",
  "evidence", "unreal-match-lifecycle")
path.mkdir_p(destination)
local output = path.join(destination, assert(run:match("([^/\\]+)$")) .. ".json")
json.write(output, report, true)
print("saved live Unreal match lifecycle: PASS; " .. output)
