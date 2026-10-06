-- Shared local-host worker budget. This is a scheduling hint, not an OS quota.
local M = {}
function M.allocate(logical, simulations, renderers)
  assert(math.type(logical)=='integer' and logical>=1 and logical<=16384, 'invalid CPU capacity')
  assert(math.type(simulations)=='integer' and simulations>=1 and simulations<=11, 'invalid simulation process count')
  assert(math.type(renderers)=='integer' and renderers>=0 and renderers<=10, 'invalid renderer process count')
  local capacity = renderers>0 and math.max(1,logical//2) or logical
  local per_process = math.max(1,capacity//simulations)
  local ecs = math.max(1,math.min(8,per_process-1))
  local io = 1
  return {schema_version=1,logical_cpus=logical,simulation_processes=simulations,renderer_processes=renderers,
    reserved_renderer_capacity=logical-capacity,ecs_threads_per_process=ecs,tokio_workers_per_process=io,
    worker_total=simulations*(ecs+io),oversubscribed=simulations*(ecs+io)>capacity,
    scope='local-worker-hint-not-os-quota; excludes main/driver/library/renderer threads',
    env={OM_ECS_WORKER_THREADS=tostring(ecs),TOKIO_WORKER_THREADS=tostring(io)}}
end
function M.local_budget(simulations,renderers)
  local capacity=require('tools.lua.lib.host').call('cpu_capacity',{})
  return M.allocate(capacity.logical_cpus,simulations,renderers)
end
return M
