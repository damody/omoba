local original_host=package.loaded['tools.lua.lib.host']
local original_time=package.loaded['tools.lua.lib.time']
local value=101
package.loaded['tools.lua.lib.host']={call=function(operation)
  assert(operation=='monotonic_ms');return {milliseconds=value}
end}
package.loaded['tools.lua.lib.time']=nil
local time=require('tools.lua.lib.time')
assert(time.monotonic_ms()==101)
assert(time.monotonic_ms()==101) -- Coarse counter can repeat; it cannot decrease.
value=117;assert(time.monotonic_ms()==117)
for _,invalid in ipairs({116,-1,1.5,'120'}) do
  value=invalid;assert(not pcall(time.monotonic_ms),'invalid or backwards clock accepted')
end
package.loaded['tools.lua.lib.host']=original_host
package.loaded['tools.lua.lib.time']=original_time
local actual=require('tools.lua.lib.time')
local first=actual.monotonic_ms()
actual.sleep_ms(40)
local second=actual.monotonic_ms()
assert(second>first,'machine clock must advance across separate helper invocations')
print('workflow clock: injected guards and real cross-process uptime passed; no UTC deadline fallback')
