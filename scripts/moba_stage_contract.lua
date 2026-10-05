-- Artifact-copy consistency, not gameplay/feature/content admission.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,hash=b.lib('path'),b.lib('hash')
local M={}
function M.bindings(root)
  return {
    {role='bridge',built=path.join(root,'omfue/bridge/target/debug/om_bridge.dll'),
      copies={path.join(root,'omfue/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll')}},
    {role='base_content',built=path.join(root,'scripts/target/debug/base_content.dll'),
      copies={path.join(root,'scripts/base_content.dll'),
        path.join(root,'omfue/Plugins/OmRuntime/Binaries/Win64/base_content.dll')}},
  }
end
function M.verify(bindings,dependencies)
  dependencies=dependencies or {is_file=path.is_file,sha256=hash.sha256}
  local report={schema_version=1,scope='artifact-copy-consistency',artifacts={}}
  for _,binding in ipairs(bindings) do
    assert(dependencies.is_file(binding.built),'missing built '..binding.role..': '..binding.built)
    local expected=dependencies.sha256(binding.built)
    assert(type(expected)=='string' and expected:match('^%x+$') and #expected==64,'invalid artifact digest')
    local row={role=binding.role,built=binding.built,sha256=expected:lower(),copies={}}
    for _,copy in ipairs(binding.copies) do
      assert(dependencies.is_file(copy),'missing staged '..binding.role..': '..copy)
      assert(dependencies.sha256(copy):lower()==row.sha256,
        'staged '..binding.role..' differs from current build; rebuild/stage before launch: '..copy)
      row.copies[#row.copies+1]=copy
    end
    report.artifacts[#report.artifacts+1]=row
  end
  return report
end
return M
