local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
assert(#arg==1,'requires one acceptance spec JSON')
r.supervise_workflow(source,arg)
local file=p.absolute(arg[1],b.root);assert(p.is_file(file)and p.attributes(file).size<=1048576,'bounded acceptance spec required')
local receipt=require('owned_acceptance').run(j.read(file),r,t)
print('Actual owned acceptance success='..tostring(receipt.success)..' Windows exit='..tostring(receipt.exit_code))
os.exit(receipt.success and 0 or 1,true)
