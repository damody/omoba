-- `hero_author.lua shadow`: build a package into a private copy of the
-- crates base_content needs, so its handlers and parity tests compile and run
-- without touching the shared checkout, the shared target directory or the
-- staged DLLs. Real content generation and gen_hero_registry run against the
-- copy; the UE build and staging steps are skipped (nothing is staged).
--
-- sync copies only files the tree does not have yet, so edits made in the
-- tree survive a re-run; start from an empty directory for a clean copy.
local lfs = require("lfs")
local M = {}

-- Crates and the few files outside them that cargo tests include or load at run time
-- (E189: base_content tests run tools/lua on scripts/test_fixtures recipes; a tree without
-- them fails those tests, which then look like flaky bot tests). The v3 conformance tests
-- (E207) run docs/character_pipeline/tools/v3_scenarios.lua, which reads the package v3
-- documents and scripts/_bootstrap.lua.
M.TREES = { "scripts/script-abi", "scripts/base_content", "scripts/lua_data", "scripts/test_fixtures", "omoba-core", "omoba-sim",
  "omoba-template-ids", "omoba-content-model", "specs", "tools/lua", "proto", "omb/src", "omoba-netem-proxy",
  "openspec/changes/simulate-client-rtt-delay/fixtures", "docs/character_pipeline/packages" }
-- Tooling the shadow build runs (not code under test): always refreshed.
M.REFRESH = { "docs/character_pipeline/tools" }
M.FILES = { "scripts/Cargo.toml", "scripts/Cargo.lock", "scripts/gen_hero_registry.lua", "scripts/content_builder.lua",
  "rust-toolchain.toml", "run_2player.bat", "scripts/compare_fog_evidence.lua", "scripts/stop_netem_proxy.lua", "scripts/_bootstrap.lua" }
local SKIP = { target = true, [".git"] = true, assets = true }

local function mkdir_p(p)
  p = p:gsub("\\", "/")
  local acc = p:match("^%a:/") or (p:sub(1, 1) == "/" and "/" or "")
  for seg in p:sub(#acc + 1):gmatch("[^/]+") do acc = acc .. seg; lfs.mkdir(acc); acc = acc .. "/" end
end

local function copy_file(s, d, counter, overwrite)
  if lfs.attributes(d) and not overwrite then return end
  local f = assert(io.open(s, "rb")); local t = f:read("a"); f:close()
  mkdir_p(d:match("^(.*)/"))
  local g = assert(io.open(d, "wb")); g:write(t); g:close()
  counter.n = counter.n + 1
end

local function copy_tree(s, d, counter, overwrite)
  if lfs.attributes(s, "mode") ~= "directory" then return end
  mkdir_p(d)
  for name in lfs.dir(s) do
    if name ~= "." and name ~= ".." and not SKIP[name] then
      local sp, dp = s .. "/" .. name, d .. "/" .. name
      if lfs.attributes(sp, "mode") == "directory" then copy_tree(sp, dp, counter, overwrite) else copy_file(sp, dp, counter, overwrite) end
    end
  end
end

-- Returns the number of files copied into `tree`.
function M.sync(repo, tree)
  local counter = { n = 0 }
  for _, d in ipairs(M.TREES) do copy_tree(repo .. "/" .. d, tree .. "/" .. d, counter) end
  for _, d in ipairs(M.REFRESH) do copy_tree(repo .. "/" .. d, tree .. "/" .. d, counter, true) end
  for _, f in ipairs(M.FILES) do
    if lfs.attributes(repo .. "/" .. f) then copy_file(repo .. "/" .. f, tree .. "/" .. f, counter) end
  end
  return counter.n
end

-- Deps for cli.build that write into `tree` and only run gen_hero_registry.
function M.deps(cli, tree, lua, process, log)
  local deps = cli.default_deps(tree, lua)
  deps.bindings = function() return {} end
  deps.run = function(exe, args, opts)
    if args[1]:find("gen_hero_registry", 1, true) then return process.run(exe, args, opts) end
    log("  (shadow) skipped " .. args[1]:match("[^/\\]+$") .. " " .. (args[2] or ""))
    return { exit_code = 0, stdout = "", stderr = "" }
  end
  return deps
end

return M
