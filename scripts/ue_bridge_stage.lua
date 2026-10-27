-- Bridge staging into an Unreal project (live omfue or a session's shadow project, E116),
-- with a record of what was staged together (E124). The om_bridge ABI is compiled into the
-- Unreal binaries through om_bridge.h, so a launch checks that the project's om_bridge.dll is
-- the one staged with its header, not that it equals whatever omfue/bridge/target holds now:
-- another session rebuilding the bridge made every other launch fail its preflight although
-- the project it launched was self-consistent. base_content is staged too, but a launch runs
-- its own copy (run_artifacts.lua), so it is not part of the record.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.DLL='Plugins/OmRuntime/Binaries/Win64/om_bridge.dll'
M.HEADER='Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h'
M.LIB='Plugins/OmRuntime/Source/ThirdParty/OmBridge/lib/Win64/om_bridge.lib'
M.BASE_CONTENT='Plugins/OmRuntime/Binaries/Win64/base_content.dll'
M.RECORD='Plugins/OmRuntime/Binaries/Win64/om_bridge.stage.json'

local function deps_of(deps)
  deps=deps or {}
  local b=require('_bootstrap')
  return {path=deps.path or b.lib('path'),sha256=deps.sha256 or b.lib('hash').sha256,
    json=deps.json or b.lib('json'),log=deps.log or print}
end

-- A crate's cargo target directory, or OM_RUST_RUN_BINARIES (one session-owned target for
-- every crate, as in run_artifacts.rust_target; E125).
local function rust_target(d,root,...)
  local env=os.getenv('OM_RUST_RUN_BINARIES')
  if env and env~='' then return d.path.absolute(env,root) end
  return d.path.join(root,...)
end

-- The current build outputs a stage copies: the header cbindgen wrote to HEADER_FILE, the
-- bridge build and the debug base_content build (the parent build makes it right before).
function M.sources(root,header_file,deps)
  local d=deps_of(deps)
  local bridge=d.path.join(rust_target(d,root,'omfue','bridge','target'),'debug')
  return {
    {role='header',source=header_file,target=M.HEADER},
    {role='lib',source=d.path.join(bridge,'om_bridge.dll.lib'),target=M.LIB},
    {role='bridge',source=d.path.join(bridge,'om_bridge.dll'),target=M.DLL},
    {role='base_content',source=d.path.join(rust_target(d,root,'scripts','target'),'debug','base_content.dll'),target=M.BASE_CONTENT},
  }
end

-- Copies SOURCES into PROJECT and writes the record. Returns the record.
function M.stage(project,sources,deps)
  local d=deps_of(deps)
  local record={schema_version=1,scope='ue-bridge-stage',project=project,artifacts={}}
  for _,item in ipairs(sources) do
    assert(d.path.is_file(item.source),'missing current build artifact: '..item.source)
    local sha=d.sha256(item.source):lower()
    local destination=d.path.join(project,item.target)
    if d.path.is_file(destination) and d.sha256(destination):lower()==sha then
      d.log('[om-bridge] unchanged '..destination)
    else
      d.path.mkdir_p(d.path.parent(destination))
      d.path.write(destination,d.path.read(item.source,true),true,true)
      assert(d.sha256(destination):lower()==sha,'staging SHA mismatch: '..destination)
      d.log('[om-bridge] staged '..destination..' '..sha)
    end
    if item.role~='base_content' then
      record.artifacts[item.role]={source=item.source,target=item.target,sha256=sha}
    end
  end
  d.path.write(d.path.join(project,M.RECORD),d.json.encode(record),true)
  return record
end

-- The Unreal project a launch runs: OM_UE_RUN_BINARIES (a session's shadow build) or live omfue.
function M.project(root,deps)
  local d=deps_of(deps)
  local env=os.getenv('OM_UE_RUN_BINARIES')
  return (env and env~='' and d.path.absolute(env,root)) or d.path.join(root,'omfue')
end

-- Launch check of PROJECT's bridge (OM_UE_RUN_BINARIES or live omfue by default). With a
-- record: the DLL and header must be the ones staged together. Without one (a project staged
-- before records existed): the DLL must equal the current bridge build, as before.
function M.verify_launch(root,project,deps)
  local d=deps_of(deps)
  project=project or M.project(root,d)
  local dll=d.path.join(project,M.DLL)
  local record_file=d.path.join(project,M.RECORD)
  local report={schema_version=1,scope='artifact-copy-consistency',artifacts={}}
  assert(d.path.is_file(dll),'missing staged bridge: '..dll)
  local actual=d.sha256(dll):lower()
  local built=d.path.join(rust_target(d,root,'omfue','bridge','target'),'debug','om_bridge.dll')
  if not d.path.is_file(record_file) then
    assert(d.path.is_file(built),'missing built bridge: '..built)
    assert(d.sha256(built):lower()==actual,'staged bridge differs from current build; rebuild/stage before launch: '..dll)
    report.artifacts[1]={role='bridge',built=built,sha256=actual,copies={dll}}
    return report
  end
  local record=d.json.decode(d.path.read(record_file))
  local bridge,header=record.artifacts and record.artifacts.bridge,record.artifacts and record.artifacts.header
  assert(bridge and header,'invalid bridge stage record: '..record_file)
  assert(actual==bridge.sha256,'staged bridge differs from its stage record; restage before launch: '..dll)
  local header_file=d.path.join(project,M.HEADER)
  assert(d.path.is_file(header_file) and d.sha256(header_file):lower()==header.sha256,
    'staged bridge header differs from its stage record; restage and rebuild before launch: '..header_file)
  if d.path.is_file(built) and d.sha256(built):lower()~=actual then
    d.log('[moba-ue] note: '..dll..' is not the newest bridge build (omfue/bridge/target/debug); build the frontend to run that one')
  end
  report.artifacts[1]={role='bridge',built=bridge.source,sha256=actual,copies={dll}}
  return report
end

return M
