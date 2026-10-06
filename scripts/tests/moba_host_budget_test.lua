package.path='scripts/?.lua;'..package.path
local m=require('moba_host_budget')
local n=0
local scope='local-worker-hint-not-os-quota; excludes main/driver/library/renderer threads; unreal respects reported physical/logical core limits but drivers/minimum engine pools/main threads are not a cpu quota'
for _,cpu in ipairs({1,2,4,8,16,32,64,128}) do
  for sims=1,11 do
    for renderers=0,10 do
      local r=m.allocate(cpu,sims,renderers)
      assert(r.schema_version==1)
      assert(r.ecs_threads_per_process>=1 and r.ecs_threads_per_process<=math.min(cpu,8))
      assert(r.tokio_workers_per_process==1 and r.worker_total==sims*(r.ecs_threads_per_process+1))
      assert(r.oversubscribed==(r.worker_total>cpu-r.reserved_renderer_capacity))
      assert(tonumber(r.env.OM_ECS_WORKER_THREADS)==r.ecs_threads_per_process)
      local sim_capacity=renderers>0 and math.max(1,cpu//2) or cpu
      local reserved=cpu-sim_capacity
      local cores=renderers>0 and math.max(1,reserved//renderers) or 0
      assert(r.reserved_renderer_capacity==reserved)
      assert(r.unreal_cores_per_renderer==cores and math.type(r.unreal_cores_per_renderer)=='integer')
      assert(r.renderer_core_hint_total==renderers*cores and math.type(r.renderer_core_hint_total)=='integer')
      assert(r.renderer_hint_oversubscribed==(renderers*cores>reserved))
      assert(r.scope==scope)
      if renderers==0 then assert(next(r.unreal_args)==nil and cores==0)
      else assert(r.unreal_args[1]=='-corelimit='..cores and r.unreal_args[2]==nil) end
      n=n+1
    end
  end
end
local r=m.allocate(32,3,2)
assert(r.ecs_threads_per_process==4 and r.worker_total==15 and r.reserved_renderer_capacity==16 and not r.oversubscribed)
assert(r.unreal_cores_per_renderer==8 and r.renderer_core_hint_total==16 and not r.renderer_hint_oversubscribed)
assert(r.unreal_args[1]=='-corelimit=8' and #r.unreal_args==1)
r.unreal_args[1]='mutated';r.unreal_args[2]='-physicalcorelimit=1'
local again=m.allocate(32,3,2)
assert(again.unreal_args~=r.unreal_args and again.unreal_args[1]=='-corelimit=8' and again.unreal_args[2]==nil)
local zero=m.allocate(32,3,0)
assert(#zero.unreal_args==0 and zero.unreal_cores_per_renderer==0 and zero.renderer_core_hint_total==0 and not zero.renderer_hint_oversubscribed)
zero.unreal_args[1]='-corelimit=1'
local zero_again=m.allocate(32,3,0)
assert(zero_again.unreal_args~=zero.unreal_args and #zero_again.unreal_args==0)
local tiny=m.allocate(1,1,2)
assert(tiny.reserved_renderer_capacity==0 and tiny.unreal_cores_per_renderer==1 and tiny.renderer_core_hint_total==2 and tiny.renderer_hint_oversubscribed)
assert(tiny.unreal_args[1]=='-corelimit=1')
for _,args in ipairs({{0,3,2},{1.5,3,2},{32,0,2},{32,12,2},{32,3,-1},{32,3,11}}) do
  assert(not pcall(m.allocate,table.unpack(args)))
end
print(('host worker budget: %d valid allocations + 6 invalid inputs passed'):format(n))
