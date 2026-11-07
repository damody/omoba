local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local report = require('moba_frame_time_report')

-- E144: percentiles of the measured span come from subtracting cumulative histograms.
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local function hist(stamp, metric, samples, max, counts)
  return ('[%s][ 1]LogOmWorldBridge: Display: OM_PERF_HIST {"v":1,"component":"unreal","metric":"%s","scope":"x","unit":"ns",'
    .. '"cumulative":true,"bucket_ns":250000,"buckets":1000,"samples":%d,"max":%d,"p50":0,"p95":0,"p99":0,'
    .. '"percentile_is":"bucket_upper_edge","counts":"%s","player_id":1,"team_id":1}'):format(stamp, metric, samples, max, counts)
end
local log = table.concat({
  'LogInit: OS: Windows 11 (26H2) [10.0.26300.9711] (), CPU: 12th Gen Intel(R) Core(TM) i9-12900KS, GPU: NVIDIA GeForce RTX 4070',
  'LogInit: Command Line: /Game/Map/Main -game -d3d11 -windowed -ResX=1920 -ResY=1080 -RenderOffScreen -ExecCmds="r.SetRes 1920x1080w"',
  -- Warm-up: 600 frames, half of them 100 ms shader hitches.
  hist('2026.10.27-05.00.10:000', 'game_frame_interval', 600, 400000000, '66:300,400:299,1000:1'),
  hist('2026.10.27-05.00.10:000', 'presentation_work', 600, 2000000, '4:600'),
  hist('2026.10.27-05.00.30:000', 'game_frame_interval', 1200, 400000000, '66:900,400:299,1000:1'),
  -- Measured span: +1000 frames, 950 at 16.6 ms, 40 at 33.3 ms, 10 at 50 ms.
  hist('2026.10.27-05.01.00:000', 'game_frame_interval', 2200, 400000000, '66:1850,133:40,200:10,400:299,1000:1'),
  hist('2026.10.27-05.01.00:000', 'presentation_work', 1600, 3000000, '4:1500,11:100'),
}, '\n')

local r = report.report(log, {warmup_seconds = 25})
local frame = r.metrics.game_frame_interval
check(frame.warmup_excluded and frame.samples == 1000, 'warm-up lines are subtracted: ' .. tostring(frame.samples))
check(math.abs(frame.p50_ms - 16.75) < 1e-9, 'p50 ' .. tostring(frame.p50_ms))
check(math.abs(frame.p95_ms - 16.75) < 1e-9, 'p95 is still the 16.6 ms bucket (950 of 1000)')
check(math.abs(frame.p99_ms - 33.5) < 1e-9, 'p99 reaches the 33.3 ms bucket ' .. tostring(frame.p99_ms))
check(frame.over_250ms == 0, 'the warm-up hitch is not in the measured span')
check(math.abs(frame.span_seconds - 30) < 1e-6 and math.abs(frame.mean_fps - 1000 / 30) < 1e-6, 'span and mean fps')
local work = r.metrics.presentation_work
check(work.samples == 1000 and math.abs(work.p95_ms - 3.0) < 1e-9, 'second metric uses its own lines')
local c = r.conditions
check(c.cpu:find('12900KS', 1, true) and c.gpu == 'NVIDIA GeForce RTX 4070', 'hardware read from the log')
check(c.off_screen and c.resolution == '1920x1080' and c.rhi == 'd3d11' and c.forced_resolution == '1920x1080', 'conditions')
local text = report.format(r)
check(text:find('OFF-SCREEN', 1, true) and text:find('p95 16.75 ms', 1, true), 'formatted report says off-screen')

-- Without warm-up exclusion (span shorter than warm-up) everything counts, and it says so.
local short = report.report(log, {warmup_seconds = 600}).metrics.game_frame_interval
check(not short.warmup_excluded and short.samples == 2200 and short.over_250ms == 1, 'short run keeps all samples')
check(short.p99_ms == 100.25, 'hitches dominate p99 when warm-up is not excluded: ' .. tostring(short.p99_ms))
-- A log from a renderer without histograms.
local none = report.report('LogInit: Command Line: -game', {})
check(next(none.metrics) == nil and report.format(none):find('no OM_PERF_HIST', 1, true), 'old renderer is reported, not guessed')
-- Overflowing percentile is reported as > 250 ms, never as a number.
local spiky = report.report(hist('2026.10.27-05.00.10:000', 'game_frame_interval', 10, 900000000, '1000:10'), {}).metrics.game_frame_interval
check(spiky.p95_ms == nil and spiky.over_250ms == 10, 'overflow has no fabricated value')
print(('frame time report: %d checks passed'):format(checks))
