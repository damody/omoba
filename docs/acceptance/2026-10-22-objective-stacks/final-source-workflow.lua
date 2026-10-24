local r=require('tools.lua.lib.process')
r.supervise_workflow('D:/omoba/omfue/Saved/BuildTemp/objective-stacks-source-v3.lua',arg)
local p=require('tools.lua.lib.path');local j=require('tools.lua.lib.json');local t=require('tools.lua.lib.time')
local receiptfile='omfue/Saved/BuildTemp/objective-stacks-rust-v3.receipt.json'
while not p.exists(receiptfile)do t.sleep_ms(1000)end
local receipt=j.read(receiptfile)
assert(receipt.success and receipt.retired and not r.owned_alive(receipt.owner) and not r.owned_alive(receipt.monitor))
local capturefile='omfue/Saved/BuildTemp/objective-stacks-source-capture-v2.receipt.json'
while not p.exists(capturefile)do t.sleep_ms(1000)end
local capture=j.read(capturefile)
assert(capture.success and capture.retired and not r.owned_alive(capture.owner) and not r.owned_alive(capture.monitor))
local value=r.run('D:/omoba/tools/lua/lua.exe',{'omfue/Saved/BuildTemp/wall-guidance-source.lua','check',
 'omfue/Saved/BuildTemp/objective-stacks-source-before-v2.json',
 'omfue/Saved/BuildTemp/objective-stacks-source-verified-v3.json'},{cwd='D:/omoba',check=false})
print(value.stdout or '');print(value.stderr or '');print('Completed source check exit='..value.exit_code)
assert(value.exit_code==0)
