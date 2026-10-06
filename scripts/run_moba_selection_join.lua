local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_selection_join.lua --invite PRIVATE_PLAYER.json --player ID --output NEW_DIRECTORY [--profile release|debug] [--ue-root PATH] [--graphics d3d11|d3d12] [--selection-timeout-seconds 1..7200]')
  print('Requires a prebuilt profile-matched moba-config and Unreal. The invitation is private to this seat; do not publish it. Selection only; no server/client runtime/gameplay is started. Cancellation never auto-locks or stops the remote host.')
  print('After finalized selection, the host-authored recipe is compiled-validated and saved as OUTPUT/match-plan.json. Use it with run_moba_role_ue.lua --connect HOST_IPV4 --local-player ID --recipe FILE.json. No second recipe transfer is needed; no automatic gameplay launch.')
  print('Optional --selection-cancel-file PATH: create it to cancel and retire only the original local selection renderer, never the remote host. The signal is not read/deleted; forced-termination supervision is separate.')
  return
end
local b=require('_bootstrap')
local join=require('moba_selection_join')
local options=join.options(arg)
local path=b.lib('path')
local output=path.absolute(options.output,b.root)
local owns_output=not path.exists(output)
local ok,err=xpcall(function() join.run(options,b.lib('process'),b.lib('time')) end,debug.traceback)
if not ok then
  if owns_output then
    path.append(path.join(output,'errors.md'),'\n# Selection join entry failure\n\n'..tostring(err)..
      '\n\nDecision: no gameplay launch, automatic locking, profile fallback or remote-host cleanup. Retain this session diagnostics only.\n')
  end
  error(err,0)
end
