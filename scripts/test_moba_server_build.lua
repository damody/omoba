local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local build=require('moba_server_build')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
test('profile-matched Rust and Lua stages only',function()
  for _,profile in ipairs({'debug','release'}) do
    local commands=build.commands({profile=profile})
    assert(#commands==4)
    for index,command in ipairs(commands) do
      local text=table.concat(command.args,' ')
      assert(not text:find('omfue',1,true) and not text:find('client-runtime',1,true))
      if index<=3 then
        assert(command.exe=='cargo' and text:find('compiled-content-only',1,true))
        assert((text:find('--release',1,true)~=nil)==(profile=='release'))
      else assert(text:find('--profile '..profile,1,true)) end
    end
  end
  rejects(function() build.commands({profile='unexpected'}) end)
end)
test('failed build never stages',function()
  local calls=0
  rejects(function() build.build({}, {run=function() calls=calls+1;return {exit_code=1} end}) end)
  assert(calls==1)
end)
test('successful build executes exact four stages',function()
  local calls=0
  build.build({profile='debug'},{run=function() calls=calls+1;return {exit_code=0} end})
  assert(calls==4)
end)
test('verify authority and a base_content build without frontend or shared copy',function()
  local files={}
  local dependencies={is_file=function(file) files[#files+1]=file;return true end}
  local report=build.verify({profile='release'},dependencies)
  assert(report.base_content_profile=='release' and #files==3)
  for _,file in ipairs(files) do
    assert(not file:find('omfue',1,true))
    -- E124: the shared scripts/base_content.dll is never consulted; runs stage their own copy.
    assert(not file:gsub('\\','/'):find('scripts/base_content.dll',1,true))
  end
  dependencies.is_file=function(file) return not file:find('moba-config.exe',1,true) end
  rejects(function() build.verify({},dependencies) end)
end)
test('either profile build satisfies the authority, the selected one first',function()
  local function only(profile) return function(file) file=file:gsub('\\','/')
    return not file:find('base_content',1,true) or file:find('/target/'..profile..'/',1,true)~=nil end end
  assert(build.verify({profile='release'},{is_file=only('debug')}).base_content_profile=='debug')
  assert(build.verify({profile='debug'},{is_file=only('release')}).base_content_profile=='release')
  assert(build.verify({profile='debug'},{is_file=function() return true end}).base_content_profile=='debug')
  rejects(function() build.verify({profile='release'},{is_file=only('none')}) end)
end)
print(('server build: %d groups passed (injected commands/files; no build or game started)'):format(count))
