local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local command=require('moba_config_command')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn,text)
  local ok,err=pcall(fn);assert(not ok,'expected rejection')
  if text then assert(tostring(err):find(text,1,true),tostring(err)) end
  return tostring(err)
end
test('profile exact resolution and default',function()
  for _,profile in ipairs({'debug','release'}) do
    local expected=path.join(b.root,'omb/target',profile,'moba-config.exe')
    local visits=0
    assert(command.resolve({profile=profile},{is_file=function(exe)
      visits=visits+1;assert(exe==expected);return true
    end})==expected and visits==1)
  end
  assert(command.resolve({}, {is_file=function() return true end})==path.join(b.root,'omb/target/release/moba-config.exe'))
  rejects(function() command.resolve({profile='other'},{is_file=function() error('must not query') end}) end,'profile')
end)
test('all deployment flags never fallback on missing binary',function()
  for _,flag in ipairs({'no_build','prepare_only','server_only','connect'}) do
    local options={profile='release',[flag]=true}
    local visits=0
    local err=rejects(function() command.resolve(options,{is_file=function()
      visits=visits+1;return false
    end}) end,'--features compiled-content-only --release')
    assert(visits==1 and err:find('No automatic build or profile fallback',1,true))
    assert(err:find(path.join(b.root,'omb/target/release/moba-config.exe'),1,true))
  end
  local err=rejects(function() command.resolve({profile='debug'},{is_file=function() return false end}) end)
  assert(not err:find('--release',1,true))
end)
test('compiled-only process environment and caller argument isolation',function()
  local args={'--lock-plan','candidate.json'}
  local result={exit_code=0,stdout='success'}
  assert(command.run({profile='debug'},args,{run=function(exe,copy,settings)
    assert(exe==path.join(b.root,'omb/target/debug/moba-config.exe'))
    assert(copy~=args and copy[1]==args[1] and copy[2]==args[2])
    assert(settings.cwd==b.root and settings.check==false)
    assert(settings.env.OMB_LUA_CONTENT=='0' and settings.env.OMB_LUA_HOT_RELOAD=='0')
    copy[1]='mutated';return result
  end},{is_file=function() return true end})==result)
  assert(args[1]=='--lock-plan')
end)
test('failed or unknown exit status rejected with diagnostics',function()
  for _,result in ipairs({{exit_code=1,stdout='old-json',stderr='invalid-config'},
    {stdout='old-json',stderr='invalid-config'}}) do
    local err=rejects(function() command.run({}, {},{run=function() return result end},
      {is_file=function() return true end}) end,'compiled configuration tool failed')
    assert(err:find('old-json',1,true) and err:find('invalid-config',1,true))
  end
end)
test('missing tool never invokes process',function()
  rejects(function() command.run({}, {},{run=function() error('must not invoke') end},
    {is_file=function() return false end}) end,'missing compiled configuration tool')
end)
print(('compiled config command: %d groups passed; injected files/process, no builds or games'):format(count))
