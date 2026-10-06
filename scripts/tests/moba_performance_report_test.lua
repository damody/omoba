-- Collector contract: complete windows can succeed; missing, empty, NaN, and wrong scope cannot.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local report = require('moba_performance_report')
local args = b.lib('args')

local checks = 0
local function check(condition, message)
  checks = checks + 1
  if not condition then error(message or 'check failed', 2) end
end

local function contains(list, wanted)
  for _, item in ipairs(list or {}) do
    if item == wanted or (type(item) == 'string' and item:find(wanted, 1, true)) then return true end
  end
  return false
end

local MEAN = 'integer_floor_sum_over_samples'
local function line(marker, record)
  return marker .. json.encode(record)
end

local function time_sample(component, metric, scope, samples, sum, max, duration, extra)
  local record = {
    v = 1, component = component, metric = metric, scope = scope, unit = 'ns',
    samples = samples, count = samples, sum = sum, max = max, mean = sum // samples,
    mean_definition = MEAN, not_a_statistic = 'p95', window_duration_ns = duration,
  }
  for key, value in pairs(extra or {}) do record[key] = value end
  return record
end

local function ipc_sample(direction, messages, bytes, max, duration, rate, connection)
  local record = {
    v = 1, component = 'presentation_ipc', metric = 'wire_bytes',
    scope = 'localhost_tcp_length_prefixed_frame', unit = 'byte', direction = direction,
    samples = messages, count = messages, messages = messages, sum = bytes, bytes = bytes,
    max = max, mean = bytes // messages, mean_definition = MEAN, not_a_statistic = 'p95',
    window_duration_ns = duration, rate_definition = 'integer_floor_bytes_per_window_second',
    rate_is_not = 'frame_interval', player_id = 7, team_id = 2, connection = connection,
  }
  record.rate_bytes_per_s = duration == 0 and json.null or rate
  return record
end

local function unreal_enable(player, team)
  return line('OM_PERF_ENABLE ', {
    v = 1, component = 'unreal', metrics = {'game_frame_interval', 'presentation_work'},
    scopes = {'game_thread_delta_seconds', 'actor_tick_fplatformtime'}, level = 'Display',
    window_samples = 60, frame_interval_is = 'game_thread_delta_seconds',
    frame_interval_is_not = 'gpu_or_render_thread', player_id = player, team_id = team,
  })
end

local function enables()
  return {
    server = line('OM_PERF_ENABLE ', {
      v = 1, component = 'server', metric = 'tick_compute',
      scope = 'state_tick_excluding_transport_send', level = 'info', window_samples = 60,
      excludes = 'scheduler_sleep,reliable_send_timeout,warmup,paused,finished', hash_input = false,
    }),
    client = line('OM_PERF_ENABLE ', {
      v = 1, component = 'client_runtime', metric = 'replica_step', scope = 'apply_encoded_frame',
      level = 'info', window_samples = 60, excludes = 'evidence_io,presentation,kcp',
      player_id = 7, team_id = 2,
    }),
    ipc = line('OM_PERF_ENABLE ', {
      v = 1, component = 'presentation_ipc', metric = 'wire_bytes',
      scope = 'localhost_tcp_length_prefixed_frame', level = 'info', window_samples = 60,
      transport = 'localhost_tcp', not_transport = 'kcp', player_id = 7, team_id = 2, connection = 11,
    }),
    unreal = unreal_enable(7, 2),
  }
end

local function samples(send_bytes, send_duration, send_rate)
  send_bytes, send_duration, send_rate = send_bytes or 2400, send_duration or 2000000000, send_rate or 1200
  return {
    server = line('OM_PERF ', time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 100, 10, 1000000000)),
    client = line('OM_PERF ', time_sample('client_runtime', 'replica_step', 'apply_encoded_frame', 60, 120000, 5000, 1000000000, {player_id = 7, team_id = 2})),
    send = line('OM_PERF ', ipc_sample('send', send_bytes == 2400 and 60 or 2, send_bytes, math.min(send_bytes, 15000000), send_duration, send_rate, 11)),
    receive = line('OM_PERF ', ipc_sample('receive', 60, 1800, 50, 2000000000, 900, 11)),
    interval = line('OM_PERF ', time_sample('unreal', 'game_frame_interval', 'game_thread_delta_seconds', 60, 1000000020, 16666667, 1000000000, {
      player_id = 7, team_id = 2, interval_is = 'game_thread_delta_seconds', interval_is_not = 'gpu_or_render_thread',
    })),
    work = line('OM_PERF ', time_sample('unreal', 'presentation_work', 'actor_tick_fplatformtime', 60, 600000, 20000, 1000000000, {
      player_id = 7, team_id = 2, work_is = 'actor_tick_fplatformtime', work_is_not = 'gpu_or_render_thread',
    })),
  }
end

local function logs_from(parts, prefix)
  local on = enables()
  local body = samples()
  for key, value in pairs(parts or {}) do body[key] = value end
  local server = table.concat({on.server, prefix and (prefix .. body.server) or body.server}, '\n')
  local runtime = table.concat({on.client, on.ipc, body.client, body.send, body.receive}, '\r\n')
  local unreal = table.concat({on.unreal, body.interval, body.work}, '\n')
  return {server = server, runtime = runtime, unreal = unreal}
end

local function ue_logs(player, team)
  local logs = logs_from()
  logs.unreal = table.concat({
    unreal_enable(player, team),
    line('OM_PERF ', time_sample('unreal', 'game_frame_interval', 'game_thread_delta_seconds', 60, 1000000020, 16666667, 1000000000, {
      player_id = player, team_id = team, interval_is = 'game_thread_delta_seconds', interval_is_not = 'gpu_or_render_thread',
    })),
    line('OM_PERF ', time_sample('unreal', 'presentation_work', 'actor_tick_fplatformtime', 60, 600000, 20000, 1000000000, {
      player_id = player, team_id = team, work_is = 'actor_tick_fplatformtime', work_is_not = 'gpu_or_render_thread',
    })),
  }, '\n')
  return logs
end

local paths = {server = 'server.log', runtime = 'runtime.log', unreal = 'ue.log'}
local function summarize(logs, baseline)
  return report.summarize(logs, paths, baseline)
end

local ok = summarize(logs_from(nil, 'INFO omobab: '))
check(ok.status == 'success', 'complete logs must succeed, got ' .. ok.status .. ' ' .. table.concat(ok.errors, '; '))
check(#ok.errors == 0 and #ok.missing == 0, 'complete logs have no errors or missing segments')
check(ok.threshold_result == 'not_evaluated' and ok.thresholds == json.null, 'no baseline means thresholds are not evaluated')
check(ok.source_paths.server == 'server.log' and ok.source_paths.runtime == 'runtime.log' and ok.source_paths.unreal == 'ue.log', 'source paths')
check(ok.metric_definitions.server_tick_compute.scope == 'state_tick_excluding_transport_send', 'server definition')
check(ok.metric_definitions.presentation_ipc_send.rate_is_not:find('frame', 1, true) ~= nil, 'byte rate is not a frame interval')
check(ok.metric_definitions.unreal_game_frame_interval['not']:find('GPU', 1, true) ~= nil, 'frame interval definition denies GPU time')
local server = ok.segments.server_tick_compute
check(server.samples == 60 and server.count == 60 and server.sum == 100 and server.mean == 1 and server.max == 10, 'server mean is integer floor, not a rounded mean')
check(server.p95 == nil and server.rate_bytes_per_s == nil and server.unit == 'ns', 'server summary has no p95 and no byte rate')
check(ok.segments.client_replica_step.player_id == 7 and ok.segments.client_replica_step.team_id == 2, 'client identity')
check(ok.segments.client_replica_step.mean == 2000 and ok.segments.client_replica_step.rate_bytes_per_s == nil, 'client step is time, not a byte rate')
check(ok.segments.presentation_ipc_send.player_id == 7 and ok.segments.presentation_ipc_send.team_id == 2, 'send identity')
check(ok.segments.presentation_ipc_send.rate_bytes_per_s == 1200 and ok.segments.presentation_ipc_send.unit == 'byte', 'send rate')
check(ok.segments.presentation_ipc_receive.player_id == 7 and ok.segments.presentation_ipc_receive.team_id == 2, 'receive identity')
check(ok.segments.presentation_ipc_receive.rate_bytes_per_s == 900 and ok.segments.presentation_ipc_receive.direction == 'receive', 'receive rate')
check(ok.segments.unreal_game_frame_interval.player_id == 7 and ok.segments.unreal_game_frame_interval.team_id == 2, 'interval identity')
check(ok.segments.unreal_game_frame_interval.rate_bytes_per_s == nil and ok.segments.unreal_game_frame_interval.scope == 'game_thread_delta_seconds', 'frame interval is not a byte rate')
check(ok.segments.unreal_presentation_work.player_id == 7 and ok.segments.unreal_presentation_work.team_id == 2, 'work identity')
check(ok.segments.unreal_presentation_work.scope == 'actor_tick_fplatformtime' and ok.segments.unreal_presentation_work.mean == 10000, 'presentation work')
check(json.encode(ok):find('"p95":', 1, true) == nil, 'encoded summary has no p95 field')

local second = line('OM_PERF ', time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 80, 8, 500000000))
local aggregate = summarize(logs_from({server = enables().server .. '\n' .. samples().server .. '\n' .. second}))
check(aggregate.status == 'success', 'two server windows must aggregate')
local totaled = aggregate.segments.server_tick_compute
check(totaled.windows == 2 and totaled.samples == 120 and totaled.sum == 180 and totaled.max == 10 and totaled.mean == 1, 'aggregate mean uses the total sum')
check(totaled.window_duration_ns == 1500000000, 'aggregate duration')

local other_connection = line('OM_PERF ', ipc_sample('send', 60, 1200, 40, 1000000000, 1200, 4))
local other_enable = line('OM_PERF_ENABLE ', {
  v = 1, component = 'presentation_ipc', metric = 'wire_bytes',
  scope = 'localhost_tcp_length_prefixed_frame', level = 'info', window_samples = 60,
  transport = 'localhost_tcp', not_transport = 'kcp', player_id = 7, team_id = 2, connection = 4,
})
local connection_logs = logs_from({send = samples().send .. '\n' .. other_connection})
connection_logs.runtime = other_enable .. '\n' .. connection_logs.runtime
local connections = summarize(connection_logs)
check(connections.status == 'success', 'two connections stay one send segment')
local listed = connections.segments.presentation_ipc_send.connections
check(listed[1] == 4 and listed[2] == 11 and #listed == 2, 'connection ids are collected and sorted')
check(connections.segments.presentation_ipc_send.bytes == 3600 and connections.segments.presentation_ipc_send.rate_bytes_per_s == 1200, 'aggregate byte rate uses total bytes and duration')

local large = samples(20000000, 1000000000, 20000000)
local wide = summarize(logs_from({send = line('OM_PERF ', ipc_sample('send', 2, 20000000, 15000000, 1000000000, 20000000, 11))}))
check(wide.status == 'success', 'large byte window must keep an exact rate, got ' .. table.concat(wide.errors, '; '))
check(wide.segments.presentation_ipc_send.rate_bytes_per_s == 20000000 and wide.segments.presentation_ipc_send.mean == 10000000, '20MB over one second is 20000000 B/s')
check(large.send:find('20000000', 1, true) ~= nil, 'fixture contains the wide window')

local function without(key)
  local parts = {}
  parts[key] = ''
  return summarize(logs_from(parts))
end
local missing_work = without('work')
check(missing_work.status == 'unverified' and missing_work.status ~= 'success', 'missing segment is unverified')
check(contains(missing_work.missing, 'unreal_presentation_work'), 'missing work is named')
local missing_enable = summarize({
  server = samples().server, runtime = logs_from().runtime, unreal = logs_from().unreal,
})
check(missing_enable.status == 'unverified' and contains(missing_enable.missing, 'server_enable'), 'missing enable is unverified')

local function replace_server(record)
  return summarize(logs_from({server = enables().server .. '\n' .. line('OM_PERF ', record)}))
end
local empty_window = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 100, 10, 1000000000)
empty_window.samples, empty_window.count, empty_window.sum, empty_window.max, empty_window.mean = 0, 0, 0, 0, 0
local empty = replace_server(empty_window)
check(empty.status == 'error' and empty.status ~= 'success', 'empty window is an error')
check(contains(empty.errors, 'empty'), 'empty window explains itself')

local nan = (enables().server .. '\n' .. samples().server):gsub('"mean":1', '"mean":NaN', 1)
local nan_report = summarize(logs_from({server = nan}))
check(nan_report.status == 'error' and contains(nan_report.errors, 'invalid JSON'), 'NaN is not a successful measurement')
local inf = (enables().server .. '\n' .. samples().server):gsub('"mean":1', '"mean":1e9999', 1)
check(summarize(logs_from({server = inf})).status == 'error', 'infinity is an error')

local wrong_scope = time_sample('server', 'tick_compute', 'whole_process_wall', 60, 100, 10, 1000000000)
check(replace_server(wrong_scope).status == 'error', 'wrong scope is an error')
local wrong_unit = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 100, 10, 1000000000)
wrong_unit.unit = 'ms'
check(replace_server(wrong_unit).status == 'error', 'wrong unit is an error')
local rounded = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 100, 10, 1000000000)
rounded.mean = 2
check(replace_server(rounded).status == 'error', 'rounded mean is not the integer floor')
local claimed_p95 = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 60, 100, 10, 1000000000)
claimed_p95.p95 = 9
check(replace_server(claimed_p95).status == 'error', 'a p95 field is an error')
claimed_p95.p95 = nil
claimed_p95.not_a_statistic = 'mean'
check(replace_server(claimed_p95).status == 'error', 'mean must not be renamed into a percentile')

local rated_interval = time_sample('unreal', 'game_frame_interval', 'game_thread_delta_seconds', 60, 1000000020, 16666667, 1000000000, {
  player_id = 7, team_id = 2, interval_is = 'game_thread_delta_seconds', interval_is_not = 'gpu_or_render_thread', rate_bytes_per_s = 1,
})
check(summarize(logs_from({interval = line('OM_PERF ', rated_interval)})).status == 'error', 'frame interval must not carry a byte rate')
local interval_on_ipc = ipc_sample('send', 60, 2400, 80, 2000000000, 1200, 11)
interval_on_ipc.interval_is = 'game_thread_delta_seconds'
check(summarize(logs_from({send = line('OM_PERF ', interval_on_ipc)})).status == 'error', 'IPC bytes must not carry a frame interval')

local misfiled = logs_from({server = enables().server})
misfiled.unreal = misfiled.unreal .. '\n' .. samples().server
local misfiled_report = summarize(misfiled)
check(misfiled_report.status == 'error' and contains(misfiled_report.errors, 'found in unreal log'), 'a server line in the Unreal log is not a server sample')

local mixed = logs_from()
mixed.runtime = mixed.runtime .. '\n' .. line('OM_PERF ', time_sample('client_runtime', 'replica_step', 'apply_encoded_frame', 60, 60, 1, 1, {player_id = 8, team_id = 2}))
check(summarize(mixed).status == 'error' and contains(summarize(mixed).errors, 'mixes player_id'), 'mixed player ids are an error')

local overflow_a = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 1, 9007199254740991, 9007199254740991, 1)
local overflow_b = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 1, 2, 2, 1)
local overflow = summarize(logs_from({server = enables().server .. '\n' .. line('OM_PERF ', overflow_a) .. '\n' .. line('OM_PERF ', overflow_b)}))
check(overflow.status == 'error' and contains(overflow.errors, 'exact integer range'), 'aggregates must stay inside exact integers')

local zero = summarize(logs_from({send = line('OM_PERF ', ipc_sample('send', 60, 2400, 80, 0, nil, 11))}))
check(zero.status == 'unverified' and contains(zero.missing, 'presentation_ipc_send_rate'), 'zero-duration byte rate is unverified, not a number')
check(zero.segments.presentation_ipc_send.rate_bytes_per_s == nil, 'zero-duration segment does not invent a rate')

local function baseline(comparison)
  return summarize(logs_from(), {comparisons = {comparison}})
end
local pass_mean = baseline({component = 'server', metric = 'tick_compute', statistic = 'mean', op = 'lte', limit = server.mean, unit = 'ns'})
check(pass_mean.status == 'success' and pass_mean.threshold_result == 'pass' and pass_mean.thresholds[1].pass == true, 'external mean limit can pass')
local pass_max = baseline({component = 'server', metric = 'tick_compute', statistic = 'max', op = 'gte', limit = server.max, unit = 'ns'})
check(pass_max.status == 'success' and pass_max.thresholds[1].actual == server.max, 'external max limit can pass')
local fail_mean = baseline({component = 'client_runtime', metric = 'replica_step', statistic = 'mean', op = 'lte', limit = 1, unit = 'ns'})
check(fail_mean.status == 'error' and fail_mean.threshold_result == 'fail', 'failed external limit is an error')
local fail_p95 = baseline({component = 'server', metric = 'tick_compute', statistic = 'p95', op = 'lte', limit = 1, unit = 'ns'})
check(fail_p95.status == 'error' and contains(fail_p95.errors, 'p95'), 'baseline p95 is rejected')
local ambiguous = baseline({component = 'presentation_ipc', metric = 'wire_bytes', statistic = 'mean', op = 'lte', limit = 999999, unit = 'byte'})
check(ambiguous.status == 'error' and contains(ambiguous.errors, 'direction'), 'wire bytes need a direction')
local send_pass = baseline({component = 'presentation_ipc', metric = 'wire_bytes', direction = 'send', statistic = 'mean', op = 'lte', limit = ok.segments.presentation_ipc_send.mean, unit = 'byte'})
check(send_pass.status == 'success' and send_pass.thresholds[1].actual == 40, 'directed send comparison uses the send window')

local captured, files = {}, {}
local function deps()
  return {
    args = args, json = json,
    read = function(path)
      if files[path] == nil then error('missing file ' .. path) end
      return files[path]
    end,
    write = function(path, data) files[path] = data end,
    stdout = function(data) captured[#captured + 1] = data end,
  }
end
local function run(argv)
  captured = {}
  return report.main(argv, deps())
end
local help_code = run({'--help'})
check(help_code == 0 and captured[1]:find('--server', 1, true) ~= nil, 'help exits 0')
local unknown_code, unknown = run({'--server', 'a', '--nope', 'x'})
check(unknown_code == 1 and unknown.status == 'error' and contains(unknown.errors, 'unknown option'), 'unknown option is a report, not a crash')
local positional_code, positional = run({'--server', 'a', '--runtime', 'b', '--unreal', 'c', 'extra'})
check(positional_code == 1 and contains(positional.errors, 'positional'), 'positional argument is an error report')
files = {}
local missing_code, missing_file = run({'--server', 'missing.log', '--runtime', 'runtime.log', '--unreal', 'ue.log'})
check(missing_code == 1 and contains(missing_file.errors, 'cannot read'), 'missing log is an error report')
files = {['server.log'] = '{}', ['runtime.log'] = '{}', ['ue.log'] = '{}', ['baseline.json'] = '{'}
local bad_base_code, bad_base = run({'--server', 'server.log', '--runtime', 'runtime.log', '--unreal', 'ue.log', '--baseline', 'baseline.json'})
check(bad_base_code == 1 and contains(bad_base.errors, 'not JSON'), 'bad baseline JSON is an error report')

local complete = logs_from(nil, 'INFO omobab: ')
files = {['server.log'] = complete.server, ['runtime.log'] = complete.runtime, ['unreal.log'] = complete.unreal}
local success_code, success = run({'--server', 'server.log', '--runtime', 'runtime.log', '--unreal', 'unreal.log', '--out', 'summary.json'})
check(success_code == 0 and success.status == 'success', 'CLI success exits 0')
check(json.decode(files['summary.json']).status == 'success', 'CLI writes the summary JSON')
check(json.decode(captured[1]).source_paths.unreal == 'unreal.log', 'CLI stdout keeps the source path')
files = {['server.log'] = '', ['runtime.log'] = '', ['unreal.log'] = ''}
local unverified_code, unverified = run({'--server', 'server.log', '--runtime', 'runtime.log', '--unreal', 'unreal.log'})
check(unverified_code == 2 and unverified.status == 'unverified', 'CLI unverified exits 2')
files = {['server.log'] = complete.server, ['runtime.log'] = complete.runtime, ['unreal.log'] = complete.unreal}
local function failing_deps()
  local base = deps()
  base.write = function() error('disk full') end
  return base
end
captured = {}
local write_code, write_report = report.main({'--server', 'server.log', '--runtime', 'runtime.log', '--unreal', 'unreal.log', '--out', 'summary.json'}, failing_deps())
check(write_code == 1 and contains(write_report.errors, 'cannot write'), 'summary write failure is an error')

local wrong_player = summarize(ue_logs(4, 2))
check(wrong_player.status == 'error' and contains(wrong_player.errors, 'not the same player and team'), 'wrong player is rejected')
local wrong_team = summarize(ue_logs(7, 1))
check(wrong_team.status == 'error' and contains(wrong_team.errors, 'not the same player and team'), 'wrong team is rejected')
local enable_mismatch = logs_from()
enable_mismatch.runtime = enable_mismatch.runtime:gsub('"player_id":7', '"player_id":9', 1)
local enable_report = summarize(enable_mismatch)
check(enable_report.status == 'error' and contains(enable_report.errors, 'enable identity'), 'enable identity must match its summary')
local orphan = summarize(logs_from({receive = line('OM_PERF ', ipc_sample('receive', 60, 1800, 50, 2000000000, 900, 99))}))
check(orphan.status == 'error' and contains(orphan.errors, 'matching enable'), 'IPC sample connection needs its own enable')
local over_max = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 2, 100, 10, 1000000000)
local over_max_report = replace_server(over_max)
check(over_max_report.status == 'error' and contains(over_max_report.errors, 'samples*max'), 'sum above samples*max is rejected')
local zero_sum = time_sample('server', 'tick_compute', 'state_tick_excluding_transport_send', 1, 0, 0, 1)
check(replace_server(zero_sum).status == 'success', 'zero sum with max 0 stays inside the window')
local exact_max = 9007199254740991
local exact_rate = summarize(logs_from({send = line('OM_PERF ', ipc_sample('send', 1, exact_max, exact_max, exact_max, 1000000000, 11))}))
check(exact_rate.status == 'success' and exact_rate.segments.presentation_ipc_send.rate_bytes_per_s == 1000000000, 'rate at the exact integer boundary stays exact')
local overflow_rate = summarize(logs_from({send = line('OM_PERF ', ipc_sample('send', 1, exact_max, exact_max, 1, 0, 11))}))
check(overflow_rate.status == 'error' and contains(overflow_rate.errors, 'exact integer range'), 'bytes*1e9 above the exact range is rejected')
local object_baseline = summarize(logs_from(), {comparisons = {component = 'server'}})
check(object_baseline.status == 'error' and object_baseline.threshold_result == 'fail', 'object baseline cannot pass')
local text_baseline = summarize(logs_from(), {comparisons = 'server'})
check(text_baseline.status == 'error' and text_baseline.threshold_result == 'fail', 'string baseline cannot pass')
local null_baseline = summarize(logs_from(), {comparisons = json.null})
check(null_baseline.status == 'error' and null_baseline.threshold_result == 'fail', 'null baseline cannot pass')
local negative_limit = baseline({component = 'server', metric = 'tick_compute', statistic = 'mean', op = 'lte', limit = -1, unit = 'ns'})
check(negative_limit.status == 'error' and negative_limit.threshold_result == 'fail' and contains(negative_limit.errors, 'non-negative'), 'negative limit is rejected')
check(not contains(negative_limit.errors, 'baseline comparison failed'), 'negative limit is not a generic comparison failure')
local fractional_limit = baseline({component = 'server', metric = 'tick_compute', statistic = 'mean', op = 'lte', limit = 1.5, unit = 'ns'})
check(fractional_limit.status == 'error' and fractional_limit.threshold_result == 'fail' and contains(fractional_limit.errors, 'non-negative'), 'fractional limit is rejected')
local zero_limit = baseline({component = 'server', metric = 'tick_compute', statistic = 'mean', op = 'gte', limit = 0, unit = 'ns'})
check(zero_limit.status == 'success' and zero_limit.threshold_result == 'pass', 'zero limit is a non-negative exact integer')

-- Codex review regressions: these cannot count as a complete formal capture.
local mismatched_capture = summarize(ue_logs(4, 1)) -- explicit player 4 / team 1 against runtime player 7 / team 2
check(mismatched_capture.status == 'error', 'cross-component player/team mismatch must fail')
local very_long_window = summarize(logs_from({send = line('OM_PERF ',
  ipc_sample('send', 2, 20000000, 15000000, 9007199254740991, 2, 11))}))
check(very_long_window.status == 'success', 'valid long-window byte rate must not be rejected by intermediate arithmetic')
check(very_long_window.segments.presentation_ipc_send.rate_bytes_per_s == 2, 'long-window byte rate is exactly 2')
local empty_baseline = summarize(logs_from(), {comparisons = {}})
check(empty_baseline.threshold_result == 'fail', 'empty baseline cannot pass')
check(empty_baseline.status == 'error', 'empty baseline is an error')
print('moba performance report: ' .. checks .. ' checks passed')
