-- Where a UE renderer drops frames, and what it was drawing then (work line C, M6 UE performance).
-- Reads one renderer log and lines up, per one-second OM_PERF window of the game-thread frame
-- interval (mean, max), what the presentation did in that second: cue plays (OM_CUE), status
-- cues (OM_STATUS_CUE), target cues (OM_CAST_TARGET_CUES), status instances (OM_BUFF) and the
-- frame spikes (OM_PERF_SPIKE: frame and work ms, entities, the top phases).
--
--   tools\lua\lua.exe scripts\ue_frame_windows.lua --log UE_LOG [--top N] [--warmup-seconds S] [--out FILE]
--
-- Prints the worst windows by max frame interval, the busiest cue seconds, and whether the worst
-- windows are the busy ones (a cue cost) or not (something else).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}

local function seconds_of(line)
  local y, mo, d, h, mi, s, ms = line:match("^%[(%d+)%.(%d+)%.(%d+)%-(%d+)%.(%d+)%.(%d+):(%d+)%]")
  if not y then return nil end
  return os.time({ year = tonumber(y), month = tonumber(mo), day = tonumber(d), hour = tonumber(h), min = tonumber(mi), sec = tonumber(s) }) + tonumber(ms) / 1000
end

-- log text -> { windows = {{t, mean_ms, max_ms}}, spikes = {{t, frame_ms, work_ms, entities, top}},
--   events = {{t, kind}}, markers = {{t, kind}} } with t in seconds since the first timestamped line.
-- Markers are what else stalls a frame (D21): a screenshot the autopilot takes (harness cost, not
-- the game's), an audio render thread stall and the stack dump it triggers on the game thread.
M.MARKERS = {
  { kind = "screenshot", needle = "Tracing Screenshot" },
  { kind = "screenshot", needle = "OM_AUTOPILOT_SHOT captured" },
  { kind = "audio_stall", needle = "Stall in AudioRenderThread" },
  { kind = "audio_stall", needle = "ThreadStackWalkAndDump" },
}
function M.parse(text)
  local out = { windows = {}, spikes = {}, events = {}, markers = {} }
  local origin
  for line in text:gmatch("[^\r\n]+") do
    local t = seconds_of(line)
    if t then
      origin = origin or t
      t = t - origin
      local perf = line:match("OM_PERF (%b{})")
      if perf then
        local ok, value = pcall(json.decode, perf)
        if ok and value.metric == "game_frame_interval" and value.mean and value.max then
          out.windows[#out.windows + 1] = { t = t, mean_ms = value.mean / 1e6, max_ms = value.max / 1e6 }
        end
      end
      local spike = line:match("OM_PERF_SPIKE (%b{})")
      if spike then
        local ok, value = pcall(json.decode, spike)
        if ok then
          out.spikes[#out.spikes + 1] = { t = t, frame_ms = value.frame_ms, work_ms = value.work_ms, entities = value.entities, top = value.top }
        end
      end
      for _, kind in ipairs({ "OM_CUE ", "OM_STATUS_CUE ", "OM_CAST_TARGET_CUES ", "OM_BUFF " }) do
        if line:find(kind, 1, true) then out.events[#out.events + 1] = { t = t, kind = kind:sub(1, -2) } end
      end
      for _, marker in ipairs(M.MARKERS) do
        if line:find(marker.needle, 1, true) then out.markers[#out.markers + 1] = { t = t, kind = marker.kind } end
      end
    end
  end
  return out
end

-- Counts of each event kind in (t - span, t].
local function counts_in(events, t, span)
  local counts = {}
  for _, e in ipairs(events) do
    if e.t > t - span and e.t <= t then counts[e.kind] = (counts[e.kind] or 0) + 1 end
  end
  return counts
end

local function cue_total(counts)
  return (counts.OM_CUE or 0) + (counts.OM_STATUS_CUE or 0) + (counts.OM_CAST_TARGET_CUES or 0) + (counts.OM_BUFF or 0)
end

-- Every frame of at least HITCH_MS after the warm-up, with its cause: a marker within the frame
-- (screenshot, audio_stall), the presentation work when it took at least half the frame, else
-- other (outside the presentation: engine, GC, the machine).
M.HITCH_MS = 100
function M.hitches(parsed, warmup)
  local list, counts = {}, {}
  for _, s in ipairs(parsed.spikes) do
    if s.t >= warmup and (s.frame_ms or 0) >= M.HITCH_MS then
      local cause
      for _, m in ipairs(parsed.markers or {}) do
        if m.t <= s.t and m.t >= s.t - math.max(s.frame_ms / 1000, 0.1) - 0.05 then cause = m.kind end
      end
      if not cause then cause = (s.work_ms or 0) >= s.frame_ms * 0.5 and "presentation" or "other" end
      list[#list + 1] = { t = s.t, frame_ms = s.frame_ms, work_ms = s.work_ms or 0, cause = cause, top = s.top }
      counts[cause] = (counts[cause] or 0) + 1
    end
  end
  return list, counts
end

-- parsed -> markdown summary, worst windows
function M.summary(parsed, top, warmup)
  top, warmup = top or 10, warmup or 30
  local windows = {}
  for _, w in ipairs(parsed.windows) do if w.t >= warmup then windows[#windows + 1] = w end end
  local lines = { "# UE frame time by second", "" }
  if #windows == 0 then
    lines[#lines + 1] = "No OM_PERF game_frame_interval windows after the warm-up."
    return table.concat(lines, "\n") .. "\n", {}
  end
  local means, maxes = {}, {}
  for _, w in ipairs(windows) do means[#means + 1] = w.mean_ms; maxes[#maxes + 1] = w.max_ms end
  table.sort(means); table.sort(maxes)
  local function pct(list, p) return list[math.max(1, math.ceil(#list * p))] end
  lines[#lines + 1] = ("%d windows after %d s warm-up. Mean frame interval: median %.1f ms, p95 %.1f ms. Window max: median %.1f ms, p95 %.1f ms, worst %.1f ms.")
    :format(#windows, warmup, pct(means, 0.5), pct(means, 0.95), pct(maxes, 0.5), pct(maxes, 0.95), maxes[#maxes])
  lines[#lines + 1] = ""
  local sorted = {}
  for _, w in ipairs(windows) do sorted[#sorted + 1] = w end
  table.sort(sorted, function(a, c) return a.max_ms > c.max_ms end)
  lines[#lines + 1] = "| t (s) | mean ms | max ms | cue | status cue | target cue | buff | spike top phases |"
  lines[#lines + 1] = "|---|---|---|---|---|---|---|---|"
  local worst = {}
  for i = 1, math.min(top, #sorted) do
    local w = sorted[i]
    local c = counts_in(parsed.events, w.t, 1.2)
    local phases = ""
    for _, s in ipairs(parsed.spikes) do
      if s.t > w.t - 1.2 and s.t <= w.t and (s.frame_ms or 0) >= w.max_ms * 0.8 then phases = (s.top or "") .. (" (%.1f ms, %d entities)"):format(s.frame_ms or 0, s.entities or 0) end
    end
    lines[#lines + 1] = ("| %.0f | %.1f | %.1f | %d | %d | %d | %d | %s |"):format(w.t, w.mean_ms, w.max_ms,
      c.OM_CUE or 0, c.OM_STATUS_CUE or 0, c.OM_CAST_TARGET_CUES or 0, c.OM_BUFF or 0, phases)
    worst[#worst + 1] = { t = w.t, max_ms = w.max_ms, cues = cue_total(c) }
  end
  -- The busiest presentation seconds and their frame times.
  local busy = {}
  for _, w in ipairs(windows) do busy[#busy + 1] = { t = w.t, mean_ms = w.mean_ms, max_ms = w.max_ms, cues = cue_total(counts_in(parsed.events, w.t, 1.2)) } end
  table.sort(busy, function(a, c) return a.cues > c.cues end)
  lines[#lines + 1] = ""
  lines[#lines + 1] = "Busiest presentation seconds (cue + status cue + target cue + buff events):"
  lines[#lines + 1] = ""
  for i = 1, math.min(5, #busy) do
    lines[#lines + 1] = ("- t=%.0f s: %d events, mean %.1f ms, max %.1f ms"):format(busy[i].t, busy[i].cues, busy[i].mean_ms, busy[i].max_ms)
  end
  local hitches, causes = M.hitches(parsed, warmup)
  local names = {}
  for cause, n in pairs(causes) do names[#names + 1] = ("%s %d"):format(cause, n) end
  table.sort(names)
  lines[#lines + 1] = ""
  lines[#lines + 1] = ("Hitches (frames of at least %d ms after the warm-up; frame_ms is capped at 400): %d%s"):format(
    M.HITCH_MS, #hitches, #names > 0 and (" (" .. table.concat(names, ", ") .. ")") or "")
  lines[#lines + 1] = ""
  for _, h in ipairs(hitches) do
    lines[#lines + 1] = ("- t=%.1f s: %.1f ms, work %.1f ms, %s%s"):format(h.t, h.frame_ms, h.work_ms, h.cause,
      h.cause == "presentation" and h.top and (" (" .. h.top .. ")") or "")
  end
  return table.concat(lines, "\n") .. "\n", worst, hitches
end

function M.main(args)
  local log, top, warmup, out
  local i = 1
  while i <= #args do
    if args[i] == "--log" then log = args[i + 1]; i = i + 1
    elseif args[i] == "--top" then top = tonumber(args[i + 1]); i = i + 1
    elseif args[i] == "--warmup-seconds" then warmup = tonumber(args[i + 1]); i = i + 1
    elseif args[i] == "--out" then out = args[i + 1]; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  assert(log, "usage: ue_frame_windows.lua --log UE_LOG [--top N] [--warmup-seconds S] [--out FILE]")
  local f = assert(io.open(log, "rb")); local text = f:read("a"); f:close()
  local md = M.summary(M.parse(text), top, warmup)
  if out then local o = assert(io.open(out, "wb")); o:write(md); o:close() end
  io.write(md)
  return 0
end

if not UE_FRAME_WINDOWS_AS_MODULE then os.exit(M.main(arg)) end
return M
