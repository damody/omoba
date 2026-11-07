-- Checks that every history_nobu hero and item compiles through v3 -> IR -> content-model:
-- scripts/gen_v3_content.lua writes target/v3-content/v3-content.json, then
-- omoba-content-model/tests/v3_full_content.rs loads every block as V3Block and validates it.
--
--   tools\lua\lua.exe scripts\check_v3_content.lua [--target-dir DIR]
--
-- --target-dir: CARGO_TARGET_DIR for the Rust test (default: the crate's own target).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local process = b.lib("process")

local target_dir
local i = 1
while i <= #arg do
  if arg[i] == "--target-dir" then target_dir = arg[i + 1]; i = i + 1 else error("unknown argument " .. arg[i]) end
  i = i + 1
end

local lua = b.lib("platform").lua_executable
local gen = process.run(lua, { scripts_dir .. "/gen_v3_content.lua" }, { cwd = b.root, check = false })
io.stdout:write(gen.stdout or "")
if gen.exit_code ~= 0 then io.stderr:write(gen.stderr or ""); os.exit(1) end

local env = { OM_V3_CONTENT = b.root .. "/target/v3-content/v3-content.json" }
if target_dir then env.CARGO_TARGET_DIR = target_dir end
local test = process.run("cargo", { "test", "--manifest-path", b.root .. "/omoba-content-model/Cargo.toml",
  "--test", "v3_full_content", "--", "--nocapture" }, { cwd = b.root, check = false, env = env })
local out = (test.stdout or "") .. (test.stderr or "")
for line in out:gmatch("[^\r\n]+") do
  if line:match("^v3 content:") or line:match("test result:") or line:match("panicked") or line:match("^error") then print(line) end
end
if test.exit_code ~= 0 then io.stderr:write(out:sub(-3000)); os.exit(1) end
print("OK: every v3 hero and item loads as content-model V3Block")
