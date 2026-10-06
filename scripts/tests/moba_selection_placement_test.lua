local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local placement=require('moba_selection_placement')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
test('default all local sorted copy',function()
  local humans={6,1,3}
  local selected=placement.local_players({},humans)
  assert(table.concat(selected,',')=='1,3,6' and table.concat(humans,',')=='6,1,3')
end)
test('explicit seats do not mutate or drop remote roster',function()
  local humans,requested={6,1,3},{[6]=true,[1]=true}
  local selected=placement.local_players({local_players=requested},humans)
  assert(table.concat(selected,',')=='1,6' and #humans==3 and requested[6] and requested[1])
end)
test('invalid or ambiguous seats rejected',function()
  for _,humans in ipairs({{}, {1,1}, {0}, {4294967296}, {1.5}, {'1'}}) do
    rejects(function() placement.local_players({},humans) end)
  end
  rejects(function() placement.local_players({local_players={[2]=true}},{1}) end)
  rejects(function() placement.local_players({local_players={[1]=false}},{1}) end)
end)
test('default host args retain existing loopback contract',function()
  local args=placement.host_args({},'candidate','output')
  assert(#args==3 and args[1]=='--selection-host' and args[2]=='candidate' and args[3]=='output')
end)
test('explicit bind requests Rust ephemeral port only',function()
  local args=placement.host_args({selection_bind='192.0.2.10'},'candidate','output')
  assert(#args==4 and args[3]=='192.0.2.10:0' and args[4]=='output')
  for _,ip in ipairs({'0.0.0.0','224.0.0.1','192.0.02.1','host','::1','127.0.0.1:12'}) do
    rejects(function() placement.host_args({selection_bind=ip},'candidate','output') end)
  end
end)
test('readiness matches exact requested interface',function()
  assert(placement.ready_address({},'127.0.0.1:1')=='127.0.0.1:1')
  assert(placement.ready_address({selection_bind='192.0.2.10'},'192.0.2.10:65535')=='192.0.2.10:65535')
end)
test('readiness wrong interface or noncanonical port rejected',function()
  for _,address in ipairs({'127.0.0.1:0','127.0.0.1:65536','127.0.0.1:01',
    '127.0.0.1:1 extra','127.0.0.1:1\n','127.0.0.2:1','host:1','0.0.0.0:1','::1:1'}) do
    rejects(function() placement.ready_address({},address) end)
  end
end)
print(('selection placement: %d groups passed; pure tables, no processes or network'):format(count))
