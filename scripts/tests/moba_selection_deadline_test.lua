package.path='scripts/?.lua;'..package.path
local budget,launch=require('moba_selection_deadline'),require('moba_role_launch')
local now=10000
local clock={monotonic_ms=function() return now end}
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
assert(budget.start({},clock)==nil)
assert(budget.start({selection_smoke_hero='training_ranger'},clock)==130000)
assert(budget.start({selection_smoke_hero='training_ranger',selection_timeout_seconds=7},clock)==17000)
local deadline=budget.start({selection_timeout_seconds=1},clock)
now=10999
budget.check(deadline,clock)
assert(budget.wait_ms(deadline,clock,15000)==1)
assert(budget.wait_ms(nil,clock,5000)==5000)
now=11000
rejects(function() budget.check(deadline,clock) end)
rejects(function() budget.wait_ms(deadline,clock,15000) end)
for _,raw in ipairs({'0','7201','1.5','NaN','-1','1e2'}) do
  rejects(function() launch.options({'--interactive-selection','--selection-timeout-seconds',raw}) end)
end
rejects(function() launch.options({'--selection-timeout-seconds','60'}) end)
assert(launch.options({'--interactive-selection','--selection-timeout-seconds','7200'}).selection_timeout_seconds==7200)
print('selection deadline: manual/default automation/override/boundary/drain and option guards passed')
