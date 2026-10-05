-- Production dependency contract: build-time Lua is allowed, normal edges are not.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local process=b.lib('process')
local checks={
  {'server','omb/Cargo.toml','omobab'},
  {'client-runtime','omoba-client-runtime/Cargo.toml'},
  {'script-dll','scripts/Cargo.toml','base_content'},
  {'unreal-bridge','omfue/bridge/Cargo.toml'},
}
for _,check in ipairs(checks) do
  local args={'tree','--manifest-path',check[2],'--features','compiled-content-only','-e','normal','--prefix','none'}
  if check[3] then args[#args+1]='-p';args[#args+1]=check[3] end
  local result=process.run('cargo',args,{cwd=b.root})
  assert(result.stdout:match('%S'),check[1]..': empty dependency report')
  for line in result.stdout:gmatch('[^\r\n]+') do
    assert(not line:match('^mlua[%s%-]') and not line:match('^lua%-src%s')
      and not line:match('^luajit%-src%s'),check[1]..' links runtime Lua: '..line)
  end
  print('PASS '..check[1]..': no Lua VM on normal dependency edges')
end
print('compiled content dependency contract: 4/4 passed (build-time authoring excluded)')
