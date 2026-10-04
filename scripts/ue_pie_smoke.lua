-- Reproducible Editor smoke; gameplay match acceptance remains a separate gate.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local process = bootstrap.lib('process')
local platform = bootstrap.lib('platform')
local json = bootstrap.lib('json')
local work = path.join(bootstrap.root, 'omfue', 'Saved', 'McpAutomation')
path.mkdir_p(work)
local report = {kind = 'editor-native-hero-smoke', results = {}, success = false}
local started = false
local heroes = assert(loadfile(path.join(bootstrap.root, 'scripts/lua_data/templates/heroes.lua')))()({})
local native_scale
for _, hero in ipairs(heroes) do
  if hero.id == 'saika_magoichi' then
    native_scale = hero.ue and hero.ue.native_visual and hero.ue.native_visual.scale
      or (hero.render and hero.render.scale and hero.render.scale > 0 and hero.render.scale * 100) or 1
  end
end
assert(native_scale and native_scale > 0 and native_scale < math.huge, 'invalid native test scale')
report.native_expected_scale = native_scale

local function call(name, arguments, label)
  local file = path.join(work, label .. '.json')
  local result = process.run(platform.lua_executable, {
    path.join(bootstrap.root, 'scripts', 'ue_mcp.lua'), '--tool', name,
    '--arguments', json.encode(json.object(arguments)), '--out', file,
  }, {cwd = bootstrap.root, check = false})
  assert(result.exit_code == 0, name .. ' failed: ' .. result.stderr .. result.stdout)
  local envelope = json.read(file)
  local detail
  for _, content in ipairs(envelope.content or {}) do
    if content.type == 'text' then
      local parsed, value = pcall(json.decode, content.text)
      if parsed and type(value) == 'table' then detail = value; break end
    end
  end
  assert(detail, name .. ' returned no structured result')
  report.results[label] = detail
  print('[ue-pie-smoke] ' .. name .. ': ok')
  return detail
end

local ok, error_message = xpcall(function()
  -- Editor automation and PIE share mutable world/session state. Do not overlap.
  call('enable_extension', {extension_id = 'Testing', enabled = true}, 'testing-enabled')
  local automation = call('get_automation_test_results', {}, 'automation-before')
  assert((automation.running or 0) == 0, 'Editor automation is active; finish it before PIE smoke')
  local before = call('get_pie_status', {}, 'pie-before')
  if not before.pie_running then
    call('begin_play_in_editor', {num_clients = 1, net_mode = 'standalone'}, 'pie-start')
    started = true
  end
  local running
  for _ = 1, 5 do
    running = call('get_pie_status', {}, 'pie-running')
    if running.pie_running and running.has_player then break end
  end
  assert(running.pie_running and running.has_player, 'PIE did not produce a player world')
  -- Capture first so subsequent test activity is available in get_pie_log.
  call('get_pie_log', {line_count = 20}, 'pie-log-start')
  local native_location = {x = 120, y = -160, z = 120}
  if started then
    local actors = call('get_pie_actors', {}, 'pie-camera-actors')
    local camera, controller
    for _, actor in ipairs(actors.actors or {}) do
      if actor.class == 'BP_RtsCameraPawn_C' or actor.class == 'OmRtsCameraPawn' then camera = actor.name end
      if actor.class == 'BP_PlayerController_C' or actor.class == 'OmPlayerController' then controller = actor.name end
    end
    assert(camera and controller, 'native visual smoke requires the RTS player camera')
    call('call_pie_blueprint_function', {actor_label = camera, function_name = 'SetUiInputGuard',
      params = {bInUiCapturesPointer = true}}, 'pie-camera-guard')
    call('call_pie_blueprint_function', {actor_label = camera, function_name = 'SetZoomDistance',
      params = {NewDistance = 700}}, 'pie-camera-zoom')
    call('wait_for_pie_event', {seconds = 1}, 'pie-camera-settle')
    local view = call('call_pie_blueprint_function', {actor_label = controller,
      function_name = 'GetPlayerViewPoint'}, 'pie-camera-view')
    local location, rotation = assert(view.outputs.Location), assert(view.outputs.Rotation)
    local function field(source, name)
      return assert(tonumber(source:match(name .. '=([%d%.%-+eE]+)')), 'invalid camera ' .. name)
    end
    local pitch, yaw = math.rad(field(rotation, 'Pitch')), math.rad(field(rotation, 'Yaw'))
    assert(math.sin(pitch) < -0.1, 'camera must face the test ground plane')
    local distance = (native_location.z - field(location, 'Z')) / math.sin(pitch)
    assert(distance > 0, 'test plane is behind the camera')
    -- Keep test art separate from the map's pre-existing hero at the view centre.
    native_location.x = field(location, 'X') + distance * math.cos(pitch) * math.cos(yaw) + 250
    native_location.y = field(location, 'Y') + distance * math.cos(pitch) * math.sin(yaw) - 250
  end
  local sequence = call('run_pie_test_sequence', {steps = {
    {action = 'spawn_pie_actor', class_path = '/Script/OmGenerated.OmHeroTrainingLuminary',
      location = {x = 300, y = 100, z = 100}, as = 'luminary'},
    {action = 'assert', actor_label = '$luminary', property = 'location.x',
      value = 300, tolerance = 0.5, label = 'generated native hero spawns'},
    {action = 'get_pie_actor_state', actor_label = '$luminary'},
    {action = 'spawn_pie_actor', class_path = '/Script/OmGenerated.OmHeroSaikaMagoichi',
      location = native_location, as = 'native_saika'},
    {action = 'assert', actor_label = '$native_saika', property = 'NativeHeroMesh.scale.x',
      value = native_scale, tolerance = 0.001, label = 'native art loads generated Lua transform'},
    {action = 'get_pie_actor_state', actor_label = '$native_saika'},
    {action = 'wait_for_pie_event', seconds = 1.0},
    {action = 'call_pie_blueprint_function', actor_label = '$native_saika',
      function_name = 'NativeHeroMesh.WasRecentlyRendered', params = {Tolerance = 1}},
    {action = 'take_pie_screenshot', file_path = path.join(work, 'pie-native-hero.png')},
    {action = 'spawn_pie_actor', class_path = '/Script/OmEditor.OmAutomationMemoryProbe', as = 'memory_probe'},
    {action = 'call_pie_blueprint_function', actor_label = '$memory_probe', function_name = 'PublishMemoryProbe',
      params = {WorldLocation = {x = native_location.x - 500, y = native_location.y + 500, z = 60}}},
    {action = 'call_pie_blueprint_function', actor_label = '$memory_probe', function_name = 'GetBridgeStats'},
    {action = 'wait_for_pie_event', seconds = 1.0},
    {action = 'take_pie_screenshot', file_path = path.join(work, 'pie-remembered-ghost.png')},
    {action = 'call_pie_blueprint_function', actor_label = '$memory_probe',
      function_name = 'RememberedGhosts.WasRecentlyRendered', params = {Tolerance = 1}},
    {action = 'call_pie_blueprint_function', actor_label = '$memory_probe', function_name = 'ClearMemoryProbe'},
    {action = 'call_pie_blueprint_function', actor_label = '$memory_probe', function_name = 'GetBridgeStats'},
  }}, 'pie-native-sequence')
  assert(sequence.verdict == 'PASS' and sequence.asserts_failed == 0 and
    sequence.asserts_could_not_tell == 0, 'native hero sequence did not pass')
  if started then
    local rendered
    local ghost_rendered
    local ghost_counts = {}
    for _, step in ipairs(sequence.results or {}) do
      if step.action == 'call_pie_blueprint_function' then
        local result = json.decode(step.result)
        if result.function_name == 'NativeHeroMesh.WasRecentlyRendered' then
          rendered = result.return_value == 'True'
        elseif result.function_name == 'RememberedGhosts.WasRecentlyRendered' then
          ghost_rendered = result.return_value == 'True'
        elseif result.function_name == 'GetBridgeStats' then
          assert(result.return_type == 'FOmWorldBridgeStats' and result.success, 'invalid memory diagnostics')
          -- Unreal ExportText omits fields equal to the struct default (zero).
          local count = tonumber(result.return_value:match('RememberedGhostCount=(%d+)')) or 0
          ghost_counts[#ghost_counts + 1] = count
        end
      end
    end
    assert(rendered, 'native mesh spawned but was not rendered in the framed viewport')
    report.native_mesh_rendered = true
    assert(ghost_rendered, 'remembered marker was not rendered in the framed viewport')
    assert(ghost_counts[1] == 1 and ghost_counts[2] == 0, 'memory fixture did not publish/clear exactly one ghost')
    report.remembered_ghost_rendered = true
    report.remembered_ghost_counts = ghost_counts
  end
  call('get_pie_log', {line_count = 40}, 'pie-log')
  local screenshot = call('take_pie_screenshot', {
    file_path = path.join(work, 'pie-smoke.png'),
  }, 'pie-screenshot')
  assert(path.is_file(screenshot.file_path), 'PIE screenshot file missing')
  report.success = true
end, debug.traceback)

if started then
  local stopped, stop_error = pcall(function()
    local status_before_stop = call('get_pie_status', {}, 'pie-before-stop')
    if status_before_stop.pie_running then call('stop_play_in_editor', {}, 'pie-stop') end
    local status = call('get_pie_status', {}, 'pie-after')
    assert(not status.pie_running, 'PIE did not stop')
  end)
  if not stopped then
    report.cleanup_error = tostring(stop_error)
    if ok then error_message = stop_error end -- Preserve an earlier startup/test failure.
    ok = false; report.success = false
  end
end
if not ok then report.error = tostring(error_message); report.success = false end
local report_file = path.join(work, 'pie-smoke-report.json')
json.write(report_file, report, true)
print('[ue-pie-smoke] report: ' .. report_file)
assert(ok, error_message)
