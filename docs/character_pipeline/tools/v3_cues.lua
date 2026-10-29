-- Presentation cue catalog (history-nobu full port, presentation line): every original particle and
-- sound the v3 entries' source uses, and which v3 cue id each one belongs to. Reads only paths from
-- the original checkout; no asset is copied. Art is later work: cues without an original asset use
-- the generic placeholder effect.
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_cues.lua [--source C:\nobu\history_nobu] [--out DIR]
--
-- Writes DIR (default docs/character_pipeline/presentation):
--   cue-catalog.lua   return { schema, source, cues = { [cue id] = { kind, kinds?, uses, assets, placeholder? } },
--                     assets = { [asset path] = { kind, cues, refs } }, unmapped = { [entry] = { assets } } }
--   cue-catalog.md    summary for people (counts, placeholders, unmapped assets per hero)
-- Sources scanned:
--   KV (.txt under scripts/npc): EffectName (modifier, AttachEffect, FireEffect, projectiles, FireSound),
--     OverheadEffect, StatusEffectName, precache "particle" / "soundfile" (listed, never mapped);
--   vscripts (.lua): string literals ending in .vpcf (CreateParticle* and the locals they read) and the
--     first string argument of EmitSound*, StartSoundEvent*, EmitGlobalSound, EmitAnnouncerSound.
-- Mapping, per v3 entry (ability or item) and its `provenance` file:line ranges:
--   1. a status `cue` takes the effect of the original modifier its notes name (modifier_X), confidence high;
--   2. every other cue takes the best scoring candidate of its kind in the entry's ranges (a cue
--      named vo / sound / voice / title / load takes sounds, any other particles): same slot prefix
--      (c17w ~ particles/c17w4/...), shared name tokens, an asset no other cue took; medium/low;
--   3. a cue with no candidate gets placeholder = "generic" ("generic_sound" for sound names);
--      candidates no cue took are `unmapped`.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local lfs = require("lfs")
local M = {}
M.SCHEMA = "v3-cues/1"
M.SCRIPTS = "game/dota_addons/nobu/scripts"

local function read(p) local f = io.open(p, "rb"); if not f then return nil end local t = f:read("a"); f:close(); return t end
local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

-------------------------------------------------------------------------
-- original sources
-------------------------------------------------------------------------
local PARTICLE_KEYS = { EffectName = true, OverheadEffect = true, StatusEffectName = true, particle = true }
local SOUND_KEYS = { soundfile = true }

-- KV text -> list of { key, value, line, stack (block names outside it) }
function M.kv_leaves(text)
  local out, stack = {}, {}
  local line, i, n = 1, 1, #text
  local pending -- last key waiting for a value or a block
  while i <= n do
    local c = text:sub(i, i)
    if c == "\n" then line = line + 1; i = i + 1
    elseif c == "/" and text:sub(i + 1, i + 1) == "/" then
      local j = text:find("\n", i, true) or n + 1
      i = j
    elseif c == "{" then stack[#stack + 1] = pending or "?"; pending = nil; i = i + 1
    elseif c == "}" then stack[#stack] = nil; pending = nil; i = i + 1
    elseif c == '"' then
      local j = text:find('"', i + 1, true) or n
      local s = text:sub(i + 1, j - 1)
      for _ in s:gmatch("\n") do line = line + 1 end
      if pending then
        local st = {}
        for k, v in ipairs(stack) do st[k] = v end
        out[#out + 1] = { key = pending, value = s, line = line, stack = st }
        pending = nil
      else pending = s end
      i = j + 1
    elseif c:match("%s") then i = i + 1
    else
      -- bare token
      local j = i
      while j <= n and not text:sub(j, j):match('[%s{}"]') do j = j + 1 end
      local s = text:sub(i, j - 1)
      if pending then
        local st = {}
        for k, v in ipairs(stack) do st[k] = v end
        out[#out + 1] = { key = pending, value = s, line = line, stack = st }
        pending = nil
      else pending = s end
      i = j
    end
  end
  return out
end

local function kind_of_path(v)
  if v:match("%.vpcf$") or v:match("%.vpcf_c$") then return "particle" end
  if v:match("%.vsndevts$") or v:match("%.vsnd$") then return "soundfile" end
  if v:match("^[%w_]+%.[%w_%.]+$") and not v:match("/") then return "sound" end
  return nil
end

-- assets of one KV file: { kind, path, line, context, precache }
function M.kv_assets(text)
  local out = {}
  for _, l in ipairs(M.kv_leaves(text)) do
    if PARTICLE_KEYS[l.key] or SOUND_KEYS[l.key] then
      local kind = kind_of_path(l.value)
      if kind then
        local precache = false
        for _, s in ipairs(l.stack) do if s == "precache" then precache = true end end
        out[#out + 1] = { kind = kind, path = l.value, line = l.line, context = table.concat(l.stack, " > "),
          precache = precache, modifier = (function() for k = #l.stack, 1, -1 do if l.stack[k]:match("^modifier_") then return l.stack[k] end end end)() }
      end
    end
  end
  return out
end

local SOUND_FNS = { "EmitSoundOnLocationWithCaster", "EmitSoundOnLocationForAllies", "EmitSoundOnClient", "EmitSoundOn",
  "EmitGlobalSound", "EmitAnnouncerSound", "StartSoundEventFromPosition", "StartSoundEvent", "EmitSound" }

-- assets of one Lua script
function M.lua_assets(text)
  local out = {}
  local ln = 0
  local fn
  for line in (text .. "\n"):gmatch("([^\n]*)\n") do
    ln = ln + 1
    local code = line:gsub("\r$", "")
    local commented = code:match("^%s*%-%-") ~= nil
    local f = code:match("^%s*function%s+([%w_%.:]+)") or code:match("^%s*local%s+function%s+([%w_]+)")
    if f then fn = f end
    for s in code:gmatch('"([^"]+%.vpcf)"') do out[#out + 1] = { kind = "particle", path = s, line = ln, context = fn, commented = commented } end
    for s in code:gmatch("'([^']+%.vpcf)'") do out[#out + 1] = { kind = "particle", path = s, line = ln, context = fn, commented = commented } end
    for _, name in ipairs(SOUND_FNS) do
      for s in code:gmatch(name .. '%s*%(%s*"([^"]+)"') do out[#out + 1] = { kind = "sound", path = s, line = ln, context = fn, commented = commented } end
      for s in code:gmatch(name .. "%s*%(%s*'([^']+)'") do out[#out + 1] = { kind = "sound", path = s, line = ln, context = fn, commented = commented } end
    end
  end
  -- one occurrence per (kind, path, line)
  local seen, uniq = {}, {}
  for _, a in ipairs(out) do
    local k = a.kind .. a.path .. a.line
    if not seen[k] then seen[k] = true; uniq[#uniq + 1] = a end
  end
  return uniq
end

local file_cache = {}
local function assets_of_file(scripts_dir, rel)
  if file_cache[rel] == nil then
    local text = read(scripts_dir .. "/" .. rel)
    if not text then file_cache[rel] = false
    elseif rel:match("%.lua$") then file_cache[rel] = M.lua_assets(text)
    else file_cache[rel] = M.kv_assets(text) end
  end
  return file_cache[rel] or nil
end

-- "npc/C_Neutral/C17.txt:47-160" / "x.lua:12" / "x.lua:3,8-9" -> rel, { {a,b}, ... } (nil ranges = no lines)
local function parse_ref(s)
  if type(s) ~= "string" then return nil end
  local rel, rest = s:match("^([%w_%./%-]+%.%a+):([%d,%-]+)")
  if not rel then rel = s:match("^([%w_%./%-]+%.%a+)$") end
  if not rel then return nil end
  local ranges
  if rest then
    ranges = {}
    for part in rest:gmatch("[^,]+") do
      local a, b = part:match("^(%d+)%-(%d+)$")
      if a then ranges[#ranges + 1] = { tonumber(a), tonumber(b) }
      elseif part:match("^%d+$") then ranges[#ranges + 1] = { tonumber(part), tonumber(part) } end
    end
  end
  return rel, ranges
end

-------------------------------------------------------------------------
-- v3 cues
-------------------------------------------------------------------------
local function tokens(s)
  local t = {}
  for w in s:lower():gmatch("[a-z]+%d*") do t[w] = true end
  for w in s:lower():gmatch("%a+") do t[w] = true end
  return t
end
local SOUND_WORDS = { vo = true, sound = true, voice = true, sfx = true, snd = true, cast_sound = true, load = true, title = true }

-- cue uses of one entry: { id, where, status = def? }
-- cue uses of an entry, with what they hang on: the enclosing action (`op`), the action list key
-- (`list`, e.g. on_hit), the nearest event or trigger (`event`), a status, a unit template, or the
-- action whose `cue` field it is (`field_op`)
local function entry_cues(def)
  local out = {}
  local function describe(ctx, extra)
    local parts = {}
    if ctx.status then parts[#parts + 1] = "status " .. ctx.status end
    if ctx.template then parts[#parts + 1] = "template " .. ctx.template end
    if ctx.event then parts[#parts + 1] = ctx.event end
    if ctx.op then parts[#parts + 1] = "in " .. ctx.op .. (ctx.list and ctx.list ~= "actions" and ("." .. ctx.list) or "") end
    if extra then parts[#parts + 1] = extra end
    return table.concat(parts, " ")
  end
  local function walk(t, ctx)
    if type(t) ~= "table" then return end
    if t.op == "cue" and type(t.id) == "string" then
      out[#out + 1] = { id = t.id, where = describe(ctx, "cue"), parent_op = ctx.op, list = ctx.list, event = ctx.event,
        template = ctx.template }
    end
    for _, k in ipairs(sorted_keys(t)) do
      local v = t[k]
      local c = { op = ctx.op, list = ctx.list, event = ctx.event, status = ctx.status, template = ctx.template }
      if t.op and t.op ~= "cue" then c.op, c.list = t.op, k end
      if type(k) == "string" and (k:match("^on_") or k == "periodic") then c.event = k end
      if k == "statuses" and type(v) == "table" then
        for _, s in ipairs(v) do
          if type(s) == "table" then
            local sc = { status = tostring(s.id), template = ctx.template }
            if type(s.cue) == "string" then out[#out + 1] = { id = s.cue, where = describe(sc, ".cue"), status = s } end
            walk(s, sc)
          end
        end
      elseif k == "unit_templates" and type(v) == "table" then
        for _, u in ipairs(v) do
          if type(u) == "table" then
            local tc = { template = tostring(u.id) }
            if type(u.cue) == "string" then out[#out + 1] = { id = u.cue, where = describe(tc, ".cue"), template = u.id } end
            walk(u, tc)
          end
        end
      elseif k == "cue" and type(v) == "string" and t.op ~= "cue" and not t.stacking then
        if not (ctx.template and not t.op) then
          out[#out + 1] = { id = v, where = describe(ctx, (t.op and (t.op .. ".cue") or ".cue")), field_op = t.op,
            template = ctx.template }
        end
      elseif type(v) == "table" then walk(v, c) end
    end
  end
  walk(def, {})
  return out
end

-- the kind of one cue use (work line C's generic placeholders): what it hangs on, else the asset kind
M.FIELD_KINDS = { projectile = "projectile", zone = "area", tether = "link", create_unit = "summon", create_illusion = "summon" }
function M.use_kind(use, cue_id, asset_kind)
  if use.status then return "aura" end
  if use.field_op then return M.FIELD_KINDS[use.field_op] or "effect" end
  if use.template then return "summon" end
  if cue_id:lower():find("gesture", 1, true) then return "gesture" end
  if use.event and use.event:match("^on_channel") then return "channel" end
  if use.list == "on_hit" then return "impact" end
  return asset_kind == "sound" and "sound" or "effect"
end

-- doc -> list of { entry, kind, slot_prefix, cues, refs }
local function doc_entries(doc)
  local out = {}
  local function add(def, kind)
    local refs = {}
    for _, p in ipairs(def.provenance or {}) do refs[#refs + 1] = p end
    for _, q in ipairs(def.quirks or {}) do if q.ref then refs[#refs + 1] = q.ref end end
    local prefix = (def.source_id or ""):lower()
    out[#out + 1] = { entry = def.id, kind = kind, prefix = prefix, cues = entry_cues(def), refs = refs, def = def }
  end
  for _, a in ipairs(doc.abilities or {}) do add(a, "ability") end
  for _, it in ipairs(doc.items or {}) do add(it, "item") end
  -- doc-level unit templates and statuses: their own pseudo entry
  local shared = { id = doc.id .. ":shared", statuses = doc.statuses, unit_templates = doc.unit_templates, routines = doc.routines }
  local e = { entry = shared.id, kind = "shared", prefix = "", cues = entry_cues(shared), refs = {}, def = shared }
  for _, t in ipairs(doc.unit_templates or {}) do for _, p in ipairs(t.provenance or {}) do e.refs[#e.refs + 1] = p end end
  if #e.cues > 0 then out[#out + 1] = e end
  return out
end

local function in_ranges(line, ranges)
  if not ranges then return true end
  for _, r in ipairs(ranges) do if line >= r[1] and line <= r[2] then return true end end
  return false
end

function M.build(source_root, docs)
  local scripts_dir = source_root .. "/" .. M.SCRIPTS
  assert(lfs.attributes(scripts_dir, "mode") == "directory", "original scripts not found: " .. scripts_dir)
  local cues, assets, unmapped = {}, {}, {}
  local function asset_row(a, rel)
    local row = assets[a.path]
    if not row then row = { kind = a.kind, cues = {}, refs = {} }; assets[a.path] = row end
    local ref = rel .. ":" .. a.line
    local have = false
    for _, r in ipairs(row.refs) do if r == ref then have = true end end
    if not have then row.refs[#row.refs + 1] = ref end
    return row
  end
  for _, d in ipairs(docs) do
    for _, e in ipairs(doc_entries(d.doc)) do
      -- candidates in this entry's ranges
      local cands, modifier_files = {}, {}
      local seen = {}
      for _, r in ipairs(e.refs) do
        local rel, ranges = parse_ref(r)
        local list = rel and assets_of_file(scripts_dir, rel)
        if list then
          if rel:match("%.txt$") then modifier_files[rel] = list end
          if ranges then
            for _, a in ipairs(list) do
              local k = rel .. ":" .. a.line .. ":" .. a.path
              if not seen[k] and in_ranges(a.line, ranges) and not a.commented and a.kind ~= "soundfile" then
                seen[k] = true
                cands[#cands + 1] = { kind = a.kind, path = a.path, line = a.line, rel = rel, context = a.context, modifier = a.modifier, precache = a.precache }
              end
            end
          end
        end
      end
      -- an asset (by path) a cue already took in this entry
      local taken = {}
      local function pick(c)
        taken[c.path] = true
        return c
      end
      -- status cues first: their modifier match is the strongest evidence
      local ordered = {}
      for _, use in ipairs(e.cues) do if use.status then ordered[#ordered + 1] = use end end
      for _, use in ipairs(e.cues) do if not use.status then ordered[#ordered + 1] = use end end
      for _, use in ipairs(ordered) do
        local cue = cues[use.id]
        if not cue then cue = { uses = {}, assets = {} }; cues[use.id] = cue end
        local use_row = { doc = d.doc.id, entry = e.entry, where = use.where }
        cue.uses[#cue.uses + 1] = use_row
        local chosen, conf
        -- 1. status cue -> the named original modifier's effect
        if use.status then
          local names = {}
          for m in tostring(use.status.notes or ""):gmatch("(modifier_[%w_]+)") do names[#names + 1] = m end
          for _, m in ipairs(names) do
            for _, c in ipairs(cands) do
              if not chosen and c.modifier == m and c.kind == "particle" and not c.precache then chosen, conf = c, "high" end
            end
            if not chosen then
              for _, rel in ipairs(sorted_keys(modifier_files)) do
                local list = modifier_files[rel]
                for _, a in ipairs(list) do
                  if not chosen and a.modifier == m and a.kind == "particle" and not a.precache then
                    chosen, conf = { kind = a.kind, path = a.path, line = a.line, rel = rel, context = a.context, modifier = a.modifier }, "high"
                  end
                end
              end
            end
          end
        end
        -- 2. best candidate by score
        if not chosen then
          local ct = tokens(use.id)
          local want_sound = false
          for w in pairs(ct) do if SOUND_WORDS[w] then want_sound = true end end
          local best, best_score
          for _, c in ipairs(cands) do
            -- a cue named as a sound (vo, sound, voice) only takes sounds; any other cue only particles
            if not c.precache and (c.kind == "sound") == want_sound then
              local score = 0
              local pt = tokens(c.path)
              if e.prefix ~= "" and c.path:lower():find(e.prefix, 1, true) then score = score + 3 end
              for w in pairs(ct) do if #w > 2 and pt[w] then score = score + 2 end end
              score = score + 2
              if taken[c.path] then score = score - 4 end
              if not best or score > best_score or (score == best_score and c.line < best.line) then best, best_score = c, score end
            end
          end
          if best and best_score >= 2 then chosen, conf = best, best_score >= 5 and "medium" or "low" end
        end
        do
          local asset_kind = chosen and chosen.kind
          if not asset_kind then
            asset_kind = "particle"
            for w in pairs(tokens(use.id)) do if SOUND_WORDS[w] then asset_kind = "sound" end end
          end
          use_row.kind = M.use_kind(use, use.id, asset_kind)
        end
        if chosen then
          pick(chosen)
          local row = asset_row(chosen, chosen.rel)
          local have = false
          for _, x in ipairs(row.cues) do if x == use.id then have = true end end
          if not have then row.cues[#row.cues + 1] = use.id end
          local dup = false
          for _, a in ipairs(cue.assets) do if a.path == chosen.path then dup = true end end
          if not dup then cue.assets[#cue.assets + 1] = { path = chosen.path, kind = chosen.kind, ref = chosen.rel .. ":" .. chosen.line, confidence = conf, entry = e.entry } end
        end
      end
      local left = {}
      for _, c in ipairs(cands) do
        if not taken[c.path] and not c.precache then
          asset_row(c, c.rel)
          left[#left + 1] = { path = c.path, kind = c.kind, ref = c.rel .. ":" .. c.line, context = c.context }
        end
      end
      if #left > 0 then unmapped[e.entry] = left end
    end
  end
  for id, cue in pairs(cues) do
    -- kind: the most frequent use kind (ties alphabetical); kinds: every kind when there are several
    local count, list = {}, {}
    for _, u in ipairs(cue.uses) do
      if u.kind and not count[u.kind] then list[#list + 1] = u.kind end
      if u.kind then count[u.kind] = (count[u.kind] or 0) + 1 end
    end
    table.sort(list, function(a, b) if count[a] ~= count[b] then return count[a] > count[b] end return a < b end)
    cue.kind = list[1]
    if #list > 1 then
      local sorted = {}
      for i, k in ipairs(list) do sorted[i] = k end
      table.sort(sorted)
      cue.kinds = sorted
    end
    if #cue.assets == 0 then
      local sound = false
      for w in pairs(tokens(id)) do if SOUND_WORDS[w] then sound = true end end
      cue.placeholder = sound and "generic_sound" or "generic"
    end
  end
  return { schema = M.SCHEMA, cues = cues, assets = assets, unmapped = unmapped }
end

-------------------------------------------------------------------------
-- output
-------------------------------------------------------------------------
local function lua_literal(v, indent)
  indent = indent or ""
  if type(v) == "string" then return string.format("%q", v) end
  if type(v) ~= "table" then return tostring(v) end
  local is_list = #v > 0 or next(v) == nil
  local parts = {}
  if is_list then
    for _, x in ipairs(v) do parts[#parts + 1] = indent .. "  " .. lua_literal(x, indent .. "  ") end
  else
    for _, k in ipairs(sorted_keys(v)) do
      local key = type(k) == "string" and k:match("^[%a_][%w_]*$") and k or ("[" .. string.format("%q", k) .. "]")
      parts[#parts + 1] = indent .. "  " .. key .. " = " .. lua_literal(v[k], indent .. "  ")
    end
  end
  if #parts == 0 then return "{}" end
  return "{\n" .. table.concat(parts, ",\n") .. ",\n" .. indent .. "}"
end

function M.render_lua(r)
  return "-- GENERATED by docs/character_pipeline/tools/v3_cues.lua from the v3 documents and the original\n" ..
    "-- history_nobu sources (paths only, nothing copied); do not edit. See cue-catalog.md.\n" ..
    "return " .. lua_literal({ schema = r.schema, source = r.source, cues = r.cues, assets = r.assets, unmapped = r.unmapped }) .. "\n"
end

function M.render_md(r)
  local out = {}
  local function w(s) out[#out + 1] = s end
  local ncue, nmapped, nplace, conf = 0, 0, 0, { high = 0, medium = 0, low = 0 }
  local placeholders = {}
  for id, c in pairs(r.cues) do
    ncue = ncue + 1
    if c.placeholder then nplace = nplace + 1; placeholders[#placeholders + 1] = id else
      nmapped = nmapped + 1
      local best = "low"
      for _, a in ipairs(c.assets) do if a.confidence == "high" or (a.confidence == "medium" and best == "low") then best = a.confidence end end
      conf[best] = conf[best] + 1
    end
  end
  table.sort(placeholders)
  local nassets, used, kinds = 0, 0, { particle = 0, sound = 0 }
  for _, a in pairs(r.assets) do nassets = nassets + 1; if #a.cues > 0 then used = used + 1 end; kinds[a.kind] = (kinds[a.kind] or 0) + 1 end
  w("# 呈現 cue 目錄")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_cues.lua` 從 v3 文件與原作 history_nobu（commit " .. tostring(r.source and r.source.commit) ..
    "）產生，不要手改。機器可讀的資料在 [cue-catalog.lua](cue-catalog.lua)。原作資產只記錄路徑，沒有複製。")
  w("")
  w("## 總計")
  w("")
  w(("- v3 文件用到的 cue：%d 個。對到原作資產的有 %d 個（high %d、medium %d、low %d），其餘 %d 個使用通用佔位特效（`placeholder = \"generic\"`，音效為 `\"generic_sound\"`）。"):format(
    ncue, nmapped, conf.high, conf.medium, conf.low, nplace))
  w(("- 條目出處範圍內的原作資產：%d 個（粒子 %d、音效 %d），其中 %d 個已對到 cue。"):format(nassets, kinds.particle or 0, kinds.sound or 0, used))
  w("- 信心：")
  w("  - **high**：狀態的 cue 對到 notes 寫的原作 modifier 的 EffectName。")
  w("  - **medium**：路徑含技能編號前綴，且名稱字詞相同。")
  w("  - **low**：只靠同一條目與種類（粒子或音效）判斷，UE 線導入美術時要人工確認。")
  w("- 沒對到 cue 的原作資產列在 `unmapped`（依條目），表示 v3 資料還沒有對應的呈現；UE 線可以補 cue，或維持不顯示。")
  w("")
  local kinds = {}
  for _, c in pairs(r.cues) do if c.kind then kinds[c.kind] = (kinds[c.kind] or 0) + 1 end end
  local kk = {}
  for k, v in pairs(kinds) do kk[#kk + 1] = { k, v } end
  table.sort(kk, function(a, b) return a[2] > b[2] end)
  w("## cue 類型（kind）")
  w("")
  w("依 cue 掛在哪裡推導，給 UE 選通用佔位特效：projectile（投射物）、area（地面區域 zone）、link（tether）、aura（狀態）、summon（召喚與單位模板）、channel（引導事件）、impact（命中回呼 on_hit）、gesture（名稱含 gesture）、其他獨立 cue 依資產分成 effect 或 sound。一個 cue 有多種用途時，`kind` 是最常見的那一種，`kinds` 列出全部。")
  w("")
  w("| kind | cue 數 |")
  w("|---|---|")
  for _, x in ipairs(kk) do w(("| %s | %d |"):format(x[1], x[2])) end
  w("")
  w("## 使用通用佔位特效的 cue")
  w("")
  w(#placeholders > 0 and ("`" .. table.concat(placeholders, "`、`") .. "`") or "（無）")
  w("")
  w("## 各條目沒對到 cue 的原作資產數")
  w("")
  w("| 條目 | 數量 |")
  w("|---|---|")
  for _, k in ipairs(sorted_keys(r.unmapped)) do w(("| `%s` | %d |"):format(k, #r.unmapped[k])) end
  w("")
  return table.concat(out, "\n")
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  package.path = root .. "/scripts/?.lua;scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  local source, out_dir = "C:/nobu/history_nobu", b.root .. "/docs/character_pipeline/presentation"
  local i = 1
  while i <= #argv do
    if argv[i] == "--source" then source = argv[i + 1]; i = i + 1
    elseif argv[i] == "--out" then out_dir = argv[i + 1]; i = i + 1
    else error("unknown argument " .. argv[i]) end
    i = i + 1
  end
  source = source:gsub("\\", "/")
  PACKAGE_V3_AS_MODULE = true
  local P = dofile(tools_dir .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  local S = dofile(tools_dir .. "/v3_sources.lua")
  local docs, commit = {}, nil
  for _, f in ipairs(S.all(b.root)) do
    local doc = assert(P.load_file(f))
    commit = commit or (doc.source and doc.source.commit)
    docs[#docs + 1] = { path = f, doc = doc }
  end
  local r = M.build(source, docs)
  r.source = { repository = "history_nobu", commit = commit, root = M.SCRIPTS }
  b.lib("path").mkdir_p(out_dir)
  local f = assert(io.open(out_dir .. "/cue-catalog.lua", "wb")); f:write(M.render_lua(r)); f:close()
  f = assert(io.open(out_dir .. "/cue-catalog.md", "wb")); f:write(M.render_md(r), "\n"); f:close()
  local n, p = 0, 0
  for _, c in pairs(r.cues) do n = n + 1; if c.placeholder then p = p + 1 end end
  print(("wrote %s: %d cues (%d mapped, %d placeholder)"):format(out_dir, n, n - p, p))
  return 0
end

if rawget(_G, "V3_CUES_AS_MODULE") then return M end
os.exit(M.main(arg))
