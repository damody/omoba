-- Full v3 content (M6 prerequisite, history-nobu full port): every history_nobu hero and shop item
-- compiled from its v3 document (docs/character_pipeline/tools/v3_sources.lua) into content-model
-- blocks with behavior IR, whether or not the game ships it. Heroes without a v2 package are
-- `listed = false` (unlisted): compiled and validated, never in hero select or the game roster. The
-- game's own content (authored_heroes.lua, V3_CATALOG_JSON) is unchanged; this file is checked by
-- omoba-content-model/tests/v3_full_content.rs (scripts/check_v3_content.lua runs both).
--
--   tools\lua\lua.exe scripts\gen_v3_content.lua [--root DIR] [--out FILE]
--
-- Output (JSON, default target/v3-content/v3-content.json):
--   { schema, kind = "full", source = { repository, commit }, content_hash,
--     heroes = [ { source_id, package, listed, game_id?, abilities = [ids] } ],
--     abilities = { [id] = block }, items = { [id] = block } }
-- block = the V3Block of V3_CATALOG_JSON plus `ir`. Entries whose behavior still runs in a Rust
-- handler (coverage = rust_handler) get the IR of their data behavior too, so every document is
-- known to compile; `coverage` says which one runs.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local json = b.lib("json")
local sha256 = require("hero_source.sha256")

local M = {}
M.OUT = "target/v3-content/v3-content.json"

local function tools(root)
  local t = root .. "/docs/character_pipeline/tools"
  PACKAGE_V3_AS_MODULE = true
  local v = dofile(t .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  return v, dofile(t .. "/v3_compile.lua"), dofile(t .. "/v3_ir.lua"), dofile(t .. "/v3_sources.lua")
end

local function copy(x)
  if type(x) ~= "table" then return x end
  local t = {}
  for k, v in pairs(x) do t[k] = copy(v) end
  return t
end

-- package name -> game hero id, for packages the game ships (they carry character.lua)
local function shipped(root, packages)
  local map = {}
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  for name in lfs.dir(root .. "/" .. packages) do
    local p = root .. "/" .. packages .. "/" .. name .. "/character.lua"
    local chunk = lfs.attributes(p) and loadfile(p, "t", env)
    if chunk then
      local ok, v = pcall(chunk)
      if ok and type(v) == "function" then ok, v = pcall(v, {}) end
      if ok and type(v) == "table" and v.hero and v.hero.id then map[name] = v.hero.id end
    end
  end
  return map
end

-- root: repo root (default this checkout) -> content, problems
function M.build(root)
  root = root or b.root
  local validator, compiler, ir_compiler, sources = tools(root)
  local problems = {}
  local names = compiler.item_names(root .. "/" .. sources.ITEMS)
  local ships = shipped(root, sources.PACKAGES)
  local content = { schema = compiler.SCHEMA, kind = "full", heroes = {}, abilities = {}, items = {} }
  local function compile(file)
    local doc, err = validator.load_file(file)
    if not doc then problems[#problems + 1] = file .. ": " .. tostring(err); return end
    local errs = validator.validate_doc(doc)
    if #errs > 0 then problems[#problems + 1] = file .. ": " .. errs[1]; return end
    content.source = content.source or { repository = doc.source.repository, commit = doc.source.commit }
    local ok, compiled = pcall(compiler.compile, doc, { names = names, ir = false })
    if not ok then problems[#problems + 1] = file .. ": " .. tostring(compiled); return end
    -- M3' ④: no behavior IR in the content (generated Rust runs the behavior, v3_rust.lua).
    for id, block in pairs(compiled.abilities) do content.abilities[id] = block end
    for id, block in pairs(compiled.items) do content.items[id] = block end
    return doc
  end
  for _, h in ipairs(sources.heroes(root)) do
    local doc = compile(h.path)
    if doc then
      local row = { source_id = h.source_id, package = h.package, listed = ships[h.package] ~= nil, game_id = ships[h.package], abilities = {} }
      for _, a in ipairs(doc.abilities or {}) do row.abilities[#row.abilities + 1] = a.id end
      content.heroes[#content.heroes + 1] = row
    end
  end
  for _, f in ipairs(sources.items(root)) do compile(f) end
  content.content_hash = sha256(json.encode(content)):sub(1, 16)
  return content, problems
end

function M.main(argv)
  local opts = {}
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--root" or a == "--out" then opts[a:sub(3)] = argv[i + 1]; i = i + 1
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local content, problems = M.build(opts.root)
  if #problems > 0 then
    for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
    io.stderr:write(#problems .. " problem(s)\n")
    return 1
  end
  local out = opts.out or (b.root .. "/" .. M.OUT)
  local dir = out:gsub("\\", "/"):match("^(.*)/")
  if dir then b.lib("path").mkdir_p(dir) end
  local f = assert(io.open(out, "wb")); f:write(json.encode(content)); f:close()
  local listed, na, ni = 0, 0, 0
  for _, h in ipairs(content.heroes) do if h.listed then listed = listed + 1 end end
  for _ in pairs(content.abilities) do na = na + 1 end
  for _ in pairs(content.items) do ni = ni + 1 end
  print(("wrote %s: %d heroes (%d listed, %d unlisted), %d abilities, %d items, hash %s"):format(
    out, #content.heroes, listed, #content.heroes - listed, na, ni, content.content_hash))
  return 0
end

if rawget(_G, "GEN_V3_CONTENT_AS_MODULE") then return M end
os.exit(M.main(arg))
