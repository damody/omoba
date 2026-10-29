-- Original nobu icons for every history_nobu hero, ability and item (M6: heroes open in bulk and
-- each needs a portrait and icons, E020). Ids come from the v3 documents (gen_source_catalog):
--   hero     <package id>        portrait   target/nobugg_src/static/pic/hero/<source id>.png (nobu's
--                                           own site), else override_hero's panorama image in the addon
--   ability  <v3 ability id>     KV AbilityTextureName under resource/flash3/images/spellicons
--   item     <game id> (v3 id without item_)  KV AbilityTextureName (items/... or spellicons/...)
-- Only nobu's own files count: a texture name the addon does not ship is Dota's built-in art, and
-- such ids (and any file that does not decode) get the placeholder and are listed. Manual
-- assignments (docs/character_pipeline/presentation/nobu-icon-overrides.lua) win over the mapping.
-- Every original is cropped to a centred square and resampled to EDGE pixels (lua-host
-- png_square_icon); the source files are never copied as they are.
--
--   tools\lua\lua.exe scripts\import_nobu_icons.lua [--source DIR] [--site-pics DIR] [--root DIR] [--ui-dir DIR] [--check]
--
-- Writes omfue/Content/Omoba/UI/nobu-<kind>-<id>.png, the index nobu-icons-v1.json beside them
-- and the report docs/character_pipeline/presentation/nobu-icons.md. --check only verifies that
-- the index lists files that exist with their recorded hashes.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local json = b.lib("json")
local hash = b.lib("hash")

local M = {}
M.EDGE = 128
M.SCHEMA = "omoba.nobu-icons.v1"
M.UI_DIR = "omfue/Content/Omoba/UI"
M.INDEX = "nobu-icons-v1.json"
M.OVERRIDES = "docs/character_pipeline/presentation/nobu-icon-overrides.lua"
M.REPORT = "docs/character_pipeline/presentation/nobu-icons.md"
M.SITE_PICS = "target/nobugg_src/static/pic"
local GAME = "game/dota_addons/nobu"

function M.file_name(kind, id)
  assert(id:match("^[a-z0-9_]+$"), kind .. " id " .. id)
  return "nobu-" .. kind .. "-" .. id .. ".png"
end

-- package directory -> hero source id (the first source_id of its v3.lua)
function M.hero_packages(packages_dir)
  local out = {}
  for name in lfs.dir(packages_dir) do
    local f = io.open(packages_dir .. "/" .. name .. "/v3.lua", "rb")
    if f then
      local text = f:read("a"); f:close()
      local sid = text:match('source_id%s*=%s*"(%u%d%d)"')
      if sid then
        assert(not out[sid], "two packages claim hero " .. sid)
        out[sid] = name
      end
    end
  end
  return out
end

local function exists(p) return lfs.attributes(p, "mode") == "file" end

-- One row of the plan: { kind, id, source_id, status = "original"|"placeholder", reason?, origin? }
-- origin is a path relative to the source root (or "site:<rel>" for the site pictures).
function M.plan(catalog, supplement, opts)
  local overrides = opts.overrides or {}
  local rows = {}
  local function add(kind, id, source_id, origin, reason)
    local forced = overrides[kind] and overrides[kind][id]
    if forced == false then origin, reason = nil, "override: placeholder"
    elseif type(forced) == "string" then origin, reason = forced, nil end
    rows[#rows + 1] = { kind = kind, id = id, source_id = source_id, origin = origin,
      status = origin and "original" or "placeholder", reason = (not origin) and reason or nil,
      manual = forced ~= nil or nil }
  end
  local function texture_origin(source_id)
    local tex = supplement.texture(source_id)
    if not tex then return nil, "no AbilityTextureName in the original KV" end
    local rel = supplement.texture_png(tex)
    if not rel then return nil, "Dota built-in texture " .. tex end
    if not rel:find("/custom/", 1, true) then return nil, "file outside nobu's custom folders: " .. rel end
    return rel
  end
  -- site pictures count only when the review lists them as nobu's own (overrides.site_portraits)
  local review = {}
  for verdict, names in pairs(overrides.site_portraits or {}) do
    for _, name in ipairs(names) do review[name] = verdict end
  end
  for _, h in ipairs(catalog.heroes) do
    local id = assert(opts.packages[h.source_id], "no package for hero " .. h.source_id)
    local name = h.source_id:lower()
    local site = opts.site_pics and (opts.site_pics .. "/hero/" .. name .. ".png")
    local origin, reason
    if site and exists(site) and review[name] == "nobu" then origin = "site:hero/" .. name .. ".png"
    else
      origin = supplement.hero_icon(h.source_id)
      if not origin then
        reason = (site and exists(site)) and (review[name] == "dota" and "the site's portrait is Dota 2 art" or "the site's portrait is not reviewed")
          or "no nobu portrait (site or override_hero panorama)"
      end
    end
    add("hero", id, h.source_id, origin, reason)
  end
  for _, a in ipairs(catalog.abilities) do
    local sid = a.v3 and a.v3.source_id
    local origin, reason
    if sid then origin, reason = texture_origin(sid) else reason = "no source id" end
    add("ability", a.id, sid, origin, reason)
  end
  for _, it in ipairs(catalog.items) do
    local origin, reason = texture_origin(it.source_id)
    add("item", (it.id:gsub("^item_", "")), it.source_id, origin, reason)
  end
  table.sort(rows, function(x, y) if x.kind ~= y.kind then return x.kind < y.kind end return x.id < y.id end)
  return rows
end

local function origin_path(row, source_root, site_pics)
  local site = row.origin:match("^site:(.*)$")
  return site and (site_pics .. "/" .. site) or (source_root .. "/" .. row.origin)
end

-- one entry per line, so a re-import diffs by id
function M.encode_index(index)
  local entries = index.entries
  index.entries = nil
  local head = json.encode(index)
  index.entries = entries
  local lines = {}
  for i, r in ipairs(entries) do lines[i] = "  " .. json.encode(r) end
  return head:sub(1, -2) .. ',"entries":[\n' .. table.concat(lines, ",\n") .. "\n]}\n"
end

function M.render_report(index)
  local out = { "# 原作 nobu 圖示", "",
    "由 `tools\\lua\\lua.exe scripts\\import_nobu_icons.lua` 產生，不要手改。人工指定寫在 `nobu-icon-overrides.lua`。", "",
    "- 來源：" .. index.source.repository .. "@" .. index.source.commit .. "（頭像優先用 nobu 官網的英雄圖）。",
    "- 只用 nobu 自製的圖：技能與道具取 addon `resource/flash3/images` 底下 custom 資料夾的檔案；KV 指向 Dota 內建圖示的一律用佔位圖。",
    "- 每張圖裁成置中正方形、縮放成 " .. index.edge .. "×" .. index.edge .. "，存成 `" .. M.UI_DIR .. "/nobu-<kind>-<id>.png`。", "" }
  local counts = {}
  for _, r in ipairs(index.entries) do
    counts[r.kind] = counts[r.kind] or { original = 0, placeholder = 0 }
    counts[r.kind][r.status] = counts[r.kind][r.status] + 1
  end
  out[#out + 1] = "| 種類 | 原作圖 | 佔位圖 |"
  out[#out + 1] = "|---|---|---|"
  for _, k in ipairs({ "hero", "ability", "item" }) do
    local c = counts[k] or { original = 0, placeholder = 0 }
    out[#out + 1] = string.format("| %s | %d | %d |", k, c.original, c.placeholder)
  end
  out[#out + 1] = ""
  out[#out + 1] = "## 使用佔位圖的條目"
  out[#out + 1] = ""
  out[#out + 1] = "| 種類 | id | 原作 id | 原因 |"
  out[#out + 1] = "|---|---|---|---|"
  for _, r in ipairs(index.entries) do
    if r.status == "placeholder" then
      out[#out + 1] = string.format("| %s | `%s` | %s | %s |", r.kind, r.id, r.source_id or "", r.reason or "")
    end
  end
  return table.concat(out, "\n") .. "\n"
end

local function read_json(p)
  local f = io.open(p, "rb")
  if not f then return nil end
  local s = f:read("a"); f:close()
  return json.decode(s)
end

function M.check(root)
  local ui = root .. "/" .. M.UI_DIR
  local index = assert(read_json(ui .. "/" .. M.INDEX), "missing " .. M.INDEX)
  assert(index.schema == M.SCHEMA, "unknown nobu icon index schema")
  local n = 0
  for _, r in ipairs(index.entries) do
    if r.status == "original" then
      local p = ui .. "/" .. r.file
      assert(exists(p), "missing " .. r.file)
      assert(hash.sha256(p) == r.sha256, "changed " .. r.file)
      n = n + 1
    end
  end
  return n
end

function M.main(args)
  local root, source_root, site_pics, check, ui = b.root, "C:/nobu/history_nobu", nil, false, nil
  local i = 1
  while i <= #args do
    local a = args[i]
    if a == "--root" then root = args[i + 1]; i = i + 1
    elseif a == "--source" then source_root = args[i + 1]; i = i + 1
    elseif a == "--site-pics" then site_pics = args[i + 1]; i = i + 1
    elseif a == "--ui-dir" then ui = args[i + 1]; i = i + 1
    elseif a == "--check" then check = true
    else error("unknown argument " .. a) end
    i = i + 1
  end
  if check then print(string.format("nobu icons: %d originals match the index", M.check(root))); return 0 end
  site_pics = (site_pics or (root .. "/" .. M.SITE_PICS)):gsub("\\", "/")
  source_root = source_root:gsub("\\", "/")
  local supplement = require("site.source_supplement").open(source_root)
  assert(supplement.available, "original sources not found at " .. source_root)
  GEN_SOURCE_CATALOG_AS_MODULE = true
  local catalog = require("gen_source_catalog").build(root)
  GEN_SOURCE_CATALOG_AS_MODULE = nil
  local overrides = exists(root .. "/" .. M.OVERRIDES) and dofile(root .. "/" .. M.OVERRIDES) or {}
  local rows = M.plan(catalog, supplement, {
    packages = M.hero_packages(root .. "/docs/character_pipeline/packages"),
    site_pics = lfs.attributes(site_pics, "mode") == "directory" and site_pics or nil,
    overrides = overrides,
  })
  local host = b.lib("host")
  ui = (ui or (root .. "/" .. M.UI_DIR)):gsub("\\", "/")
  lfs.mkdir(ui)
  local commit = io.popen('git -C "' .. source_root .. '" rev-parse --short HEAD'):read("l") or "unknown"
  local written, failed = 0, 0
  for _, r in ipairs(rows) do
    if r.status == "original" then
      local src = origin_path(r, source_root, site_pics)
      r.file = M.file_name(r.kind, r.id)
      local ok, report = pcall(host.call, "png_square_icon", { source = src, output = ui .. "/" .. r.file, edge = M.EDGE })
      if ok then
        r.sha256, r.source_sha256 = report.output_sha256, report.source_sha256
        r.source_size = { report.source_width, report.source_height }
        written = written + 1
      else
        os.remove(ui .. "/" .. r.file)
        local head = io.open(src, "rb")
        local magic = head and head:read(3) or ""
        if head then head:close() end
        r.status, r.file = "placeholder", nil
        r.reason = (magic == "\255\216\255" and "JPEG (no JPEG decoder in the toolchain): " or "original does not decode as PNG: ") .. r.origin
        failed = failed + 1
      end
    end
  end
  -- files of ids that are now placeholders or gone
  local keep = {}
  for _, r in ipairs(rows) do if r.file then keep[r.file] = true end end
  for name in lfs.dir(ui) do
    if name:match("^nobu%-%a+%-[a-z0-9_]+%.png$") and not keep[name] then os.remove(ui .. "/" .. name) end
  end
  local index = { schema = M.SCHEMA, edge = M.EDGE,
    source = { repository = "history_nobu", commit = commit, site = "nobugg static/pic" }, entries = rows }
  local f = assert(io.open(ui .. "/" .. M.INDEX, "wb")); f:write(M.encode_index(index)); f:close()
  f = assert(io.open(root .. "/" .. M.REPORT, "wb")); f:write(M.render_report(index)); f:close()
  print(string.format("nobu icons: %d written, %d undecodable, %d placeholders in total; index %s",
    written, failed, #rows - written, ui .. "/" .. M.INDEX))
  return 0
end

if not NOBU_ICONS_AS_MODULE then os.exit(M.main(arg)) end
return M
