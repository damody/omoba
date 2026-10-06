-- Pure parser for formal OM_PERF / OM_PERF_ENABLE lines.
-- Thresholds are never invented here. Mean is the integer floor of sum/samples, not a p95.
local json = require('tools.lua.lib.json')

local M = {}
local EXACT_MAX = 9007199254740991

M.definitions = {
  server_tick_compute = {
    component = 'server', metric = 'tick_compute', unit = 'ns',
    scope = 'state_tick_excluding_transport_send',
    measures = 'Completed State::tick wall time after subtracting every reliable_send_with_watchdog elapsed interval.',
    excludes = 'Clock::tick scheduler sleep, transport send_timeout, warmup, paused, and finished ticks.',
    ['not'] = 'Not the debug run+dispatch+outcomes partial sum, not a GPU time, and not a p95.',
  },
  client_replica_step = {
    component = 'client_runtime', metric = 'replica_step', unit = 'ns',
    scope = 'apply_encoded_frame',
    measures = 'Wall time of one successful replica apply_encoded_frame call, stopped before evidence IO and presentation.',
    ['not'] = 'Not KCP wire time, not evidence checkpoint IO, and not a p95.',
  },
  presentation_ipc_send = {
    component = 'presentation_ipc', metric = 'wire_bytes', direction = 'send', unit = 'byte',
    scope = 'localhost_tcp_length_prefixed_frame',
    measures = 'Bytes of presentation frames fully written on one localhost TCP connection, including the 4-byte length prefix.',
    rate = 'integer floor(bytes * 1000000000 / window_duration_ns), bytes per second over the sample window.',
    rate_is_not = 'Not a game-frame interval and not a KCP byte count.',
  },
  presentation_ipc_receive = {
    component = 'presentation_ipc', metric = 'wire_bytes', direction = 'receive', unit = 'byte',
    scope = 'localhost_tcp_length_prefixed_frame',
    measures = 'Bytes of presentation frames fully read and accepted on that same connection, including the length prefix.',
    rate = 'integer floor(bytes * 1000000000 / window_duration_ns).',
    rate_is_not = 'Not a game-frame interval. A failed or truncated read is not a sample.',
  },
  unreal_game_frame_interval = {
    component = 'unreal', metric = 'game_frame_interval', unit = 'ns',
    scope = 'game_thread_delta_seconds',
    measures = 'Unreal game-thread Tick DeltaSeconds converted to nanoseconds while presentation runtime is active.',
    ['not'] = 'Not GPU time, not render-thread time, and not a byte rate. Startup and paused gaps are not samples.',
  },
  unreal_presentation_work = {
    component = 'unreal', metric = 'presentation_work', unit = 'ns',
    scope = 'actor_tick_fplatformtime',
    measures = 'FPlatformTime duration of the active world-bridge presentation Tick body.',
    ['not'] = 'Not the frame interval and not GPU time.',
  },
}

local ORDER = {
  'server_tick_compute', 'client_replica_step', 'presentation_ipc_send',
  'presentation_ipc_receive', 'unreal_game_frame_interval', 'unreal_presentation_work',
}
local FILE_OF = {
  server_tick_compute = 'server',
  client_replica_step = 'runtime',
  presentation_ipc_send = 'runtime',
  presentation_ipc_receive = 'runtime',
  unreal_game_frame_interval = 'unreal',
  unreal_presentation_work = 'unreal',
}
local ENABLE_OF = {
  server = 'server',
  client_runtime = 'runtime',
  presentation_ipc = 'runtime',
  unreal = 'unreal',
}

local function integer(value)
  if type(value) ~= 'number' then return nil end
  if value ~= value or value == math.huge or value == -math.huge then return nil end
  if value < 0 or value % 1 ~= 0 or value > EXACT_MAX then return nil end
  return value
end

local function text_list(value)
  if type(value) ~= 'table' then return nil end
  local out = {}
  for index, item in ipairs(value) do
    if type(item) ~= 'string' then return nil end
    out[index] = item
  end
  return out
end

local function has_text(list, wanted)
  if not list then return false end
  for _, item in ipairs(list) do if item == wanted then return true end end
  return false
end

local function push(list, value)
  list[#list + 1] = value
end

local function json_at(line, marker)
  local at = line:find(marker, 1, true)
  if not at then return nil end
  local body = line:sub(at + #marker):gsub('%s+$', '')
  local ok, value = pcall(json.decode, body)
  if not ok or type(value) ~= 'table' then return nil, 'invalid JSON after ' .. marker end
  return value
end

local function segment_of(record)
  if record.component == 'server' and record.metric == 'tick_compute' then return 'server_tick_compute' end
  if record.component == 'client_runtime' and record.metric == 'replica_step' then return 'client_replica_step' end
  if record.component == 'presentation_ipc' and record.metric == 'wire_bytes' and record.direction == 'send' then
    return 'presentation_ipc_send'
  end
  if record.component == 'presentation_ipc' and record.metric == 'wire_bytes' and record.direction == 'receive' then
    return 'presentation_ipc_receive'
  end
  if record.component == 'unreal' and record.metric == 'game_frame_interval' then return 'unreal_game_frame_interval' end
  if record.component == 'unreal' and record.metric == 'presentation_work' then return 'unreal_presentation_work' end
  return nil
end

local function check_enable(role, record)
  if integer(record.v) ~= 1 or integer(record.window_samples) ~= 60 then
    return 'enable contract is not v=1 window_samples=60'
  end
  local owner = ENABLE_OF[record.component]
  if owner == nil then return 'unknown enable component ' .. tostring(record.component) end
  if owner ~= role then return record.component .. ' enable found in ' .. role .. ' log' end
  if record.component == 'server' then
    if record.metric ~= 'tick_compute' or record.scope ~= M.definitions.server_tick_compute.scope
        or record.level ~= 'info' or record.hash_input ~= false
        or type(record.excludes) ~= 'string'
        or not record.excludes:find('scheduler_sleep', 1, true)
        or not record.excludes:find('reliable_send_timeout', 1, true) then
      return 'server enable scope or excludes mismatch'
    end
  elseif record.component == 'client_runtime' then
    if record.metric ~= 'replica_step' or record.scope ~= M.definitions.client_replica_step.scope
        or record.level ~= 'info' or type(record.excludes) ~= 'string'
        or not record.excludes:find('evidence_io', 1, true)
        or integer(record.player_id) == nil or integer(record.team_id) == nil then
      return 'client enable scope or identity mismatch'
    end
  elseif record.component == 'presentation_ipc' then
    if record.metric ~= 'wire_bytes' or record.scope ~= M.definitions.presentation_ipc_send.scope
        or record.level ~= 'info' or record.transport ~= 'localhost_tcp' or record.not_transport ~= 'kcp'
        or integer(record.player_id) == nil or integer(record.team_id) == nil
        or integer(record.connection) == nil then
      return 'presentation IPC enable is not localhost TCP'
    end
  elseif record.component == 'unreal' then
    local metrics, scopes = text_list(record.metrics), text_list(record.scopes)
    if record.level ~= 'Display' or record.frame_interval_is ~= 'game_thread_delta_seconds'
        or record.frame_interval_is_not ~= 'gpu_or_render_thread'
        or not has_text(metrics, 'game_frame_interval') or not has_text(metrics, 'presentation_work')
        or not has_text(scopes, 'game_thread_delta_seconds') or not has_text(scopes, 'actor_tick_fplatformtime')
        or integer(record.player_id) == nil or integer(record.team_id) == nil then
      return 'unreal enable does not separate frame interval from GPU time'
    end
  end
  return nil
end

local function exact_add(left, right)
  if integer(left) == nil or integer(right) == nil then return nil end
  if right > EXACT_MAX - left then return nil end
  return left + right
end

-- floor(a * b / d) in at most 53 binary steps. Intermediates stay inside 64-bit integers.
-- high is the greatest power of two that is <= b. Capping it at EXACT_MAX//2 drops bit 52.
-- The returned value is still an exact JSON integer, or nil when it is not.
local function muldiv_floor(a, b, d)
  if integer(a) == nil or integer(b) == nil or integer(d) == nil or d < 1 then return nil end
  local quotient = a // d
  if quotient ~= 0 and b > EXACT_MAX // quotient then return nil end
  local whole = quotient * b
  local remainder = a % d
  if remainder == 0 then return whole end
  local high = 1
  while high <= b // 2 do high = high * 2 end
  local acc, rem, steps = 0, 0, 0
  while true do
    steps = steps + 1
    if steps > 64 then return nil end
    local doubled = rem * 2
    if (b // high) % 2 == 1 then doubled = doubled + remainder end
    acc = acc * 2 + doubled // d
    rem = doubled % d
    if high == 1 then break end
    high = high // 2
  end
  local result = whole + acc
  if integer(result) == nil then return nil end
  return result
end

local function rate_of(bytes, duration)
  if duration == 0 then return nil end
  return muldiv_floor(bytes, 1000000000, duration)
end

local function check_sample(role, record)
  local id = segment_of(record)
  if not id then return nil, 'unknown or wrong-scope OM_PERF record' end
  if FILE_OF[id] ~= role then return nil, id .. ' found in ' .. role .. ' log' end
  local def = M.definitions[id]
  if integer(record.v) ~= 1 or record.scope ~= def.scope or record.unit ~= def.unit then
    return nil, id .. ' scope or unit mismatch'
  end
  local samples, count = integer(record.samples), integer(record.count)
  local sum, max, mean = integer(record.sum), integer(record.max), integer(record.mean)
  local duration = integer(record.window_duration_ns)
  if not samples or samples < 1 or count ~= samples or not sum or not max or not mean or not duration then
    return nil, id .. ' has an empty, non-integer, or non-finite window'
  end
  if samples >= 1 and (sum - 1) // samples + 1 > max then
    return nil, id .. ' sum exceeds samples*max'
  end
  if mean ~= sum // samples or max > sum then return nil, id .. ' mean is not integer floor(sum/samples)' end
  if record.mean_definition ~= 'integer_floor_sum_over_samples' or record.not_a_statistic ~= 'p95' or record.p95 ~= nil then
    return nil, id .. ' must not present mean as p95'
  end
  local sample = {
    samples = samples, count = count, sum = sum, max = max, mean = mean,
    window_duration_ns = duration, player_id = integer(record.player_id),
    team_id = integer(record.team_id), connection = integer(record.connection),
  }
  if id == 'server_tick_compute' then
    if record.rate_bytes_per_s ~= nil or record.interval_is ~= nil then
      return nil, 'server tick must not carry a byte rate or frame interval'
    end
  elseif id == 'client_replica_step' then
    if sample.player_id == nil or sample.team_id == nil or record.rate_bytes_per_s ~= nil then
      return nil, 'client step requires player and team and no byte rate'
    end
  elseif id == 'presentation_ipc_send' or id == 'presentation_ipc_receive' then
    local messages, bytes = integer(record.messages), integer(record.bytes)
    if messages ~= samples or bytes ~= sum or sample.player_id == nil or sample.team_id == nil
        or sample.connection == nil or record.rate_definition ~= 'integer_floor_bytes_per_window_second'
        or record.rate_is_not ~= 'frame_interval' or record.interval_is ~= nil then
      return nil, id .. ' must be a byte window, not a frame interval'
    end
    sample.messages, sample.bytes = messages, bytes
    if duration == 0 then
      if record.rate_bytes_per_s ~= json.null then return nil, id .. ' zero-duration rate must be null' end
      sample.rate_bytes_per_s = nil
    else
      local rate = muldiv_floor(bytes, 1000000000, duration)
      if rate == nil then return nil, id .. ' byte rate is outside exact integer range' end
      if integer(record.rate_bytes_per_s) ~= rate then return nil, id .. ' rate does not match bytes/duration' end
      sample.rate_bytes_per_s = rate
    end
  else
    if record.rate_bytes_per_s ~= nil then return nil, id .. ' is not a byte rate' end
    if sample.player_id == nil or sample.team_id == nil then return nil, id .. ' requires player and team' end
    if id == 'unreal_game_frame_interval' then
      if record.interval_is ~= 'game_thread_delta_seconds' or record.interval_is_not ~= 'gpu_or_render_thread' then
        return nil, 'frame interval scope was claimed as GPU or render-thread time'
      end
    elseif record.work_is ~= 'actor_tick_fplatformtime' or record.work_is_not ~= 'gpu_or_render_thread' then
      return nil, 'presentation work scope mismatch'
    end
  end
  return sample
end

local function note_enable_identity(record, identities)
  if record.component == 'server' then return nil end
  local player_id, team_id = integer(record.player_id), integer(record.team_id)
  local slot = identities[record.component]
  if not slot then
    slot = {player_id = player_id, team_id = team_id, connections = {}}
    identities[record.component] = slot
  elseif slot.player_id ~= player_id or slot.team_id ~= team_id then
    return record.component .. ' enable identity does not match its other enable'
  end
  if record.component == 'presentation_ipc' then
    slot.connections[integer(record.connection)] = true
  end
  return nil
end

local function same_capture_identity(segments, identities, errors)
  local owners = {
    client_replica_step = 'client_runtime',
    presentation_ipc_send = 'presentation_ipc',
    presentation_ipc_receive = 'presentation_ipc',
    unreal_game_frame_interval = 'unreal',
    unreal_presentation_work = 'unreal',
  }
  local capture_player, capture_team
  for _, id in ipairs(ORDER) do
    local component = owners[id]
    local segment = component and segments[id] or nil
    if segment and segment.player_id ~= nil then
      local enable = identities[component]
      if not enable or enable.player_id ~= segment.player_id or enable.team_id ~= segment.team_id then
        push(errors, component .. ' enable identity does not match its summary')
      end
      if component == 'presentation_ipc' then
        for _, connection in ipairs(segment.connections or {}) do
          if not enable or not enable.connections[connection] then
            push(errors, 'presentation IPC connection ' .. connection .. ' has no matching enable')
          end
        end
      end
      if capture_player == nil then
        capture_player, capture_team = segment.player_id, segment.team_id
      elseif capture_player ~= segment.player_id or capture_team ~= segment.team_id then
        push(errors, 'client, presentation IPC, and unreal are not the same player and team')
      end
    end
  end
end

local function comparison_array(value)
  if type(value) ~= 'table' or value == json.null or #value < 1 then return false end
  local count = 0
  for key in pairs(value) do
    if type(key) ~= 'number' or key < 1 or key % 1 ~= 0 then return false end
    count = count + 1
  end
  return count == #value
end

local function add_identity(bucket, sample, errors, id)
  for _, key in ipairs({'player_id', 'team_id'}) do
    if sample[key] ~= nil then
      if bucket[key] == nil then bucket[key] = sample[key]
      elseif bucket[key] ~= sample[key] then push(errors, id .. ' mixes ' .. key) end
    end
  end
  if sample.connection ~= nil then
    bucket.connections = bucket.connections or {}
    bucket.connections[sample.connection] = true
  end
end

function M.summarize(logs, paths, baseline)
  local errors, missing, enabled, identities = {}, {}, {}, {}
  local buckets = {}
  for _, id in ipairs(ORDER) do buckets[id] = nil end
  for _, role in ipairs({'server', 'runtime', 'unreal'}) do
    local text = logs and logs[role]
    if type(text) ~= 'string' then push(errors, 'missing ' .. role .. ' log text') end
    for line in tostring(text or ''):gmatch('[^\n]+') do
      line = line:gsub('\r$', '')
      local enable_record, enable_err = json_at(line, 'OM_PERF_ENABLE ')
      local sample_record, sample_err = json_at(line, 'OM_PERF ')
      if enable_err then push(errors, role .. ': ' .. enable_err)
      elseif enable_record then
        local problem = check_enable(role, enable_record)
        if problem then push(errors, role .. ': ' .. problem)
        else
          enabled[enable_record.component] = true
          local identity_problem = note_enable_identity(enable_record, identities)
          if identity_problem then push(errors, role .. ': ' .. identity_problem) end
        end
      end
      if sample_err then push(errors, role .. ': ' .. sample_err)
      elseif sample_record then
        local sample, problem = check_sample(role, sample_record)
        if problem then push(errors, role .. ': ' .. problem)
        else
          local id = segment_of(sample_record)
          local bucket = buckets[id]
          if not bucket then
            bucket = {windows = 0, samples = 0, count = 0, sum = 0, max = 0, window_duration_ns = 0, bytes = 0, messages = 0}
            buckets[id] = bucket
          end
          if not bucket.broken then
            local next_windows = exact_add(bucket.windows, 1)
            local next_samples = exact_add(bucket.samples, sample.samples)
            local next_count = exact_add(bucket.count, sample.count)
            local next_sum = exact_add(bucket.sum, sample.sum)
            local next_duration = exact_add(bucket.window_duration_ns, sample.window_duration_ns)
            local next_bytes = exact_add(bucket.bytes, sample.bytes or 0)
            local next_messages = exact_add(bucket.messages, sample.messages or 0)
            if not next_windows or not next_samples or not next_count or not next_sum
                or not next_duration or not next_bytes or not next_messages then
              push(errors, id .. ' aggregate is outside exact integer range')
              bucket.broken = true
            else
              bucket.windows, bucket.samples, bucket.count = next_windows, next_samples, next_count
              bucket.sum, bucket.window_duration_ns = next_sum, next_duration
              bucket.bytes, bucket.messages = next_bytes, next_messages
              bucket.max = math.max(bucket.max, sample.max)
            end
          end
          add_identity(bucket, sample, errors, id)
        end
      end
    end
  end
  for _, component in ipairs({'server', 'client_runtime', 'presentation_ipc', 'unreal'}) do
    if not enabled[component] then push(missing, component .. '_enable') end
  end
  local segments = {}
  for _, id in ipairs(ORDER) do
    local bucket = buckets[id]
    if bucket and bucket.broken then
    elseif not bucket or bucket.samples < 1 then push(missing, id)
    else
      local segment = {
        component = M.definitions[id].component, metric = M.definitions[id].metric,
        scope = M.definitions[id].scope, unit = M.definitions[id].unit,
        windows = bucket.windows, samples = bucket.samples, count = bucket.count,
        sum = bucket.sum, max = bucket.max, mean = bucket.sum // bucket.samples,
        window_duration_ns = bucket.window_duration_ns,
        player_id = bucket.player_id, team_id = bucket.team_id,
      }
      if id == 'presentation_ipc_send' or id == 'presentation_ipc_receive' then
        segment.direction = M.definitions[id].direction
        segment.messages = bucket.messages
        segment.bytes = bucket.bytes
        if bucket.window_duration_ns == 0 then
          push(missing, id .. '_rate')
        else
          local rate = rate_of(bucket.bytes, bucket.window_duration_ns)
          if rate == nil then push(errors, id .. ' aggregate rate is outside exact integer range')
          else segment.rate_bytes_per_s = rate end
        end
      end
      if bucket.connections then
        local connections = {}
        for connection in pairs(bucket.connections) do connections[#connections + 1] = connection end
        table.sort(connections)
        segment.connections = connections
      end
      segments[id] = segment
    end
  end
  same_capture_identity(segments, identities, errors)
  local thresholds, threshold_result = json.null, 'not_evaluated'
  if baseline ~= nil then
    thresholds, threshold_result = {}, 'pass'
    local comparisons = type(baseline) == 'table' and baseline.comparisons or nil
    if not comparison_array(comparisons) then
      push(errors, 'baseline requires a non-empty comparisons array')
      threshold_result = 'fail'
    else
      for _, comparison in ipairs(comparisons) do
        local statistic = type(comparison) == 'table' and comparison.statistic or nil
        if statistic == 'p95' or statistic ~= 'max' and statistic ~= 'mean' then
          push(errors, 'baseline statistic must be mean or max, never p95')
          threshold_result = 'fail'
        else
          local matches = {}
          for _, candidate in ipairs(ORDER) do
            local def = M.definitions[candidate]
            if def.component == comparison.component and def.metric == comparison.metric
                and (comparison.direction == nil or def.direction == comparison.direction) then
              matches[#matches + 1] = candidate
            end
          end
          local limit = type(comparison) == 'table' and comparison.limit or nil
          local op = type(comparison) == 'table' and comparison.op or nil
          if #matches > 1 then
            push(errors, 'baseline comparison needs a direction to choose one wire_bytes window')
            threshold_result = 'fail'
            thresholds[#thresholds + 1] = {
              component = comparison.component, metric = comparison.metric, direction = comparison.direction,
              statistic = statistic, op = op, limit = limit, unit = comparison.unit,
              actual = json.null, pass = false,
            }
          else
            local exact_limit = integer(limit)
            local segment = matches[1] and segments[matches[1]] or nil
            local actual = segment and segment[statistic] or nil
            local ok = exact_limit ~= nil and segment and segment.unit == comparison.unit
                and (op == 'lte' or op == 'gte') and type(actual) == 'number'
                and ((op == 'lte' and actual <= exact_limit) or (op == 'gte' and actual >= exact_limit))
            thresholds[#thresholds + 1] = {
              component = comparison.component, metric = comparison.metric, direction = comparison.direction,
              statistic = statistic, op = op, limit = limit, unit = comparison.unit,
              actual = actual or json.null, pass = ok and true or false,
            }
            if exact_limit == nil then
              threshold_result = 'fail'
              push(errors, 'baseline limit must be a non-negative exact integer')
            elseif not ok then
              threshold_result = 'fail'
              push(errors, 'baseline comparison failed for ' .. tostring(comparison.component) .. ' ' .. tostring(comparison.metric))
            end
          end
        end
      end
    end
  end
  local status = 'success'
  if #errors > 0 then status = 'error' elseif #missing > 0 then status = 'unverified' end
  return {
    status = status, errors = errors, missing = missing,
    source_paths = {
      server = paths and paths.server or json.null,
      runtime = paths and paths.runtime or json.null,
      unreal = paths and paths.unreal or json.null,
    },
    metric_definitions = M.definitions, segments = segments,
    thresholds = thresholds, threshold_result = threshold_result,
  }
end

function M.options(argv, args)
  local parsed = args.parse(argv or {})
  if parsed.help == true and #parsed.positional == 0 and parsed.server == nil then return {help = true} end
  local allowed = {server = true, runtime = true, unreal = true, baseline = true, out = true, help = true, positional = true}
  for key in pairs(parsed) do
    if not allowed[key] then return nil, 'unknown option --' .. key end
  end
  if #parsed.positional > 0 then return nil, 'unexpected positional argument' end
  for _, name in ipairs({'server', 'runtime', 'unreal'}) do
    if type(parsed[name]) ~= 'string' or parsed[name] == '' then return nil, 'missing --' .. name end
  end
  if parsed.baseline ~= nil and type(parsed.baseline) ~= 'string' then return nil, 'missing --baseline path' end
  if parsed.out ~= nil and type(parsed.out) ~= 'string' then return nil, 'missing --out path' end
  return {
    server = parsed.server, runtime = parsed.runtime, unreal = parsed.unreal,
    baseline = parsed.baseline, out = parsed.out,
  }
end

local function failed(message, paths)
  return {
    status = 'error', errors = {message}, missing = {},
    source_paths = paths or {server = json.null, runtime = json.null, unreal = json.null},
    metric_definitions = M.definitions, segments = {},
    thresholds = json.null, threshold_result = 'not_evaluated',
  }
end

function M.main(argv, deps)
  deps = deps or {}
  local path = deps.path or require('tools.lua.lib.path')
  local args = deps.args or require('tools.lua.lib.args')
  local codec = deps.json or json
  local read = deps.read or path.read
  local write = deps.write or function(file, data) path.write(file, data, true) end
  local stdout = deps.stdout or io.write
  local options, err = M.options(argv, args)
  if not options then
    local report = failed(err)
    stdout(codec.encode(report) .. '\n')
    return 1, report
  end
  if options.help then
    stdout('Usage: tools/lua/lua.exe scripts/collect_moba_performance.lua --server SERVER_STDERR.log --runtime RUNTIME_STDERR.log --unreal UE.log [--baseline BASELINE.json] [--out SUMMARY.json]\n')
    return 0
  end
  local paths = {server = options.server, runtime = options.runtime, unreal = options.unreal}
  local logs = {}
  for _, role in ipairs({'server', 'runtime', 'unreal'}) do
    local ok, text = pcall(read, options[role])
    if not ok then
      local report = failed('cannot read ' .. role .. ' log: ' .. tostring(text), paths)
      stdout(codec.encode(report) .. '\n')
      return 1, report
    end
    logs[role] = text
  end
  local baseline = nil
  if options.baseline then
    local ok, text = pcall(read, options.baseline)
    if not ok then
      local report = failed('cannot read baseline: ' .. tostring(text), paths)
      stdout(codec.encode(report) .. '\n')
      return 1, report
    end
    local decoded, value = pcall(codec.decode, text)
    if not decoded then
      local report = failed('baseline is not JSON: ' .. tostring(value), paths)
      stdout(codec.encode(report) .. '\n')
      return 1, report
    end
    baseline = value
  end
  local report = M.summarize(logs, paths, baseline)
  local encoded = codec.encode(report) .. '\n'
  if options.out then
    local ok, write_err = pcall(write, options.out, encoded)
    if not ok then
      local failed_report = failed('cannot write summary: ' .. tostring(write_err), paths)
      stdout(codec.encode(failed_report) .. '\n')
      return 1, failed_report
    end
  end
  stdout(encoded)
  local code = report.status == 'success' and 0 or report.status == 'unverified' and 2 or 1
  return code, report
end

return M
