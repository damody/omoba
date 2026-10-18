-- Pure admission and command planning for source-engine initialization and refresh.
local M = {}
function M.parallel_actions(value)
  assert(type(value)=='string' and value:match('^%d+$'), '--max-parallel-actions requires an integer from 1 to 64')
  local count=tonumber(value)
  assert(count and count>=1 and count<=64, '--max-parallel-actions requires an integer from 1 to 64')
  return count
end
-- Resource limits for ordinary builds are per invocation, never engine configuration.
function M.ordinary_build_args(options)
  assert(type(options.project)=='string' and options.project~='', 'ordinary build requires a project')
  local args={'OmGameEditor','Win64','Development','-Project='..options.project,
    '-WaitMutex','-NoHotReloadFromIDE','-NoEngineChanges','-NoUBA'}
  if options.disable_action_cache then args[#args+1]='-nocache' end
  if options.parallel then args[#args+1]='-MaxParallelActions='..M.parallel_actions(tostring(options.parallel)) end
  return args
end
function M.plan(options)
	local executor=options.build_executor or 'default'
	assert(executor=='default' or executor=='local', 'invalid source-engine build executor')
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
  local args={'OmGameEditor','Win64','Development',options.project,'-WaitMutex','-NoHotReloadFromIDE'}
  if executor=='local' then args[#args+1]='-NoUBA' end
  if options.disable_action_cache then args[#args+1]='-nocache' end
  if options.parallel then args[#args+1]='-MaxParallelActions='..options.parallel end
  -- Match the source Editor prerequisite emitted by UE's VS project generator.
  local shader_args={'ShaderCompileWorker','Win64','Development','-WaitMutex'}
  if executor=='local' then shader_args[#shader_args+1]='-NoUBA' end
  if options.disable_action_cache then shader_args[#shader_args+1]='-nocache' end
  if options.parallel then shader_args[#shader_args+1]='-MaxParallelActions='..options.parallel end
  return {initialize=true,args=args,shader_args=shader_args}
end
return M
