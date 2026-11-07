package.path='scripts/?.lua;'..package.path
local build=require('consistent_build')
-- Settled inputs by default in these fixtures (no start wait).
local settled={newest_input=function() return 0 end,now=function() return 100000 end,sleep_ms=function() end}
local function with(options) for k,v in pairs(settled) do if options[k]==nil then options[k]=v end end return options end

-- One stable pass runs the three builds exactly once.
local calls=0
local passes=build.run(with({ue_root='U',quiet=true,fingerprint=function() return 'same' end}),
  function() calls=calls+1;return {exit_code=0} end)
assert(passes==1 and calls==4)

-- A source change during the pass forces the whole set again.
local values={'a','b','b','b'}
local index=0
calls=0
passes=build.run(with({ue_root='U',quiet=true,fingerprint=function() index=index+1;return values[index] end}),
  function() calls=calls+1;return {exit_code=0} end)
assert(passes==2 and calls==8)

-- A failure with settled inputs stops immediately.
calls=0
assert(not pcall(build.run,{ue_root='U',quiet=true,fingerprint=function() return 'x' end,
  newest_input=function() return 0 end,now=function() return 10000 end},
  function() calls=calls+1;return {exit_code=calls==2 and 1 or 0} end))
assert(calls==2)
-- A failure while another writer edited recently waits and retries the whole set.
calls=0
local slept=0
local passes_after_wait=build.run({ue_root='U',quiet=true,fingerprint=function() return 'x' end,
  newest_input=function() return 9900 end,now=function() return 10000 end,sleep_ms=function(ms) if ms==build.BUSY_WAIT_MS then slept=slept+1 end end},
  function() calls=calls+1;return {exit_code=calls==2 and 1 or 0} end)
assert(slept==1 and passes_after_wait==1 and calls==2+4)
-- A writer that keeps the tree broken is reported after the bounded waits.
slept=0
assert(not pcall(build.run,{ue_root='U',quiet=true,fingerprint=function() return 'x' end,
  newest_input=function() return 9900 end,now=function() return 10000 end,sleep_ms=function(ms) if ms==build.BUSY_WAIT_MS then slept=slept+1 end end},
  function() return {exit_code=1} end))
assert(slept==build.MAX_BUSY_RETRIES)
local tick=0
assert(not pcall(build.run,with({ue_root='U',quiet=true,fingerprint=function() tick=tick+1;return tostring(tick) end}),
  function() return {exit_code=0} end))
-- A pass waits for quiet inputs first; the wait is bounded.
local clock,polls=0,0
local quiet_after=build.run({ue_root='U',quiet=true,fingerprint=function() return 'q' end,
  newest_input=function() return 0 end,now=function() return clock end,
  sleep_ms=function(ms) polls=polls+1;clock=clock+ms//1000 end},function() return {exit_code=0} end)
assert(quiet_after==1 and polls==build.START_QUIET_SECONDS*1000//build.START_POLL_MS)

-- Commands: authority, runtime and Unreal, all compiled-content-only release.
local commands=build.commands('R','U')
assert(#commands==4 and table.concat(commands[1].args,' '):find('--bin omobab',1,true))
assert(table.concat(commands[3].args,' '):find('omoba-netem-proxy',1,true))
assert(table.concat(commands[2].args,' '):find('omoba-client-runtime',1,true))
assert(table.concat(commands[4].args,' '):find('build_ue_moba.lua',1,true))
for i=1,2 do assert(table.concat(commands[i].args,' '):find('compiled-content-only --release',1,true)) end
assert(#build.fingerprint('.',{'scripts/tests'})>0)
-- A held UnrealBuildTool mutex is a busy machine, not a build failure.
assert(build.ubt_busy({stdout='A conflicting instance of UnrealBuildTool_Mutex_1 is already running. Result: Failed (ConflictingInstance)'}))
assert(not build.ubt_busy({stdout='error C4430',stderr=''}))
print('consistent build: stable/unstable passes, failure stop and command set passed')
