local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_selection_host.lua --selection-bind HOST_IPV4 --output NEW_DIRECTORY [--recipe AUTHORING.lua|PLAN.json] [--profile release|debug] [--selection-timeout-seconds 1..7200]')
  print('Headless selection only: no Unreal, gameplay server/client, implicit build or automatic locking. Share only each remote human\'s private invitation in OUTPUT/host. All humans must lock and finalize through the existing selection client.')
  print('The final host-authored recipe is saved as OUTPUT/match-plan.json. Start gameplay explicitly with run_moba_role_ue.lua --server-only --server-bind HOST_IPV4 --recipe FILE.json --output NEW_GAME_DIRECTORY. Remote players use their saved recipe with --connect.')
  print('Optional --selection-cancel-file PATH: create it to cancel without automatic locking or gameplay. It is not read/deleted and does not supervise forced termination.')
  return
end
local b=require('_bootstrap')
local host=require('moba_selection_host')
local options=host.options(arg)
host.run(options,b.lib('process'),b.lib('time'))
