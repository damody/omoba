-- Pure admission and command planning for source-engine initialization and refresh.
local M = {}

-- Environment of every Build.bat run (E402): MSBuild's reusable worker nodes are shared by every
-- build on the machine, and one build ending shut down the nodes another was using (MSB4166, exit
-- 999). Each build keeps its own nodes.
M.BUILD_ENV = {MSBUILDDISABLENODEREUSE = '1'}
function M.build_env(extra)
  local env = {}
  for key, value in pairs(M.BUILD_ENV) do env[key] = value end
  for key, value in pairs(extra or {}) do env[key] = value end
  return env
end
function M.compiler_version(value)
  assert(type(value)=='string' and (value=='Latest' or value=='Preview'
    or value:match('^%d+%.%d+%.%d+$') or value:match('^%d+%.%d+%.%d+%.%d+$')),
    '--compiler-version requires Latest, Preview or a numeric MSVC version')
  return value
end
-- Parallel compile actions by available commit (E443). UBT sizes its actions from free physical
-- memory at 1.5 GB each (16 on this machine), but one cl.exe on this project's unity files commits
-- about 2.7 GB, so the machine runs out of commit long before RAM and other sessions' processes end
-- with 0xC000012D (STATUS_COMMITMENT_LIMIT). An ordinary build caps its actions by the commit left
-- when it starts (memory_admission.parallel_jobs over lua-host memory_status, E441).
M.COMMIT_PER_ACTION = 3 * 1024 * 1024 * 1024
M.MAX_COMMIT_ACTIONS = 16
function M.commit_parallel_actions(status)
  return require('memory_admission').parallel_jobs(M.COMMIT_PER_ACTION, M.MAX_COMMIT_ACTIONS, status)
end
function M.parallel_actions(value)
  assert(type(value)=='string' and value:match('^%d+$'), '--max-parallel-actions requires an integer from 1 to 64')
  local count=tonumber(value)
  assert(count and count>=1 and count<=64, '--max-parallel-actions requires an integer from 1 to 64')
  return count
end
-- Re-discover project source membership on every build. Cached UBT makefiles
-- may otherwise silently omit new .cpp files; keep engine/global config intact.
-- Resource limits for ordinary builds are per invocation, never engine configuration.
function M.ordinary_build_args(options)
  assert(type(options.project)=='string' and options.project~='', 'ordinary build requires a project')
  local args={'OmGameEditor','Win64','Development','-Project='..options.project,
    '-WaitMutex','-NoHotReloadFromIDE','-NoEngineChanges','-NoUBA','-NoUBTMakefiles'}
  if options.compiler_version then args[#args+1]='-CompilerVersion='..M.compiler_version(options.compiler_version) end
  if options.disable_action_cache then args[#args+1]='-nocache' end
  if options.parallel then args[#args+1]='-MaxParallelActions='..M.parallel_actions(tostring(options.parallel)) end
  return args
end
function M.plan(options)
	local executor=options.build_executor or 'default'
	assert(executor=='default' or executor=='local', 'invalid source-engine build executor')
  assert(not options.compiler_version or (executor=='local' and options.mode~='verify'),
    '--compiler-version requires a local build')
  if options.compiler_version then M.compiler_version(options.compiler_version) end
  assert(not options.disable_action_cache or (executor=='local' and options.mode~='verify'),
    '--disable-action-cache requires a local build')
  assert(not options.refresh or options.requested, '--refresh-engine requires explicit source-engine build admission')
  assert(not (options.requested and options.mode=='verify'), '--bootstrap-engine cannot be used with --verify-staged-only')
  assert(not options.parallel or options.requested, '--max-parallel-actions requires --bootstrap-engine')
  if options.parallel then M.parallel_actions(tostring(options.parallel)) end
  if options.mode=='verify' then return {initialize=false} end
  if options.editor_present and not (options.requested and (options.initialization_pending or options.refresh)) then
    return {initialize=false}
  end
  assert(options.requested, 'UnrealEditor.exe is missing; initialize a source engine with --bootstrap-engine')
  assert(options.source_present and options.build_present,
    '--bootstrap-engine requires engine source and Build.bat; refusing before any build')
  local args={'OmGameEditor','Win64','Development',options.project,'-WaitMutex','-NoHotReloadFromIDE','-NoUBTMakefiles'}
  if executor=='local' then args[#args+1]='-NoUBA' end
  if options.disable_action_cache then args[#args+1]='-nocache' end
  if options.compiler_version then args[#args+1]='-CompilerVersion='..options.compiler_version end
  if options.parallel then args[#args+1]='-MaxParallelActions='..options.parallel end
  -- Match the source Editor prerequisite emitted by UE's VS project generator.
  local shader_args={'ShaderCompileWorker','Win64','Development','-WaitMutex'}
  if executor=='local' then shader_args[#shader_args+1]='-NoUBA' end
  if options.disable_action_cache then shader_args[#shader_args+1]='-nocache' end
  if options.compiler_version then shader_args[#shader_args+1]='-CompilerVersion='..options.compiler_version end
  if options.parallel then shader_args[#shader_args+1]='-MaxParallelActions='..options.parallel end
  return {initialize=true,args=args,shader_args=shader_args}
end
return M
