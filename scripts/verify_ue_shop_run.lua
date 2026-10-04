-- Re-evaluate saved evidence without starting a match or sending gameplay input.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process, hash = b.lib('path'), b.lib('json'), b.lib('process'), b.lib('hash')
local observation = require('ue_two_team_observation')
local run_id = assert(arg[1], 'Usage: tools/lua/lua.exe scripts/verify_ue_shop_run.lua RUN_ID')
assert(#arg == 1 and run_id:match('^interactive%-ue%-%d+$'), 'invalid UE run ID')
local root = path.join(b.root, 'target', 'interactive-runs', run_id)
assert(path.is_directory(root), 'saved run directory does not exist')
local report = {success = false, kind = 'saved-unreal-shop-60hz-verification', run_id = run_id,
  reconstructed_from_original_evidence = true, teams = {}, processes = {}, evidence_hashes = {}}
local function read(relative)
  local file = path.join(root, relative)
  report.evidence_hashes[relative] = hash.sha256(file)
  return path.read(file)
end
local ok, failure = xpcall(function()
  report.shop_input_route = 'world_bridge_api'
  local launcher
  if path.is_file(path.join(root, 'unreal-ipc-smoke-report.json')) then
    launcher = json.decode(read('unreal-ipc-smoke-report.json'))
    if launcher.shop_input_route then
      assert(launcher.shop_input_route == 'slate_pointer' or launcher.shop_input_route == 'world_bridge_api', 'unknown shop input route')
      report.shop_input_route = launcher.shop_input_route
    end
  end
  local config = read('server-game.toml')
  report.tick_rate_hz = tonumber(config:match('STEP_FPS%s*=%s*(%d+)'))
  assert(report.tick_rate_hz == 60 and config:find('MATCH_GAMEPLAY_MODE%s*=%s*"single_lane"'), 'not a real 60Hz single_lane run')
  report.three_way_parity = observation.observe_parity(read('server/three-way-checkpoints.jsonl'), json.decode)
  for _, role in ipairs({'server', 'runtime-p1', 'runtime-p2', 'ue-p1', 'ue-p2'}) do
    local pid = assert(math.tointeger(tonumber(read(role .. '.pid'))), 'invalid saved PID')
    report.processes[#report.processes + 1] = require('ue_saved_process_identity').check(process.inspect(pid),pid,role,launcher and launcher.processes)
  end
  report.cleanup_verified = true
  for team = 1, 2 do
    local text = read('logs/ue-p' .. team .. '.stdout.log')
    local runtime = read('logs/runtime-p' .. team .. '.stderr.log')
    local shop = observation.observe_shop(text, team)
    local buttons = observation.observe_shop_buttons(text, team)
    if text:find('OM_SHOP_BUTTON ', 1, true) then report.shop_input_route = 'slate_pointer' end
    local movement = observation.observe(text, team)
    assert(shop.complete, 'missing ordered settled shop results/pending cleanup for team ' .. team)
    assert(movement.own_only and movement.moved, 'missing own-team rendered movement')
    assert(text:find('OM_MOBA_HUD player=' .. team .. ' phase=1 alive=1', 1, true), 'owner MOBA HUD missing')
    assert(text:find('OM_OWNER_ECONOMY player=' .. team .. ' gold=', 1, true), 'owner economy HUD missing')
    assert(text:find('OM_PRESENTATION issued approach move player=' .. team, 1, true), 'Unreal did not submit movement')
    local consumed = tonumber(runtime:match('renderer first consumed snapshot player=' .. team .. ' team=' .. team .. ' sequence=(%d+)'))
    assert(consumed and consumed > 0, 'missing actual renderer consumption')
    assert(runtime:find('secure shop protocol player=' .. team .. ' enabled=true', 1, true), 'shop protocol not negotiated')
    local move = json.decode(read('team-' .. team .. '-runtime/scripted-move-evidence.json'))
    assert(move.origin_x_raw ~= move.current_x_raw or move.origin_y_raw ~= move.current_y_raw, 'replica did not move')
    local safe = json.decode(read('team-' .. team .. '-runtime/filtered-world.latest.json'))
    assert(safe.team_id == team and safe.replica_tick >= shop.sold.tick, 'wrong team or stale replica')
    local parity = report.three_way_parity[team]
    assert(parity.passed >= 6 and parity.last_tick >= shop.sold.tick, 'no post-sale full hash checkpoint')
    report.teams[team] = {team_id = team, shop_transactions = shop, shop_buttons = buttons, movement = movement,
      replica_move = move, safe_tick = safe.replica_tick, consumed_snapshot_sequence = consumed}
  end
  if report.shop_input_route == 'slate_pointer' then
    for team = 1, 2 do assert(report.teams[team].shop_buttons.complete, 'missing exact native button callback/request evidence for team ' .. team) end
  end
  -- The opt-in checker streams original protobuf snapshots and verifies every
  -- Gold/inventory/immutable receipt, not just log strings or a final balance.
  local capture = process.run('cargo', {'test', '--manifest-path', path.join(b.root, 'omoba-client-runtime/Cargo.toml'),
    'real_unreal_single_lane_shop_transaction_capture', '--', '--ignored', '--nocapture'},
    {cwd = b.root, env = {OMOBA_UE_SHOP_CAPTURE_ROOT = root}, check = false})
  assert(capture.exit_code == 0, 'original protobuf capture verification failed: ' .. capture.stderr .. capture.stdout)
  report.capture_validation = {success = true, snapshots = {}, immutable_receipts_per_team = 3}
  for team, count, gold in capture.stdout:gmatch('real shop team=(%d+) snapshots=(%d+) immutable_receipts=3 gold=(%d+)') do
    report.capture_validation.snapshots[#report.capture_validation.snapshots + 1] = {
      team_id = tonumber(team), count = tonumber(count), final_gold = tonumber(gold)}
  end
  assert(#report.capture_validation.snapshots == 2, 'capture checker did not validate both teams')
  report.success = true
end, debug.traceback)
if not ok then report.error = tostring(failure) end
local output = path.join(root, 'unreal-shop-verification-report.json')
json.write(output, report, true)
print('[ue-shop-verification] success=' .. tostring(report.success) .. '; report: ' .. output)
assert(ok, failure)
