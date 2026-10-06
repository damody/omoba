-- Compiled configuration tool policy shared by pre-match and selection workflows.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local M={}
function M.resolve(options,dependencies)
  local profile=options.profile or 'release'
  assert(profile=='debug' or profile=='release','configuration profile must be debug or release')
  local exe=path.join(b.root,'omb/target',profile,'moba-config.exe')
  dependencies=dependencies or {is_file=path.is_file}
  assert(dependencies.is_file(exe),'missing compiled configuration tool: '..exe..
    '\nBuild explicitly: cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only'..
    (profile=='release' and ' --release' or '')..'\nNo automatic build or profile fallback.')
  return exe
end
function M.run(options,args,process,dependencies)
  local exe=M.resolve(options,dependencies)
  local copied={};for i,arg in ipairs(args) do copied[i]=arg end
  local result=process.run(exe,copied,{cwd=b.root,
    env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},check=false,label='compiled role configuration'})
  assert(result.exit_code==0,'compiled configuration tool failed: '..exe..' exit '..tostring(result.exit_code)..
    '\nstdout:\n'..tostring(result.stdout or '')..'\nstderr:\n'..tostring(result.stderr or ''))
  return result
end
return M
