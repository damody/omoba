-- What a hero's abilities showed in a UE live run (work line C, M6: after a hero is switched to
-- generated Rust, every ability's presentation is checked in UE). Reads the UE logs of a run and
-- counts, per ability of the hero:
--   OM_CUE ability=<id> cue=<cue> art=effect|sound|placeholder|style|unresolved
--       (catalog cue art, or `style`: an event kind such as `cast` drawn by the ability's cue style;
--       `unresolved`: a target cue id the UE catalog does not know)
--   OM_BUFF buff=<status id> unit=<id> route=aura|status_cue|ring|channel|off  (each status instance)
--   OM_CAST_TARGET_CUES cast=<id> ... played=N                              (target cues)
--   OM_STATUS_CUE buff=<status id> cue=<cue> ... art=0|1                    (status cues; the ability's v3 statuses)
--   OM_FORCED_ARC unit=<id> peak_cm=<cm>                                    (knockback arcs, whole run)
-- and lists what the content says each ability should show (cast-cues-v1.json, status-cues-v1.json)
-- next to it. "unresolved" (a cue id the UE catalog does not know) is always a defect.
--
--   tools\lua\lua.exe scripts\ue_presentation_report.lua --hero HERO_ID --log FILE [--log FILE ...] [--out FILE]
--
-- HERO_ID is the content hero id (honda_tadakatsu); its abilities come from target/site/source-catalog.json
-- (gen_source_catalog) through the hero's nobu source id.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}
M.CATALOG = "target/site/source-catalog.json"
M.ICONS = "omfue/Content/Omoba/UI/nobu-icons-v1.json"
M.CAST_CUES = "omfue/Content/Omoba/UI/cast-cues-v1.json"
M.STATUS_CUES = "omfue/Content/Omoba/UI/status-cues-v1.json"

-- log text -> { cue = {[ability] = {[art] = n, cues = {[cue] = n}}}, target = {[cast] = n},
--   status = {[status] = {played = n, art = n}}, arcs = n, peaks = {cm...} }
function M.parse(text)
  local out = { cue = {}, target = {}, status = {}, buff = {}, arcs = 0, peaks = {} }
  for line in text:gmatch("[^\n]+") do
    local ability, cue, art = line:match("OM_CUE ability=(%S+) cue=(%S+) unit=%-?%d+ art=(%a+)")
    if ability then
      local row = out.cue[ability] or { cues = {} }
      out.cue[ability] = row
      row[art] = (row[art] or 0) + 1
      row.cues[cue] = (row.cues[cue] or 0) + 1
    end
    local cast, played = line:match("OM_CAST_TARGET_CUES cast=(%S+) anchor=%-?%d+ played=(%d+)")
    if cast then out.target[cast] = (out.target[cast] or 0) + tonumber(played) end
    local buff, has_art = line:match("OM_STATUS_CUE buff=(%S+) cue=%S+ unit=%-?%d+ art=(%d)")
    if buff then
      local row = out.status[buff] or { played = 0, art = 0 }
      out.status[buff] = row
      row.played = row.played + 1
      if has_art == "1" then row.art = row.art + 1 end
    end
    local status_id, route = line:match("OM_BUFF buff=(%S+) unit=%-?%d+ route=(%a[%a_]*)")
    if status_id then
      local row = out.buff[status_id] or {}
      out.buff[status_id] = row
      row[route] = (row[route] or 0) + 1
    end
    local peak = line:match("OM_FORCED_ARC unit=%-?%d+ peak_cm=(%-?[%d.]+)")
    if peak then out.arcs = out.arcs + 1; out.peaks[#out.peaks + 1] = tonumber(peak) end
  end
  return out
end

-- hero id, catalog, nobu icon index, cast cue table, status cue table -> ordered ability rows
-- { id, cast_cues = {cue...}, target_cues = {cue...}, statuses = {{id, cue}...} }
function M.expected(hero_id, catalog, icons, cast_cues, status_cues)
  local source_id
  for _, e in ipairs(icons.entries) do if e.kind == "hero" and e.id == hero_id then source_id = e.source_id end end
  assert(source_id, "hero " .. hero_id .. " is not in " .. M.ICONS)
  local rows = {}
  for _, a in ipairs(catalog.abilities) do
    if a.hero == source_id then
      local row = { id = a.id, cast_cues = {}, target_cues = {}, statuses = {},
        passive = a.v3 and a.v3.bot and a.v3.bot.reason == "passive" or false }
      for _, cue in ipairs((cast_cues.abilities or {})[a.id] or {}) do
        local list = cue.at == "target" and row.target_cues or row.cast_cues
        list[#list + 1] = cue.cue
      end
      for _, s in ipairs(a.v3 and a.v3.statuses or {}) do
        -- A status applied as a shared runtime status (`stun`) reaches UE under that shared id.
        local shared = s.runtime_id and not s.runtime_id:find("[:{]") and s.runtime_id or nil
        if s.visibility ~= "hidden" then row.statuses[#row.statuses + 1] = { id = s.id, shared = shared, cue = status_cues.statuses[s.id] } end
      end
      rows[#rows + 1] = row
    end
  end
  return source_id, rows
end

local function count_text(row)
  if not row then return "0" end
  local parts = {}
  for _, art in ipairs({ "effect", "placeholder", "style", "sound", "unresolved" }) do
    if row[art] then parts[#parts + 1] = art .. " " .. row[art] end
  end
  return table.concat(parts, "、")
end

-- -> markdown, defects (list of strings)
function M.render(hero_id, source_id, rows, seen)
  local out = { "# " .. hero_id .. "（" .. source_id .. "）UE 呈現檢查", "",
    "| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |", "|---|---|---|---|---|" }
  local defects = {}
  for _, r in ipairs(rows) do
    local cue_row = seen.cue[r.id]
    local target = seen.target[r.id] or 0
    local statuses = {}
    local status_total = 0
    for _, s in ipairs(r.statuses) do
      local routes, shown = {}, 0
      for _, route in ipairs({ "aura", "status_cue", "ring", "channel", "off" }) do
        local n = (seen.buff[s.id] or (s.shared and seen.buff[s.shared]) or {})[route]
        if n then routes[#routes + 1] = route .. " " .. n; if route ~= "off" then shown = shown + n end end
      end
      status_total = status_total + shown
      statuses[#statuses + 1] = s.id .. (s.shared and ("（" .. s.shared .. "）") or "") .. "：" .. (#routes > 0 and table.concat(routes, "、") or "0")
    end
    local verdict = "有呈現"
    if cue_row and cue_row.unresolved then
      verdict = "缺 cue 定義"
      defects[#defects + 1] = r.id .. ": " .. cue_row.unresolved .. " cue plays unresolved"
    elseif not cue_row and target == 0 and status_total == 0 and r.passive
      and #r.cast_cues == 0 and #r.target_cues == 0 and #r.statuses == 0 then
      -- A passive with no cue and no public status (extra damage, a heal on cast) has nothing
      -- these markers can show; damage and heal numbers are not checked here.
      verdict = "被動，無 cue／狀態可檢查"
    elseif not cue_row and target == 0 and status_total == 0 then
      verdict = "本局沒有呈現"
      defects[#defects + 1] = r.id .. ": nothing shown"
    end
    if #r.target_cues > 0 and target == 0 and verdict == "有呈現" then verdict = "有呈現（目標 cue 沒出現）" end
    out[#out + 1] = string.format("| `%s` | %s | %d（預期 %d 個） | %s | %s |", r.id, count_text(cue_row), target, #r.target_cues,
      #statuses > 0 and table.concat(statuses, "；") or "—", verdict)
  end
  out[#out + 1] = ""
  out[#out + 1] = "擊退拋物線（全局）：" .. seen.arcs .. " 次" .. (seen.arcs > 0 and ("，最高 " .. math.floor(math.max(table.unpack(seen.peaks))) .. " cm") or "")
  return table.concat(out, "\n") .. "\n", defects
end

function M.main(args)
  local hero, logs, out = nil, {}, nil
  local i = 1
  while i <= #args do
    if args[i] == "--hero" then hero = args[i + 1]; i = i + 1
    elseif args[i] == "--log" then logs[#logs + 1] = args[i + 1]; i = i + 1
    elseif args[i] == "--out" then out = args[i + 1]; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  assert(hero and #logs > 0, "usage: ue_presentation_report.lua --hero HERO_ID --log FILE [--log FILE ...] [--out FILE]")
  local function read(p) local f = assert(io.open(p, "rb"), p); local s = f:read("a"); f:close(); return s end
  local function root(p) return b.root .. "/" .. p end
  local text = {}
  for _, p in ipairs(logs) do text[#text + 1] = read(p) end
  local seen = M.parse(table.concat(text, "\n"))
  local source_id, rows = M.expected(hero, json.decode(read(root(M.CATALOG))), json.decode(read(root(M.ICONS))),
    json.decode(read(root(M.CAST_CUES))), json.decode(read(root(M.STATUS_CUES))))
  local md, defects = M.render(hero, source_id, rows, seen)
  if out then local f = assert(io.open(out, "wb")); f:write(md); f:close() else io.write(md) end
  for _, d in ipairs(defects) do print("DEFECT " .. d) end
  return 0
end

if not UE_PRESENTATION_REPORT_AS_MODULE then os.exit(M.main(arg)) end
return M
