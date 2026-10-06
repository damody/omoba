-- Headless pre-match room only. No Unreal, gameplay World or automatic locking.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}
function M.options(args)
  local allowed={['--recipe']=true,['--profile']=true,['--output']=true,
    ['--selection-bind']=true,['--selection-timeout-seconds']=true,['--selection-cancel-file']=true}
  local forwarded={'--interactive-selection'}
  local seen={};local i=1
  while i<=#args do
    local key=args[i]
    assert(allowed[key],'unknown headless selection option: '..tostring(key))
    assert(not seen[key],'duplicate headless selection option: '..key)
    seen[key]=true;i=i+1
    local value=assert(args[i],key..' requires a value')
    forwarded[#forwarded+1]=key;forwarded[#forwarded+1]=value
    i=i+1
  end
  local options=require('moba_role_launch').options(forwarded)
  assert(options.output,'headless selection requires a new --output directory')
  assert(options.selection_bind,'headless selection requires an explicit --selection-bind IPv4')
  return options
end
function M.run(options,process,time)
  local output=path.absolute(assert(options.output),b.root)
  assert(not path.exists(output),'headless selection output must be a new directory')
  require('moba_network_launch').ipv4(assert(options.selection_bind,'headless selection requires an explicit bind'))
  assert(not options.connect and not options.server_only and not options.finish_timeout_seconds
    and not options.selection_smoke_hero and next(options.local_players or {})==nil,
    'headless selection cannot launch clients, gameplay or local renderers')
  local preparation=require('moba_selection_prepare')
  local ok,result=xpcall(function()
    local inspected=preparation.inspect(options)
    local candidate,preflight=preparation.prepare(options,process,output,inspected)
    require('moba_shared_selection').run_headless(options,process,time,output,candidate,
      inspected.exe,inspected.humans,preflight)
    local selected=assert(options.recipe,'headless room did not publish a final recipe')
    assert(path.is_file(selected),'headless room final recipe missing')
    path.write(path.join(output,'result.json'),json.encode({schema_version=1,
      scope='selection-host-only',tick_rate_hz=60,human_count=#inspected.humans,recipe_json=selected}))
    print('Headless selection completed; gameplay has not started. Recipe: '..selected)
    return selected
  end,debug.traceback)
  if not ok then
    path.append(path.join(output,'errors.md'),'\n# Headless selection error\n\n'..tostring(result)..
      '\n\nDecision: no gameplay launch, automatic locking, frontend dependency or profile fallback.\n')
    error(result,0)
  end
  return result
end
return M
