local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local process=b.lib('process')
assert(#arg==1,'requires one private spawn.json')
process.supervise_workflow(source,arg)
require('moba_session_process').run_worker(arg[1],process)
