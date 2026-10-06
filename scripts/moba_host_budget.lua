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
  local reserved = logical-capacity
  local unreal_cores = renderers>0 and math.max(1,reserved//renderers) or 0
  local unreal_args = renderers>0 and {'-corelimit='..unreal_cores} or {}
  return {schema_version=1,logical_cpus=logical,simulation_processes=simulations,renderer_processes=renderers,
    reserved_renderer_capacity=reserved,ecs_threads_per_process=ecs,tokio_workers_per_process=io,
    worker_total=simulations*(ecs+io),oversubscribed=simulations*(ecs+io)>capacity,
    unreal_cores_per_renderer=unreal_cores,unreal_args=unreal_args,
    renderer_core_hint_total=renderers*unreal_cores,renderer_hint_oversubscribed=renderers*unreal_cores>reserved,
    scope='local-worker-hint-not-os-quota; excludes main/driver/library/renderer threads; unreal respects reported physical/logical core limits but drivers/minimum engine pools/main threads are not a cpu quota',
    env={OM_ECS_WORKER_THREADS=tostring(ecs),TOKIO_WORKER_THREADS=tostring(io)}}
end
function M.local_budget(simulations,renderers)
  local capacity=require('tools.lua.lib.host').call('cpu_capacity',{})
  return M.allocate(capacity.logical_cpus,simulations,renderers)
end
return M
