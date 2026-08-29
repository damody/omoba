local script = debug.getinfo(1, "S").source:sub(2)
local root = script:gsub("[/\\]docs[/\\]character_pipeline[/\\]tools[/\\]bootstrap.lua$", "")
package.path = root .. "/?.lua;" .. package.path

local args = require("tools.lua.lib.args").parse(arg)
local path = require("tools.lua.lib.path")
local process = require("tools.lua.lib.process")
local json = require("tools.lua.lib.json")
local platform = require("tools.lua.lib.platform")

assert((args.prepare and not args.diagnose) or (args.diagnose and not args.prepare),
  "choose exactly one of --prepare or --diagnose")

local home = os.getenv(platform.is_windows and "USERPROFILE" or "HOME") or ""
local function expand(value) return tostring(value):gsub("^~", home) end
local shared = expand(args["shared-root"] or "/media/damody/新增磁碟區/AI_Pic")
local vroot = expand(args["linux-venv-root"] or "~/.cache/omoba-character-pipeline/venvs")
local name = args["venv-name"] or "character-pipeline"
local cache = expand(args["cache-root"] or "~/.cache/omoba-character-pipeline")
local venv = path.join(vroot, name)
local python = path.join(venv, platform.is_windows and "Scripts/python.exe" or "bin/python")
local checks = {}

local function item(check_name, status, kind, message, extra)
  local value = extra or {}
  value.name, value.status, value.kind, value.message = check_name, status, kind, message
  table.insert(checks, value)
  return value
end

local function command(check_name, exe, argv, options)
  options = options or {}
  local ok, result = pcall(process.run, exe, argv, { cwd = options.cwd, check = false })
  if not ok then
    return item(check_name, "warn", "Executable", tostring(result),
      { executable = exe, exit_code = json.null })
  end
  local output = (result.stdout .. result.stderr):gsub("%s+$", "")
  return item(check_name, result.exit_code == 0 and "ok" or "warn", "Executable",
    output ~= "" and output or (result.exit_code == 0 and "completed" or "command failed"),
    { executable = exe, exit_code = result.exit_code })
end

local function first_existing(candidates)
  for _, candidate in ipairs(candidates) do if path.is_file(candidate) then return candidate end end
end

if args.prepare then
  assert(not path.absolute(venv):lower():find(path.absolute(shared):lower(), 1, true),
    "Linux venv must not be inside shared AI root")
  path.mkdir_p(vroot)
  if not path.is_file(python) then
    local result = process.run(platform.is_windows and "py" or "python3",
      { "-m", "venv", venv }, { check = false })
    assert(result.exit_code == 0, "venv creation failed: " .. result.stderr)
  end
  command("venv_python", python, { "--version" })
  print(json.encode({ schema_version = 1, mode = "prepare", shared_root = shared,
    linux_venv_root = vroot, venv_path = venv, venv_python = python, checks = checks }))
  return
end

command("gpu", platform.is_windows and "nvidia-smi.exe" or "nvidia-smi",
  { "--query-gpu=name,driver_version", "--format=csv,noheader" })
command("python", python, { "--version" })
if path.is_file(python) then
  command("torch_cuda", python, { "-c",
    "import torch; print(torch.__version__, torch.cuda.is_available()); raise SystemExit(0 if torch.cuda.is_available() else 2)" })
else
  item("torch_cuda", "warn", "PythonPackage", "venv Python missing", { path = python })
end

local comfy = path.join(shared, "ComfyUI", "ComfyUI")
local webui = path.join(shared, "stable-diffusion-webui")
for _, entry in ipairs({ { "shared_root", shared }, { "comfyui_root", comfy },
  { "stable_diffusion_webui_root", webui } }) do
  local exists = path.is_directory(entry[2])
  item(entry[1], exists and "ok" or "warn", "Path",
    exists and "path available" or "path missing", { path = entry[2] })
end

if path.is_file(python) and path.is_file(path.join(comfy, "folder_paths.py")) then
  command("comfyui_smoke", python,
    { "-c", "import folder_paths; print('comfyui import smoke ok')" }, { cwd = comfy })
else
  item("comfyui_smoke", "warn", "Smoke",
    "ComfyUI root or selected venv Python missing; smoke not run",
    { executable = python, path = comfy, exit_code = json.null })
end

local blender = args.blender or os.getenv("BLENDER_EXE") or first_existing({
  path.join(cache, "blender", "blender.exe"), path.join(cache, "blender", "blender"),
  path.join(cache, "bin", "blender.exe"), path.join(cache, "bin", "blender")
}) or (platform.is_windows and "blender.exe" or "blender")
command("blender_background_smoke", blender,
  { "--background", "--factory-startup", "--python-expr",
    "print('blender background smoke ok')" })

for _, marker in ipairs({
  path.join(shared, "stable-diffusion-webui", "venv", "Scripts", "python.exe"),
  path.join(shared, "ComfyUI", "python_embeded", "python.exe")
}) do
  item("windows_runtime_marker", path.exists(marker) and "ok" or "warn", "Safety",
    "read-only marker; never executed", { path = marker })
end

local errors, warnings = 0, 0
for _, check in ipairs(checks) do
  if check.status == "error" then errors = errors + 1
  elseif check.status == "warn" then warnings = warnings + 1 end
end
print(json.encode({ schema_version = 1, mode = "diagnose", shared_root = shared,
  linux_venv_root = vroot, venv_path = venv, venv_python = python, cache_root = cache,
  checks = checks, summary = { errors = errors, warnings = warnings } }))
