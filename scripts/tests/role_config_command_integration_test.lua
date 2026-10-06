local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process=b.lib('path'),b.lib('process')
local launch=require('moba_role_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target/compiled-config-tests'))
local root
for i=1,1000 do local value=path.join(parent,os.time()..'-'..i);if lfs.mkdir(value) then root=value;break end end
assert(root)
for _,profile in ipairs({'debug','release'}) do
  local options=launch.options({'--profile',profile,'--server-only','--prepare-only','--no-build',
    '--output',path.join(root,profile)})
  local calls=0
  local expected=path.join(b.root,'omb/target',profile,'moba-config.exe')
  local adapter={run=function(exe,args,settings)
    assert(exe==expected,'preflight must use selected binary, never cargo')
    assert(settings.env.OMB_LUA_CONTENT=='0' and settings.env.OMB_LUA_HOT_RELOAD=='0')
    calls=calls+1
    assert(args[1]==(calls==1 and '--lock-plan' or '--config'))
    return process.run(exe,args,settings)
  end}
  local plan=launch.prepare(options,adapter)
  assert(calls==2 and plan.profile==profile and plan.mode=='dedicated-server' and #plan.clients==0)
  assert(plan.tick_rate_hz==60 and plan.content_mode=='compiled-content-only')
  print('PASS compiled '..profile..' config and lock-plan; no cargo/server/renderer')
end
local config_command=require('moba_config_command')
local original_resolve=config_command.resolve
local refused=path.join(root,'missing-tool')
config_command.resolve=function() error('injected missing compiled tool') end
local ok,err=pcall(launch.prepare,launch.options({'--prepare-only','--no-build','--output',refused}),{
  run=function() error('must not execute any process') end,
})
config_command.resolve=original_resolve
assert(not ok and tostring(err):find('injected missing compiled tool',1,true))
assert(not path.exists(refused),'missing tool must fail before output creation')
print('PASS missing tool leaves no output and starts no process')
print('compiled configuration integration: 3 groups passed (2 native profiles, 1 injected missing-tool)')
