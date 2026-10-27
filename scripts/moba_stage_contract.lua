-- Artifact-copy consistency, not gameplay/feature/content admission.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,hash=b.lib('path'),b.lib('hash')
local M={}
-- PROJECT is the Unreal project whose plugin binaries a launch will run: by default the one
-- OM_UE_RUN_BINARIES names (E116: a session's shadow build, copied into the run snapshot),
-- otherwise live omfue. A build that just staged live omfue passes it explicitly.
function M.bindings(root,project)
  local env=os.getenv('OM_UE_RUN_BINARIES')
  project=project or (env and env~='' and path.absolute(env,root)) or path.join(root,'omfue')
  local plugin=path.join(project,'Plugins/OmRuntime/Binaries/Win64')
  return {
    {role='bridge',built=path.join(root,'omfue/bridge/target/debug/om_bridge.dll'),
      copies={path.join(plugin,'om_bridge.dll')}},
    {role='base_content',built=path.join(root,'scripts/target/debug/base_content.dll'),
      copies={path.join(root,'scripts/base_content.dll'),path.join(plugin,'base_content.dll')}},
  }
end
-- A build checks these right after staging. A launch does not (E124): it runs its own
-- base_content copy (run_artifacts.lua) and checks the project's bridge against its stage
-- record (ue_bridge_stage.verify_launch), so another session's rebuild never blocks it.
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
