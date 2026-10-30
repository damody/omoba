-- ue_frame_windows: per-second frame windows lined up with the presentation events of that second.
package.path = "scripts/?.lua;" .. package.path
UE_FRAME_WINDOWS_AS_MODULE = true
local M = dofile("scripts/ue_frame_windows.lua")
local function perf(t, mean_ns, max_ns)
  return ('[2026.10.30-16.%s:000][  1]LogOmWorldBridge: Display: OM_PERF {"metric":"game_frame_interval","mean":%d,"max":%d}'):format(t, mean_ns, max_ns)
end
local log = table.concat({
  "[2026.10.30-16.50.00:000][  0]LogInit: start",
  perf("50.40", 16700000, 20000000),
  "[2026.10.30-16.50.40:500][  2]LogTemp: Display: OM_CUE ability=x cue=cast unit=1 art=style",
  "[2026.10.30-16.50.40:600][  2]LogTemp: Display: OM_STATUS_CUE buff=y cue=z unit=1 art=1",
  perf("50.41", 30000000, 400000000),
  '[2026.10.30-16.50.41:000][  3]LogOmWorldBridge: Display: OM_PERF_SPIKE {"frame_ms":400.0,"work_ms":350.0,"entities":12,"top":"ability_events:340.0"}',
  perf("50.50", 16000000, 18000000),
}, "\n")
local parsed = M.parse(log)
assert(#parsed.windows == 3 and #parsed.spikes == 1 and #parsed.events == 2, "windows, spikes and events parsed")
assert(parsed.windows[2].max_ms == 400 and math.abs(parsed.windows[2].t - 41) < 1e-6, "times are seconds since the first line")
local md, worst = M.summary(parsed, 1, 30)
assert(#worst == 1 and worst[1].max_ms == 400 and worst[1].cues == 2, "the worst window carries the events of its second")
assert(md:find("ability_events:340.0", 1, true), "the spike's top phases are listed")
assert(M.summary(parsed, 1, 1000):find("No OM_PERF", 1, true), "no window after the warm-up")
print("ue_frame_windows_test ok")
