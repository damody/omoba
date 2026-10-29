-- Author tool: index / validate / build (plan E2). See scripts/hero_author.lua.
local generate = require("hero_author.generate")
local install = require("hero_author.install")
local build = require("hero_author.build")
local rust_install = require("hero_author.rust_install")
local v3 = require("hero_author.v3")
local lfs = require("lfs")

local M = {}

local function read(p)
  local f = io.open(p, "rb"); if not f then return nil end
  local t = f:read("a"); f:close(); return t
end
local function write(p, text)
  local f = assert(io.open(p, "wb")); assert(f:write(text)); assert(f:close())
end
local function mkdir_p(p)
  p = p:gsub("\\", "/")
  local acc = p:match("^%a:/") or (p:sub(1, 1) == "/" and "/" or "")
  for seg in p:sub(#acc + 1):gmatch("[^/]+") do
    acc = acc .. seg
    if not lfs.attributes(acc) then assert(lfs.mkdir(acc), "mkdir failed: " .. acc) end
    acc = acc .. "/"
  end
end
local function exists(p) return lfs.attributes(p) ~= nil end
local function copy(src, dst) write(dst, assert(read(src), "copy source missing: " .. src)) end
-- Package and index files are data: load them with only pure helpers
-- (no io/os/require/load), so a package cannot run side effects.
local function safe_env()
  return { pairs = pairs, ipairs = ipairs, next = next, select = select, type = type, tostring = tostring,
    tonumber = tonumber, assert = assert, error = error, string = string, table = table, math = math, utf8 = utf8 }
end
M.safe_env = safe_env
local function sandbox(file)
  local chunk = assert(loadfile(file, "t", safe_env()))
  local v = chunk()
  if type(v) == "function" then v = v({}) end
  return v
end

-- Default dependencies; tests override root, runner and bindings.
function M.default_deps(root, lua)
  local process = require("tools.lua.lib.process")
  return {
    root = root, lua = lua,
    read = read, write = write, exists = exists, copy = copy, mkdir_p = mkdir_p, rmdir = lfs.rmdir, remove = os.remove,
    run = function(exe, args, opts) return process.run(exe, args, opts) end,
    bindings = function(r) return require("moba_stage_contract").bindings(r) end,
    out = function(s) io.stdout:write(s, "\n") end,
    err = function(s) io.stderr:write(s, "\n") end,
  }
end

local function parse(argv)
  local o = { positional = {} }
  local i = 1
  while i <= #argv do
    local v = argv[i]
    if v:sub(1, 2) == "--" then
      local name = v:sub(3)
      local nx = argv[i + 1]
      if nx == nil or nx:sub(1, 2) == "--" then o[name] = true else o[name] = nx; i = i + 1 end
    else
      o.positional[#o.positional + 1] = v
    end
    i = i + 1
  end
  return o
end

-- root: the repo root, for the v3 validator and compiler a v3.lua needs.
local function load_package(dir, root)
  local pkg = {}
  for _, name in ipairs({ "character", "source", "abilities", "capabilities", "presentation" }) do
    pkg[name] = sandbox(dir .. "/" .. name .. ".lua")
  end
  if exists(dir .. "/summons.lua") then pkg.summons = sandbox(dir .. "/summons.lua") end
  -- Schema v3 document (the package's authoring source): values and text for every ability;
  -- see hero_author/v3.lua and docs/character_pipeline/tools/v3_sources.lua.
  -- kit.lua was the M1 name; a leftover one would be ignored and the build would lose every v3 block
  assert(not exists(dir .. "/kit.lua"), dir .. "/kit.lua: the package's v3 document is v3.lua (merge it, then remove kit.lua)")
  if exists(dir .. "/v3.lua") then
    pkg.kit_compiled = v3.compile(root, sandbox(dir .. "/v3.lua"))
    pkg.rust_texts = {}
    if lfs.attributes(dir .. "/rust", "mode") == "directory" then
      for name in lfs.dir(dir .. "/rust") do
        if name:match("%.rs$") then pkg.rust_texts["rust/" .. name] = read(dir .. "/rust/" .. name) end
      end
    end
  end
  return pkg
end

local function copy_lua_tree(src, dst)
  mkdir_p(dst)
  for name in lfs.dir(src) do
    if name ~= "." and name ~= ".." then
      local s, d = src .. "/" .. name, dst .. "/" .. name
      local mode = lfs.attributes(s, "mode")
      if mode == "directory" then copy_lua_tree(s, d)
      elseif name:match("%.lua$") then copy(s, d) end
    end
  end
end

local function remove_tree(target, guard)
  assert(target:find(guard, 1, true), "unsafe cleanup target " .. target)
  if lfs.attributes(target, "mode") == "directory" then
    for name in lfs.dir(target) do
      if name ~= "." and name ~= ".." then remove_tree(target .. "/" .. name, guard) end
    end
    lfs.rmdir(target)
  elseif exists(target) then
    os.remove(target)
  end
end

-- `rust` is the package's rust_install plan (or nil): files it installs count
-- as present, so a package that carries its handlers is complete.
local function missing_handlers(root, pkg, ids, rust)
  local out = {}
  local module = pkg.character.identity.rust_module
  local module_mod = root .. "/scripts/base_content/src/heroes/" .. tostring(module) .. "/mod.rs"
  local heroes_mod = root .. "/scripts/base_content/src/heroes/mod.rs"
  local mod_rs = (rust and rust.writes[module_mod]) or read(module_mod) or ""
  local heroes_rs = (rust and rust.writes[heroes_mod]) or read(heroes_mod) or ""
  if module and not heroes_rs:find("mod " .. module .. ";", 1, true) then
    out[#out + 1] = "module " .. module .. " is not declared in scripts/base_content/src/heroes/mod.rs"
  end
  for _, ab in ipairs(pkg.abilities.abilities) do
    if (ids == nil or ids[ab.id]) and (ab.implementation or {}).kind == "rust_handler" then
      if not mod_rs:find(ab.id .. "_ffi", 1, true) then
        out[#out + 1] = ab.id .. "_ffi not exported from scripts/base_content/src/heroes/" .. tostring(module) .. "/mod.rs"
      end
    end
  end
  return out
end

-- Generates the authored file and hooks against a content root; returns
-- { writes = { [path] = text }, authored = data } without touching disk.
local function compose(content_root, entries)
  local authored_path = content_root .. "/" .. generate.AUTHORED_FILE
  local previous_text = read(authored_path)
  local previous = previous_text and generate.load(previous_text) or nil
  local data = generate.merge(previous, entries)
  local writes = { [authored_path] = generate.render(data) }
  for relative in pairs(install.HOOKS) do
    local p = content_root .. "/" .. relative
    local text = assert(read(p), "missing template " .. p)
    local patched = install.patch(relative, text)
    if patched then writes[p] = patched end
  end
  return writes, data
end

-- validate: schema + catalog/D3 cross-checks + generation dry run.
function M.validate(dir, opts, deps)
  local root = deps.root
  local problems, notes = {}, {}
  VALIDATE_PACKAGE_AS_MODULE = true
  local validator = dofile(root .. "/docs/character_pipeline/tools/validate_package.lua")
  VALIDATE_PACKAGE_AS_MODULE = nil
  local index = root .. "/docs/plans/2026-10-24-omfue-month-mvp/source-index/"
  local vopts = {}
  local catalog_file = opts.catalog or index .. "source-catalog.lua"
  local req_file = opts.requirements or index .. "candidate-requirements.lua"
  if exists(catalog_file) then vopts.catalog = sandbox(catalog_file) else notes[#notes + 1] = "no source catalog; provenance not cross-checked" end
  if exists(req_file) then vopts.requirements = sandbox(req_file) else notes[#notes + 1] = "no D3 requirements; capability statuses not cross-checked" end
  local errs, warns = validator.validate(dir, vopts)
  for _, e in ipairs(errs) do problems[#problems + 1] = e end
  for _, w in ipairs(warns) do notes[#notes + 1] = w end
  if #problems > 0 then return { ok = false, problems = problems, notes = notes } end

  local pkg = load_package(dir, root)
  local unsupported = {}
  for _, ab in ipairs(pkg.abilities.abilities) do
    if ab.status ~= "supported" then unsupported[#unsupported + 1] = ab.id .. " (" .. ab.status .. ")" end
  end
  local rust = rust_install.plan(root, dir, pkg)
  if rust then notes[#notes + 1] = "package installs " .. #rust.files .. " Rust files into scripts/base_content at build (full kit)" end
  for _, m in ipairs(missing_handlers(root, pkg, nil, rust)) do notes[#notes + 1] = "missing handler: " .. m end

  -- Generation dry run on a scratch copy of the template Lua files.
  local content_root = opts.content_root or (root .. "/scripts/lua_data")
  local scratch = root .. "/target/hero-author-validate/" .. os.time() .. "-" .. math.random(1, 1e9)
  copy_lua_tree(content_root .. "/templates", scratch .. "/templates")
  local ok, err = pcall(function()
    local entries = generate.package_entries(pkg, nil)
    local writes = compose(scratch, entries)
    for p, text in pairs(writes) do write(p, text) end
    local builder = require("content_builder").new(scratch)
    local heroes = builder.include("templates/heroes.lua")
    local target
    for _, h in ipairs(heroes) do if h.id == pkg.character.hero.id then target = h end end
    if pkg.character.identity.integration == "update_existing" then
      assert(target, "update_existing target hero " .. pkg.character.hero.id .. " not found in templates")
      assert(target.rust_module == pkg.character.identity.rust_module, "rust_module differs from existing hero")
    else
      assert(target and target.authored_package == pkg.character.hero.id, "appended hero did not appear in templates")
    end
    -- Same MOBA birth-rank rule the Rust content model enforces at build time.
    local by_id = {}
    for _, ability in ipairs(builder.include("templates/abilities.lua")) do by_id[ability.id] = ability end
    local loadout_error = generate.check_moba_loadout(target, by_id)
    assert(not loadout_error, loadout_error)
    for _, mode in ipairs({ "registry", "ids" }) do
      local r = deps.run(deps.lua, { root .. "/scripts/gen_hero_registry.lua", scratch, mode }, { cwd = root, check = false })
      assert(r.exit_code == 0, "gen_hero_registry " .. mode .. " rejected generated content: " .. (r.stderr or "") .. (r.stdout or ""))
      if mode == "ids" then
        for _, ab in ipairs(entries.abilities) do
          assert(r.stdout:find('"' .. ab.id .. '"', 1, true), "generated ability " .. ab.id .. " missing from template ids")
        end
      end
    end
  end)
  remove_tree(scratch, "hero-author-validate")
  if not ok then problems[#problems + 1] = "GenerationError: " .. tostring(err) end
  return { ok = #problems == 0, problems = problems, notes = notes, unsupported = unsupported,
    kit_status = pkg.capabilities.kit_status, package = pkg }
end

function M.build(dir, opts, deps)
  local root = deps.root
  local v = M.validate(dir, opts, deps)
  if not v.ok then return { ok = false, stage = "validate", problems = v.problems } end
  local pkg = v.package
  local slice
  if type(opts.slice) == "string" then
    slice = {}
    local by = {}
    for _, ab in ipairs(pkg.abilities.abilities) do by[ab.id] = ab end
    for id in opts.slice:gmatch("[^,]+") do
      if not by[id] then return { ok = false, stage = "slice", problems = { "unknown ability in --slice: " .. id } } end
      if by[id].status ~= "supported" then
        return { ok = false, stage = "slice", problems = { id .. " is " .. by[id].status .. "; only supported abilities can be built" } }
      end
      slice[id] = true
    end
  elseif v.kit_status ~= "supported" then
    return { ok = false, stage = "gate", problems = { "kit is " .. tostring(v.kit_status) ..
      "; build a thin slice with --slice <supported ability ids> (unsupported: " .. table.concat(v.unsupported, ", ") .. ")" } }
  end
  local rust = rust_install.plan(root, dir, pkg, slice)
  local missing = missing_handlers(root, pkg, slice, rust)
  if #missing > 0 then return { ok = false, stage = "handlers", problems = missing } end
  local content_root = opts.content_root or (root .. "/scripts/lua_data")
  local entries = generate.package_entries(pkg, slice)
  local writes = compose(content_root, entries)
  local protected = build.protected(root, content_root, deps.bindings(root))
  for _, p in ipairs(rust and rust.files or {}) do writes[p] = rust.writes[p] end
  -- Every written file is rolled back on failure, whatever produced it.
  local listed = {}
  for _, p in ipairs(protected) do listed[p] = true end
  local written = {}
  for p in pairs(writes) do written[#written + 1] = p end
  table.sort(written)
  for _, p in ipairs(written) do
    if not listed[p] then protected[#protected + 1] = p; listed[p] = true end
  end
  local plan = {
    root = root, lua = deps.lua, content_root = content_root, writes = writes,
    protected = protected,
    backup_dir = root .. "/target/hero-author-backup/" .. os.time() .. "-" .. math.random(1, 1e9),
    steps = build.steps(root, content_root),
    skipped = entries.skipped,
  }
  if not opts.execute then return { ok = true, dry_run = true, plan = plan } end
  local result = build.execute(plan, deps)
  result.plan = plan
  if not result.ok then result.stage = "execute"; result.problems = { result.reason } end
  return result
end

function M.run(argv, deps)
  local o = parse(argv)
  local command = o.positional[1]
  local function fail(lines) for _, l in ipairs(lines) do deps.err(l) end return 1 end
  if command == "index" then
    local args = { deps.root .. "/scripts/hero_source_index.lua" }
    for i = 1, #argv do args[#args + 1] = argv[i] end
    local r = deps.run(deps.lua, args, { cwd = deps.root, check = false })
    if r.stdout and r.stdout ~= "" then deps.out((r.stdout:gsub("%s+$", ""))) end
    if r.stderr and r.stderr ~= "" then deps.err((r.stderr:gsub("%s+$", ""))) end
    return r.exit_code
  elseif command == "validate" or command == "build" then
    local dir = o.positional[2]
    if not dir then return fail({ "usage: hero_author.lua " .. command .. " <package_dir> [options]" }) end
    local opts = { catalog = o.catalog ~= true and o.catalog or nil, requirements = o.requirements ~= true and o.requirements or nil,
      content_root = o["content-root"] ~= true and o["content-root"] or nil, slice = o.slice, execute = o.execute == true }
    local result = command == "validate" and M.validate(dir, opts, deps) or M.build(dir, opts, deps)
    for _, n in ipairs(result.notes or {}) do deps.err(n) end
    if not result.ok then return fail(result.problems or { "failed" }) end
    if command == "validate" then
      deps.out("OK: package valid; kit_status " .. tostring(result.kit_status) ..
        (#result.unsupported > 0 and ("; not yet supported: " .. table.concat(result.unsupported, ", ")) or ""))
    elseif result.dry_run then
      deps.out("DRY RUN (add --execute to apply). Writes:")
      local paths = {}
      for p in pairs(result.plan.writes) do paths[#paths + 1] = p end
      table.sort(paths)
      for _, p in ipairs(paths) do deps.out("  write " .. p) end
      for _, s in ipairs(result.plan.steps) do deps.out("  step  " .. s.id .. ": " .. table.concat(s.args, " ")) end
      deps.out("  rollback protects " .. #result.plan.protected .. " files")
      if #result.plan.skipped > 0 then deps.out("  not built (outside slice): " .. table.concat(result.plan.skipped, ", ")) end
    else
      deps.out("OK: build complete (" .. table.concat(result.steps, ", ") .. ")")
    end
    return 0
  end
  if command == "shadow" then
    -- Private test tree: sync the crates, then build the package into it
    -- with only gen_hero_registry (no UE build, nothing staged).
    local dir, tree = o.positional[2], o.tree ~= true and o.tree or nil
    if not dir or not tree then return fail({ "usage: hero_author.lua shadow <package_dir> --tree <dir> [--slice id,id]" }) end
    tree = tree:gsub("\\", "/")
    assert(not tree:lower():find(deps.root:lower() .. "/scripts", 1, true) and tree:lower() ~= deps.root:lower(),
      "--tree must be a separate directory, not the shared checkout")
    local shadow = require("hero_author.shadow")
    deps.out("synced " .. shadow.sync(deps.root, tree) .. " new files into " .. tree)
    local index = deps.root .. "/docs/plans/2026-10-24-omfue-month-mvp/source-index/"
    local sdeps = shadow.deps(M, tree, deps.lua, require("tools.lua.lib.process"), deps.out)
    sdeps.out, sdeps.err = deps.out, deps.err
    local result = M.build(dir, { catalog = index .. "source-catalog.lua", requirements = index .. "candidate-requirements.lua",
      slice = o.slice, execute = true }, sdeps)
    for _, n in ipairs(result.notes or {}) do deps.err(n) end
    if not result.ok then return fail(result.problems or { "failed" }) end
    deps.out("OK: shadow build complete. Test with your own target dir, e.g.")
    deps.out("  set CARGO_TARGET_DIR=" .. tree .. "/../cargo && cargo test --manifest-path " .. tree .. "/scripts/Cargo.toml -p base_content --lib -- heroes::")
    return 0
  end
  return fail({ "usage: hero_author.lua index|validate|build|shadow ..." })
end

return M
