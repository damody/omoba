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
test('verify authority and matching content without frontend',function()
  local files={}
  local dependencies={is_file=function(file) files[#files+1]=file;return true end,
    sha256=function() return string.rep('a',64) end}
  local report=build.verify({profile='release'},dependencies)
  assert(#report.artifacts==1 and #files==4)
  for _,file in ipairs(files) do assert(not file:find('omfue',1,true)) end
  dependencies.is_file=function(file) return not file:find('moba-config.exe',1,true) end
  rejects(function() build.verify({},dependencies) end)
  dependencies.is_file=function() return true end
  dependencies.sha256=function(file) return string.rep(file:gsub('\\','/'):find('/target/',1,true) and 'a' or 'b',64) end
  rejects(function() build.verify({},dependencies) end)
end)
print(('server build: %d groups passed (injected commands/files; no build or game started)'):format(count))
