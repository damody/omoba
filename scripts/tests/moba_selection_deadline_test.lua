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
local exists=false
local paths={absolute=function(value,root) assert(value=='cancel.signal' and root);return 'D:/cancel.signal' end,
  exists=function(value) assert(value=='D:/cancel.signal');return exists end}
local no_clock={monotonic_ms=function() error('unbounded cancellation must not sample a clock') end}
local manual=budget.start({selection_cancel_file='cancel.signal'},no_clock,paths)
assert(type(manual)=='table' and manual.cancel_file=='D:/cancel.signal')
budget.check(manual,no_clock)
assert(budget.wait_ms(manual,no_clock,5000)==5000)
exists=true
rejects(function() budget.check(manual,no_clock) end)
rejects(function() budget.wait_ms(manual,no_clock,5000) end)
rejects(function() budget.start({selection_cancel_file='cancel.signal'},no_clock,paths) end)
exists=false;now=1000
local combined=budget.start({selection_cancel_file='cancel.signal',selection_timeout_seconds=1},clock,paths)
now=1999;assert(budget.wait_ms(combined,clock,5000)==1)
exists=true;rejects(function() budget.wait_ms(combined,clock,5000) end)
exists=false;now=2000;rejects(function() budget.check(combined,clock) end)
rejects(function() launch.options({'--selection-cancel-file','cancel.signal'}) end)
rejects(function() launch.options({'--interactive-selection','--selection-cancel-file',''}) end)
assert(launch.options({'--interactive-selection','--selection-cancel-file','cancel.signal'}).selection_cancel_file:match('cancel.signal$'))
local headless=require('moba_selection_host').options({'--selection-bind','127.0.0.1',
  '--output','new','--selection-cancel-file','cancel.signal'})
local remote=require('moba_selection_join').options({'--invite','private.json','--player','1',
  '--output','new','--selection-cancel-file','cancel.signal'})
assert(headless.selection_cancel_file==remote.selection_cancel_file)
print('selection cancellation: unbounded/no-clock, preexisting, late request, deadline cap and three entry option guards passed')
