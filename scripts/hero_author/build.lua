-- `hero_author.lua build` transaction: write generated content, run the
-- existing build chain, and roll every touched file back on any failure.
--
-- deps (injectable for tests):
--   read(p) -> text|nil, write(p, text), exists(p), copy(src, dst), mkdir_p(p),
--   run(exe, args, opts) -> { exit_code, stdout, stderr }, log(line)
local M = {}

-- Build chain after content is written. Each step is a Lua tool under the
-- repo root run with the pinned tools\lua\lua.exe.
function M.steps(root, content_root)
  return {
    { id = "registry", label = "generate base_content registry from templates",
      args = { root .. "/scripts/gen_hero_registry.lua", content_root, "registry" } },
    { id = "ids", label = "generate template ids from templates",
      args = { root .. "/scripts/gen_hero_registry.lua", content_root, "ids" } },
    { id = "build_ue", label = "build base_content, stage DLL, build bridge and OmGame",
      args = { root .. "/scripts/build_ue_moba.lua" } },
    { id = "verify_staged", label = "verify server/runtime/DLL/bridge staged copies are one build",
      args = { root .. "/scripts/build_ue_moba.lua", "--verify-staged-only" } },
    { id = "compiled_contract", label = "check compiled-content-only dependency contract",
      args = { root .. "/scripts/test_compiled_content_contract.lua" } },
  }
end

-- Every file whose previous version must survive a failed build: the
-- content files we write plus built and staged binaries (bindings from
-- scripts/moba_stage_contract.lua), so the last working build stays usable.
function M.protected(root, content_root, bindings)
  local files = {
    content_root .. "/templates/authored_heroes.lua",
    content_root .. "/templates/heroes.lua",
    content_root .. "/templates/abilities.lua",
    -- build_ue appends placeholder portraits; a rolled-back hero must not keep one.
    root .. "/omfue/Content/Omoba/UI/hud-assets-v1.json",
  }
  for _, b in ipairs(bindings or {}) do
    files[#files + 1] = b.built
    for _, c in ipairs(b.copies) do files[#files + 1] = c end
  end
  return files
end

-- plan: { root, content_root, lua, writes = { [path] = text }, protected = {...}, backup_dir, steps }
function M.execute(plan, deps)
  local log = deps.log or function() end
  local snapshot, created = {}, {}
  deps.mkdir_p(plan.backup_dir)
  for i, p in ipairs(plan.protected) do
    if deps.exists(p) then
      local copy = plan.backup_dir .. "/" .. i .. ".bak"
      deps.copy(p, copy)
      snapshot[#snapshot + 1] = { path = p, backup = copy }
    else
      snapshot[#snapshot + 1] = { path = p, absent = true }
    end
  end
  local function rollback(reason)
    log("ROLLBACK: " .. reason)
    for _, s in ipairs(snapshot) do
      if s.absent then
        if deps.exists(s.path) then deps.remove(s.path) end
      else
        deps.copy(s.backup, s.path)
      end
    end
    -- Directories this build created (deepest first) go away with their files.
    for i = #created, 1, -1 do if deps.rmdir then pcall(deps.rmdir, created[i]) end end
    return { ok = false, reason = reason, restored = #snapshot }
  end
  -- New files may live in directories that do not exist yet (an appended
  -- hero's Rust module).
  local function ensure_parent(p)
    local missing, dir = {}, (p:gsub("\\", "/")):match("^(.*)/[^/]+$")
    while dir and dir ~= "" and not dir:match("^%a:$") and not deps.exists(dir) do
      table.insert(missing, 1, dir)
      dir = dir:match("^(.*)/[^/]+$")
    end
    for _, d in ipairs(missing) do deps.mkdir_p(d); created[#created + 1] = d end
  end
  local paths = {}
  for p in pairs(plan.writes) do paths[#paths + 1] = p end
  table.sort(paths)
  for _, p in ipairs(paths) do
    local ok, err = pcall(function() ensure_parent(p); deps.write(p, plan.writes[p]) end)
    if not ok then return rollback("write failed: " .. p .. ": " .. tostring(err)) end
  end
  local done = {}
  for _, step in ipairs(plan.steps) do
    log("STEP " .. step.id .. ": " .. step.label)
    local ok, result = pcall(deps.run, plan.lua, step.args, { cwd = plan.root, check = false })
    if not ok then return rollback("step " .. step.id .. " could not start: " .. tostring(result)) end
    if result.exit_code ~= 0 then
      return rollback("step " .. step.id .. " failed (exit " .. tostring(result.exit_code) .. "): " ..
        ((result.stderr or "") .. (result.stdout or "")):sub(-2000))
    end
    done[#done + 1] = step.id
  end
  return { ok = true, steps = done }
end

return M
