local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local j,t=b.lib('json'),b.lib('time')
assert(#arg==2,'requires mode and summary path')
print('Intermediate stage exit=0');io.stdout:flush()
t.sleep_ms(200)
if arg[1]=='fail'then j.write(arg[2],{success=true},false);error('deliberate later stage failure after successful summary',0)end
if arg[1]=='missing'then return end
assert(arg[1]=='pass');j.write(arg[2],{success=true},false)
