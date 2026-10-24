local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local gate=require('ue_build_prerequisites')
local count=0
local function accept(output, version)
  assert(gate.require_ready({run=function(exe,args,options)
    assert(exe=='cbindgen' and #args==1 and args[1]=='--version' and options.check==false)
    return {exit_code=0,stdout=output}
  end})==version)
  count=count+1
end
local function reject(run)
  local ok,err=pcall(gate.require_ready,{run=run})
  assert(not ok and tostring(err):find('cargo install cbindgen --version 0.29.4 --locked',1,true),tostring(err))
  count=count+1
end
accept('cbindgen 0.29.4\r\n','0.29.4')
accept('cbindgen 0.29.4\n','0.29.4')
accept('cbindgen 0.29.4','0.29.4')
reject(function() error('executable not found') end)
reject(function() return {exit_code=9009,stdout='cbindgen 0.29.4'} end)
reject(function() return {exit_code=0,stdout='unrelated executable 0.29.4'} end)
reject(function() return {exit_code=0,stdout='cbindgen 0.29.4\nerror'} end)
reject(function() return {exit_code=0} end)
reject(function() return nil end)
print(('Unreal build prerequisite: %d checks passed'):format(count))
