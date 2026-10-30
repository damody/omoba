-- What the shelf items that run from items.dll show in UE (work line C, M7): for every item the script
-- DLL runs (registered and on the shelf), its icon (nobu icon index or generated art), its active
-- (the shelf row's cooldown, which the item slot counts down), every cue its document plays (cue
-- catalog kind or a cue setting, so UE draws at least the kind's placeholder) and every status a
-- player sees (a `buff`/`debuff` status is a status icon only when templates/status_buffs.lua gives
-- it a buff: its key must be the BuffStore key the script holds it under, with a name to show).
--
--   tools\lua\lua.exe scripts\item_presentation_audit.lua --generated FILE.rs [--out FILE.md]
--
-- FILE.rs is the items crate's generated source (scripts/target/<profile>/build/items_impl-*/out/
-- generated.rs); its `super::item_<id>::ffi()` rows are the items the DLL registers.
package.path = "scripts/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")
local lfs = require("lfs")

local M = {}
M.ITEMS_DIR = "docs/character_pipeline/items"
M.ICONS = "omfue/Content/Omoba/UI/nobu-icons-v1.json"
M.GENERATED = "omfue/Content/Omoba/UI/generated-art-v1.json"
M.CUES = "docs/character_pipeline/presentation/cue-catalog.lua"
M.SETTINGS = "omfue/Config/DefaultGame.ini"

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  local v = assert(loadfile(path, "t", env))()
  if type(v) == "function" then v = v({}) end
  return v
end

-- every `op = "cue"` id and status `cue` in a table, recursively
local function cues_in(t, out)
  if type(t) ~= "table" then return out end
  if t.op == "cue" and type(t.id) == "string" then out[t.id] = true end
  for k, v in pairs(t) do
    if k == "cue" and type(v) == "string" then out[v] = true end
    if type(v) == "table" then cues_in(v, out) end
  end
  return out
end

-- the item ids items.dll registers
function M.registered(generated_text)
  local ids, seen = {}, {}
  for id in generated_text:gmatch("super::item_([%w_]+)::ffi%(%)") do
    if not seen[id] then seen[id] = true; ids[#ids + 1] = id end
  end
  table.sort(ids)
  return ids
end

-- cue ids a cue setting names (CueOverrides keys)
function M.setting_cues(ini_text)
  local out = {}
  for id in ini_text:gmatch('%("([%w_]+)",%(') do out[id] = true end
  return out
end

-- one item's rows; `inputs` = { doc_item, shelf_row, icons_by_source, catalog_cues, setting_cues, status_keys }
function M.audit(id, inputs)
  local r = { id = id, problems = {} }
  local d, row = inputs.doc_item, inputs.shelf_row
  r.name = d and d.text and d.text.zh_TW and d.text.zh_TW.name or id
  if not row then r.problems[#r.problems + 1] = "不在上架目錄（腳本已註冊但商店沒有這件）" end
  local icon = inputs.icons[d and d.source_id or ("item_" .. id)]
  r.icon = icon and icon.status or "missing"
  if r.icon ~= "original" and r.icon ~= "generated" then r.problems[#r.problems + 1] = "圖示缺（" .. (icon and icon.reason or "不在圖示索引") .. "）" end
  r.active = d and d.active ~= nil
  if r.active and row and not (row.cooldown and row.cooldown > 0) then
    r.problems[#r.problems + 1] = "主動技能在上架目錄沒有冷卻（道具格不會倒數）"
  end
  r.cues = {}
  local cues = cues_in(d and { d.active, d.passive, d.statuses } or {}, {})
  for cue in pairs(cues) do
    local kind = inputs.catalog_cues[cue] and inputs.catalog_cues[cue].kind
    local set = inputs.setting_cues[cue]
    r.cues[#r.cues + 1] = { id = cue, kind = kind, setting = set }
    if not kind and not set then r.problems[#r.problems + 1] = "cue `" .. cue .. "` 不在 cue 目錄也沒有 cue 設定（UE 不會畫）" end
  end
  table.sort(r.cues, function(x, y) return x.id < y.id end)
  r.statuses = {}
  for _, s in ipairs(d and d.statuses or {}) do
    if s.visibility == "buff" or s.visibility == "debuff" then
      local key = s.runtime_id or s.id
      local named = s.text and s.text.zh_TW and s.text.zh_TW.name
      local icon_ok = inputs.status_keys[key] == true
      r.statuses[#r.statuses + 1] = { id = s.id, key = key, named = named ~= nil, shown = icon_ok }
      if not icon_ok then r.problems[#r.problems + 1] = "狀態 `" .. s.id .. "`（key `" .. key .. "`）沒有 buff，UE 沒有狀態圖示" end
      if not named then r.problems[#r.problems + 1] = "狀態 `" .. s.id .. "` 沒有 zh_TW 名稱" end
    end
  end
  return r
end

function M.render(rows)
  local lines = { "# 上架道具在 UE 的呈現（items.dll，工作線 C）", "",
    "由 `scripts/item_presentation_audit.lua` 產生。只列 items.dll 有註冊、而且在上架目錄的道具。", "",
    "| 道具 | 圖示 | 主動 | cue | 玩家看得到的狀態 | 問題 |", "|---|---|---|---|---|---|" }
  local bad = 0
  for _, r in ipairs(rows) do
    local cues = {}
    for _, c in ipairs(r.cues) do cues[#cues + 1] = c.id .. (c.kind and ("（" .. c.kind .. "）") or c.setting and "（設定）" or "（缺）") end
    local st = {}
    for _, s in ipairs(r.statuses) do st[#st + 1] = s.id .. (s.shown and "" or "（缺）") end
    if #r.problems > 0 then bad = bad + 1 end
    lines[#lines + 1] = string.format("| %s `%s` | %s | %s | %s | %s | %s |", r.name, r.id, r.icon, r.active and "有" or "—",
      #cues > 0 and table.concat(cues, "、") or "—", #st > 0 and table.concat(st, "、") or "—",
      #r.problems > 0 and table.concat(r.problems, "；") or "")
  end
  table.insert(lines, 4, string.format("共 %d 件，有問題 %d 件。", #rows, bad))
  table.insert(lines, 5, "")
  return table.concat(lines, "\n") .. "\n", bad
end

function M.main(args)
  local generated, out
  local i = 1
  while i <= #args do
    if args[i] == "--generated" then i = i + 1; generated = args[i]
    elseif args[i] == "--out" then i = i + 1; out = args[i]
    else error("unknown argument: " .. tostring(args[i])) end
    i = i + 1
  end
  assert(generated, "usage: item_presentation_audit.lua --generated FILE.rs [--out FILE.md]")
  local ids = M.registered(assert(io.open(generated, "rb")):read("a"))
  -- the item documents by shelf id
  local docs = {}
  for name in lfs.dir(M.ITEMS_DIR) do
    if name:match("%.lua$") then
      for _, it in ipairs(load_doc(M.ITEMS_DIR .. "/" .. name).items or {}) do docs[(it.id:gsub("^item_", ""))] = it end
    end
  end
  local ctx = require("content_builder").new(b.root .. "/scripts/lua_data")
  local shelf = {}
  for _, row in ipairs(ctx.include("templates/moba_items.lua")) do if not row.test_only then shelf[row.id] = row end end
  -- the status keys that get a buff (status_buffs.lua over the shelf items' v3 statuses)
  local status_keys = {}
  -- the same call templates.lua makes: hand-written buffs, then one per visible v3 status
  local abilities = {}
  for _, a in ipairs(ctx.include("templates/authored_heroes.lua").abilities or {}) do abilities[#abilities + 1] = a end
  local buffs = ctx.include("templates/status_buffs.lua").apply(ctx.include("templates/buffs.lua"), abilities, ctx.include("templates/item_texts_v3.lua"))
  for _, entry in ipairs(buffs) do
    if entry.runtime_id or entry.id then status_keys[entry.runtime_id or entry.id] = true end
  end
  local icons_index = json.read(M.ICONS)
  local generated_art = lfs.attributes(M.GENERATED, "mode") and json.read(M.GENERATED)
  if generated_art then
    GENERATED_ART_AS_MODULE = true
    icons_index = require("generated_art").overlay(icons_index, generated_art)
  end
  local icons = {}
  for _, e in ipairs(icons_index.entries) do if e.kind == "item" then icons[e.source_id] = e end end
  local catalog_cues = dofile(M.CUES).cues
  local setting_cues = M.setting_cues(assert(io.open(M.SETTINGS, "rb")):read("a"))
  local rows = {}
  for _, id in ipairs(ids) do
    if shelf[id] then
      rows[#rows + 1] = M.audit(id, { doc_item = docs[id], shelf_row = shelf[id], icons = icons,
        catalog_cues = catalog_cues, setting_cues = setting_cues, status_keys = status_keys })
    end
  end
  local text, bad = M.render(rows)
  if out then local f = assert(io.open(out, "wb")); f:write(text); f:close() end
  io.write(text)
  return bad
end

if not ITEM_AUDIT_AS_MODULE and arg and arg[0] and arg[0]:match("item_presentation_audit%.lua$") then os.exit(M.main(arg) == 0 and 0 or 1) end
return M
