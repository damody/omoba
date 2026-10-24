-- Probe external tools before code generation, compilation or staging writes.
local M = {}
function M.require_ready(process)
  local repair = 'Install cbindgen with: cargo install cbindgen --version 0.29.4 --locked; then retry in a terminal with cbindgen on PATH'
  local ok, result = pcall(process.run, 'cbindgen', {'--version'}, {check=false})
  assert(ok and type(result)=='table' and result.exit_code==0,
    'Unreal bridge prerequisite cbindgen is unavailable. '..repair)
  local version = type(result.stdout)=='string' and result.stdout:match('^cbindgen (%d+%.%d+%.%d+)%s*$')
  assert(version, 'Unreal bridge prerequisite returned an invalid cbindgen version. '..repair)
  return version
end
return M
