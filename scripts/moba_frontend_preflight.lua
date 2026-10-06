-- Read-only frontend deployment gate shared by gameplay and remote selection.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,platform=b.lib('path'),b.lib('platform')
local M={}
function M.verify(ue_root,process,dependencies)
  assert(type(ue_root)=='string' and ue_root~='','frontend verification requires selected Unreal root')
  dependencies=dependencies or {}
  local require_ready=dependencies.require_ready or require('ue_binary_preflight').require_ready
  require_ready(ue_root,path.join(b.root,'omfue','om.uproject'))
  local result=process.run(platform.lua_executable,{
    path.join(b.root,'scripts','build_ue_moba.lua'),'--verify-staged-only','--ue-root',ue_root,
  },{cwd=b.root,check=false,label='frontend staged verification'})
  assert(type(result)=='table' and result.exit_code==0,
    'frontend staged verification failed; no automatic repair or fallback\nexit '..
    tostring(type(result)=='table' and result.exit_code or nil)..'\nstdout:\n'..
    tostring(type(result)=='table' and result.stdout or '')..'\nstderr:\n'..
    tostring(type(result)=='table' and result.stderr or ''))
  if result.stdout and result.stdout~='' then print(result.stdout) end
  return result
end
return M
