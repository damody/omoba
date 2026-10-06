# Bounded Grok task: complete shared Unreal renderer worker hint

Return a unified diff only. Do not call tools or edit files: this packet contains the full relevant code. Primary applies/reviews/integrates into launchers. No commit/push/reset/deletion/credentials/external action. Do not claim tests ran. Do not broaden into engine/UE/gameplay/hash/performance thresholds. All other dirty work belongs to user.

Concrete gap: shared local_budget reserves CPU capacity for renderers but never exports a renderer hint, so actual Unreal launchers ignore the reservation. Local UE5.8 source has been independently verified: FGenericPlatformMisc::GetConfiguredCoreLimits parses -corelimit=N and applies N to physical and logical core counts; Windows NumberOfCoresIncludingHyperthreads honors logical limit. Use only -corelimit=N, not experimental physicalcorelimit or affinity. This is a worker-count hint, NOT an OS CPU quota or proof of performance root cause.

Scope exactly scripts/moba_host_budget.lua and scripts/tests/moba_host_budget_test.lua. Keep existing ecs/io allocation and schema1. Add unreal_cores_per_renderer = renderers>0 ? max(1,reserved_renderer_capacity // renderers) : 0; unreal_args = renderers>0 ? {'-corelimit='..value} : {}. Export renderer_core_hint_total=renderers*cores and renderer_hint_oversubscribed = total>reserved capacity. Existing oversubscribed still refers only to simulation workers (preserve semantics). Append scope statement that Unreal respects reported physical/logical core limits but drivers/minimum engine pools/main threads aren't a CPU quota. 32logical/3simulation/2renderers must give8/renderer and -corelimit=8, reserved16. Tiny capacities clamp1 honestly mark hint oversubscribed. Zero renderers produces no flag.

Extend the matrix test to check exact formulas/flags, integers, no mutable table sharing (alter one returned unreal_args then new allocate unchanged), zero renderer empty args and no division byzero, existing validation unchanged. Primary exact validation fixedLua scripts/tests/moba_host_budget_test.lua; do not execute.

Current scripts/moba_host_budget.lua:
```lua
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
```

Current scripts/tests/moba_host_budget_test.lua:
```lua
package.path='scripts/?.lua;'..package.path
local m=require('moba_host_budget')
local n=0
for _,cpu in ipairs({1,2,4,8,16,32,64,128}) do
  for sims=1,11 do
    for renderers=0,10 do
      local r=m.allocate(cpu,sims,renderers)
      assert(r.ecs_threads_per_process>=1 and r.ecs_threads_per_process<=math.min(cpu,8))
      assert(r.tokio_workers_per_process==1 and r.worker_total==sims*(r.ecs_threads_per_process+1))
      assert(r.oversubscribed==(r.worker_total>cpu-r.reserved_renderer_capacity))
      assert(tonumber(r.env.OM_ECS_WORKER_THREADS)==r.ecs_threads_per_process)
      n=n+1
    end
  end
end
local r=m.allocate(32,3,2)
assert(r.ecs_threads_per_process==4 and r.worker_total==15 and r.reserved_renderer_capacity==16 and not r.oversubscribed)
for _,args in ipairs({{0,3,2},{1.5,3,2},{32,0,2},{32,12,2},{32,3,-1},{32,3,11}}) do
  assert(not pcall(m.allocate,table.unpack(args)))
end
print(('host worker budget: %d valid allocations + 6 invalid inputs passed'):format(n))
```

Report diff plus concrete review notes and validation not-run statement. Don't invent a successful performance run. User wants Grok bounded subagent; main owns integration and verification. Do not ask user.
