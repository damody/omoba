package.path='scripts/?.lua;'..package.path
local launch=require('moba_role_launch')

-- Presentation IPC follows the player ID, so two same-machine --connect runs differ.
assert(launch.presentation_address(57061,1)=='127.0.0.1:57062')
assert(launch.presentation_address(57061,2)=='127.0.0.1:57063')
assert(launch.presentation_address(57061,7)=='127.0.0.1:57068')
assert(not pcall(launch.presentation_address,65530,6))
assert(not pcall(launch.presentation_address,57061,0))

local function parse(args) return launch.options(args) end
local default=parse({})
assert(default.host_simulations==nil and default.host_renderers==nil)
local split=parse({'--server-only','--host-simulations','3','--host-renderers','2'})
assert(split.host_simulations==3 and split.host_renderers==2)
local client=parse({'--connect','127.0.0.1','--recipe','plan.json','--local-player','2',
  '--host-simulations','3','--host-renderers','2'})
assert(client.host_simulations==3 and client.host_renderers==2)
assert(parse({'--host-renderers','0'}).host_renderers==0)
assert(parse({'--debug-query'}).debug_query==true and parse({}).debug_query==nil)
for _,bad in ipairs({{'--host-simulations','0'},{'--host-simulations','12'},{'--host-renderers','11'},
    {'--host-renderers','-1'},{'--host-simulations','1.5'},{'--host-simulations','x'}}) do
  assert(not pcall(parse,bad),'accepted invalid '..bad[1]..' '..bad[2])
end

-- Client autopilot is a renderer option: profile names only, never on a dedicated server.
assert(parse({'--autopilot','lane_v0'}).autopilot=='lane_v0')
for _,bad in ipairs({{'--autopilot','bad profile'},{'--server-only','--autopilot','lane_v0'},
    {'--interactive-selection','--autopilot','lane_v0'}}) do
  assert(not pcall(parse,bad),'accepted invalid autopilot option')
end

-- Project switches must not contain engine FParse substring tokens (E029).
assert(launch.assert_engine_safe_args({'-om-autopilot=lane_v0','-om-autopilot-shot-interval=120','-game','-ExecCmds=t.MaxFPS 60'}))
for _,bad in ipairs({'-om-autopilot-screenshot-seconds=120','-om-x-benchmark','-om-target-fps=30','-OM-Y-Seconds=5'}) do
  assert(not pcall(launch.assert_engine_safe_args,{bad}),'accepted '..bad)
end

-- Identical totals give every split invocation the same per-process share.
local budget=require('moba_host_budget')
local shared=budget.allocate(32,3,2)
local alone_server=budget.allocate(32,1,0)
assert(shared.ecs_threads_per_process<alone_server.ecs_threads_per_process)
assert(not shared.oversubscribed and shared.unreal_cores_per_renderer==8)
print('role launcher host split: presentation ports and host totals passed')
