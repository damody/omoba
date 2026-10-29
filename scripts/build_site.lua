-- Static website from the content catalog (plan 2026-10-28-history-nobu-full-port, line E).
--
--   tools\lua\lua.exe scripts\build_site.lua [--catalog FILE] [--docs PATH ...] [--out DIR]
--                                            [--game-catalog FILE | --no-game-catalog]
--                                            [--previous FILE] [--source-root DIR] [--no-icons]
--
--   (default)      the full site: every original hero and item from the source catalog
--                  (scripts/gen_source_catalog.lua, built in-process), with the game's own
--                  values (V3_CATALOG_JSON, exported by `cargo run -p omoba-template-ids
--                  --example export_content_catalog`) for shipped heroes and shelf items.
--                  Heroes the game does not ship are marked 未上架.
--   --game-catalog the game's V3_CATALOG_JSON export to use instead of exporting it now;
--                  --no-game-catalog builds from the source catalog alone.
--   --catalog      a site catalog JSON (catalog.json of an earlier build), a source catalog
--                  or a V3_CATALOG_JSON export.
--   --docs         v3 documents or directories (repeatable), compiled by
--                  scripts/site/census_adapter.lua (tests, single documents).
--   --out          output directory (default target/site); zh-TW/ and en/ are rebuilt.
--   --previous     catalog JSON of the previous version; the patch notes compare against it.
--   --source-root  the pinned original (default C:/nobu/history_nobu) for icons and shop
--                  categories; missing icons become a placeholder.
-- The build writes catalog.json next to the pages and fails when any page check fails
-- (scripts/site/check.lua).
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json = b.lib('path'), b.lib('json')
local lfs = require('lfs')
local SHA = require('site.sha256')
local I = require('site.i18n')
local P = require('site.pages')
local CL = require('site.changelog')
local CHECK = require('site.check')

local M = {}

-- The original supplement; with icons on, its icons are the HUD's own (site/ue_icons.lua):
-- the art the game shows, else original nobu art. ICON_FILES maps each site path to its source.
local ICON_FILES = {}
local function open_supplement(source_root, icons)
  local sup = require('site.source_supplement').open(source_root)
  if icons == false or not sup.available then return sup end
  local wrapped, ok = require('site.ue_icons').wrap(sup, b.root)
  if ok then ICON_FILES = wrapped.files end
  return wrapped
end

-- a site catalog (catalog.json of a build) or the compiled catalog's v3 export
local function read_catalog(file, source_root, icons)
  local cat = json.read(file)
  local reader = require('site.catalog_reader')
  if reader.is_source(cat) then
    local sup = open_supplement(source_root, icons)
    return reader.read_source(cat, nil, sup.available and sup or nil)
  end
  if reader.is_v3_export(cat) then
    local sup = open_supplement(source_root, icons)
    return reader.read(cat, sup.available and sup or nil)
  end
  assert(cat.format == 'omoba-site-catalog', file .. ' is neither a site catalog nor a v3 content catalog export')
  return cat
end

-- content hash: sha256 of the canonical JSON (sorted keys) without the hash itself
function M.hash(catalog)
  local saved = catalog.content_hash
  catalog.content_hash = nil
  local text = json.encode(catalog)
  catalog.content_hash = saved
  return SHA.hex(text)
end

local function rm_tree(dir)
  if lfs.attributes(dir, 'mode') ~= 'directory' then return end
  for name in lfs.dir(dir) do
    if name ~= '.' and name ~= '..' then
      local p = dir .. '/' .. name
      if lfs.attributes(p, 'mode') == 'directory' then rm_tree(p) else assert(os.remove(p)) end
    end
  end
  assert(lfs.rmdir(dir))
end

local PLACEHOLDER = [[<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="6" fill="#cfc6b6"/><path d="M20 44l9-12 7 8 5-6 9 10z" fill="#9b917f"/><circle cx="24" cy="24" r="5" fill="#9b917f"/></svg>
]]

local function rank_css(max)
  local out = { '' }
  for k = 1, max do
    out[#out + 1] = string.format('.rs > input:nth-of-type(%d):checked ~ .panel:nth-of-type(%d) { display: block; }', k, k)
  end
  return table.concat(out, '\n') .. '\n'
end

-- The game's V3_CATALOG_JSON, exported now (nil with a reason when cargo cannot run).
function M.export_game_catalog()
  local out = path.join(b.root, 'target', 'site-cache', 'content-catalog.json')
  path.mkdir_p(path.parent(out))
  local r = b.lib('process').run('cargo', { 'run', '-q', '--manifest-path', path.join(b.root, 'omoba-template-ids', 'Cargo.toml'),
    '--example', 'export_content_catalog', '--', out }, { cwd = b.root, check = false })
  if r.exit_code ~= 0 or not path.is_file(out) then return nil, 'export_content_catalog failed: ' .. tostring(r.stderr):sub(1, 400) end
  return json.read(out)
end

-- opts: catalog (table) | catalog_file | docs | (default: source catalog + game catalog);
-- game_catalog_file; game_catalog = false; out; previous_file; source_root; icons (bool)
-- returns { pages = { {path, html, hero?} }, catalog, errors, warnings }
function M.build(opts)
  local errors, warnings = {}, {}
  local catalog
  local sup
  if opts.catalog then catalog = opts.catalog
  elseif opts.catalog_file then catalog = read_catalog(opts.catalog_file, opts.source_root, opts.icons)
  elseif not opts.docs then
    sup = open_supplement(opts.source_root, opts.icons)
    if not sup.available then warnings[#warnings + 1] = 'original not found at ' .. sup.root .. '; icons are placeholders' end
    rawset(_G, 'GEN_SOURCE_CATALOG_AS_MODULE', true)
    local G = dofile(path.join(b.root, 'scripts', 'gen_source_catalog.lua'))
    rawset(_G, 'GEN_SOURCE_CATALOG_AS_MODULE', nil)
    local src, problems = G.build(b.root)
    -- a document that does not validate is not published (its numbers cannot be trusted)
    for _, p in ipairs(problems) do errors[#errors + 1] = p end
    local game
    if opts.game_catalog_file then game = json.read(opts.game_catalog_file)
    elseif opts.game_catalog ~= false then
      local why
      game, why = M.export_game_catalog()
      if not game then warnings[#warnings + 1] = why .. '; shipped heroes show the source values' end
    end
    if #errors == 0 then catalog = require('site.catalog_reader').read_source(src, game, sup.available and sup or nil) end
  else
    local A = require('site.census_adapter')
    sup = open_supplement(opts.source_root, opts.icons)
    if not sup.available then warnings[#warnings + 1] = 'original not found at ' .. sup.root .. '; icons are placeholders' end
    local errs, warns
    catalog, errs, warns = A.build(opts.docs, { supplement = sup, validate = opts.validate })
    for _, e in ipairs(errs) do errors[#errors + 1] = e end
    for _, w in ipairs(warns) do warnings[#warnings + 1] = w end
  end
  if #errors > 0 then return { errors = errors, warnings = warnings } end
  -- the compiled catalog's own hash when the site renders it, so the site and the game show the same value
  catalog.content_hash = catalog.source_content_hash or M.hash(catalog)
  local C = P.context(catalog)
  local Cold
  -- without --previous, the patch notes compare against the catalog the last build left in --out
  if not opts.previous_file and opts.out and opts.auto_previous ~= false then
    local last = path.join(path.absolute(opts.out, b.root), 'catalog.json')
    if path.is_file(last) then opts.previous_file = last end
  end
  if opts.previous_file then
    local prev = read_catalog(opts.previous_file, opts.source_root)
    prev.content_hash = prev.content_hash or M.hash(prev)
    Cold = P.context(prev)
  end

  local pages = {}
  local max_rank = 1
  for _, a in ipairs(catalog.abilities) do max_rank = math.max(max_rank, (a.max_level or 1) + 1) end
  for _, lang in ipairs(I.LANGS) do
    local list = { P.home_page(C, lang.code), P.shop_page(C, lang.code), CL.page(Cold, C, lang.code, lang.dir) }
    for _, h in ipairs(catalog.heroes) do
      local pg = P.hero_page(C, h, lang.code); pg.hero = h; list[#list + 1] = pg
    end
    for _, it in ipairs(catalog.items) do list[#list + 1] = P.item_page(C, it, lang.code) end
    for _, pg in ipairs(list) do
      local html = P.layout(C, pg)
      local errs = CHECK.page(html, C, Cold, pg.hero)
      for _, e in ipairs(errs) do errors[#errors + 1] = pg.path .. ': ' .. e end
      pages[#pages + 1] = { path = pg.path, html = html, hero = pg.hero, lang = lang.code }
    end
  end
  if C.missing then
    local list = {}
    for k in pairs(C.missing) do list[#list + 1] = k end
    table.sort(list)
    warnings[#warnings + 1] = #list .. ' value(s) referenced by text but missing from the catalog (shown as ?): ' .. table.concat(list, ', ')
  end
  local result = { pages = pages, catalog = catalog, errors = errors, warnings = warnings, context = C, missing = C.missing,
    rank_css = rank_css(max_rank) }

  if opts.out and #errors == 0 then
    local out = path.absolute(opts.out, b.root)
    for _, lang in ipairs(I.LANGS) do rm_tree(out .. '/' .. lang.dir) end
    for _, pg in ipairs(pages) do path.write(out .. '/' .. pg.path, pg.html, true, true) end
    local css = path.read(b.root .. '/scripts/site/style.css', true)
    path.write(out .. '/assets/style.css', css .. result.rank_css, true, true)
    path.write(out .. '/img/placeholder.svg', PLACEHOLDER, true, true)
    path.write(out .. '/index.html', table.concat({
      '<!doctype html><html lang="zh-Hant-TW"><head><meta charset="utf-8">',
      '<meta http-equiv="refresh" content="0; url=zh-TW/index.html"><title>', I.strings('zh_TW').site_title, '</title></head>',
      '<body><a href="zh-TW/index.html">繁體中文</a> · <a href="en/index.html">English</a></body></html>\n' }), true, true)
    json.write(out .. '/catalog.json', catalog, true)
    if opts.icons ~= false then
      local root = (opts.source_root or require('site.source_supplement').DEFAULT_ROOT):gsub('\\', '/')
      local copied, seen = 0, {}
      local host = b.lib('host')
      -- img/ue holds only this build's HUD icons (an id whose art changed or left must not linger)
      local ue_dir = out .. '/img/ue'
      if path.is_directory(ue_dir) then
        for name in require('lfs').dir(ue_dir) do
          if name:match('%.png$') then os.remove(ue_dir .. '/' .. name) end
        end
      end
      local function copy(rel)
        if not rel or seen[rel] then return end
        seen[rel] = true
        if ICON_FILES[rel] then
          -- the HUD's art, at the site's icon size (authored sources are 1254 px)
          path.mkdir_p(path.parent(out .. '/img/' .. rel))
          host.call('png_square_icon', { source = ICON_FILES[rel], output = path.absolute(out .. '/img/' .. rel), edge = 128 })
          copied = copied + 1
          return
        end
        local src = root .. '/' .. rel
        if path.is_file(src) then
          path.write(out .. '/img/' .. rel:gsub('^.-/images/', ''), path.read(src, true), true, true)
          copied = copied + 1
        end
      end
      for _, h in ipairs(catalog.heroes) do copy(h.icon) end
      for _, a in ipairs(catalog.abilities) do copy(a.icon) end
      for _, it in ipairs(catalog.items) do copy(it.icon) end
      result.icons = copied
    end
    result.out = out
  end
  return result
end

local function main(argv)
  local opts = { docs = nil, out = 'target/site', icons = true }
  local i = 1
  while i <= #argv do
    local a = argv[i]
    local function val() i = i + 1; return assert(argv[i], a .. ' needs a value') end
    if a == '--catalog' then opts.catalog_file = val()
    elseif a == '--docs' then opts.docs = opts.docs or {}; opts.docs[#opts.docs + 1] = val()
    elseif a == '--out' then opts.out = val()
    elseif a == '--previous' then opts.previous_file = val()
    elseif a == '--game-catalog' then opts.game_catalog_file = val()
    elseif a == '--no-game-catalog' then opts.game_catalog = false
    elseif a == '--source-root' then opts.source_root = val()
    elseif a == '--no-icons' then opts.icons = false
    elseif a == '--help' or a == '-h' then
      print('Usage: tools\\lua\\lua.exe scripts\\build_site.lua [--catalog FILE] [--docs PATH ...] [--out DIR] [--previous FILE] [--source-root DIR] [--no-icons]')
      return 0
    else error('unknown argument: ' .. a) end
    i = i + 1
  end
  local r = M.build(opts)
  for _, w in ipairs(r.warnings) do io.stderr:write('[site] warning: ', w, '\n') end
  if #r.errors > 0 then
    for _, e in ipairs(r.errors) do io.stderr:write('[site] error: ', e, '\n') end
    io.stderr:write(string.format('[site] %d errors; nothing written\n', #r.errors))
    return 1
  end
  print(string.format('[site] %d pages, %d heroes, %d items, %d icons -> %s', #r.pages, #r.catalog.heroes, #r.catalog.items,
    r.icons or 0, r.out))
  print('[site] content hash ' .. r.catalog.content_hash)
  return 0
end

if arg and arg[0] and arg[0]:match('build_site%.lua$') then os.exit(main(arg)) end
return M
