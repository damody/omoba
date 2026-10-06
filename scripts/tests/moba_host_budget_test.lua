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
