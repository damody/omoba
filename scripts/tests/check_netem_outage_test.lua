package.path='scripts/?.lua;'..package.path
local check=require('check_netem_outage')
local b=require('_bootstrap');local path,json=b.lib('path'),b.lib('json')

local function fixture(name,runtime1,runtime6,server,outages)
  local root=path.join(b.root,'target','tests','netem-outage',name)
  for _,dir in ipairs({'netem','server','p1','p6'}) do path.mkdir_p(path.join(root,dir)) end
  path.write(path.join(root,'netem','proxy-evidence.json'),json.encode({schema_version=2,status='PASS',outages=outages}),true)
  path.write(path.join(root,'server','match-plan.json'),json.encode({players={{player_id=1,team_id=1,bot=false},{player_id=2,team_id=1,bot=true},{player_id=6,team_id=2,bot=false}}}),true)
  path.write(path.join(root,'server','server.stdout.log'),server,true)
  path.write(path.join(root,'p1','runtime-p1.stderr.log'),runtime1,true)
  path.write(path.join(root,'p6','runtime-p6.stderr.log'),runtime6,true)
  return root
end
local windows={
  {team_id=1,scheduled_at_ms=90000,scheduled_duration_ms=10000,actual_start_ms=1,actual_end_ms=2,dropped_client_to_server=9,dropped_server_to_client=600},
  {team_id=2,scheduled_at_ms=420000,scheduled_duration_ms=40000,actual_start_ms=3,dropped_client_to_server=30,dropped_server_to_client=1800},
}
local good=fixture('good','authority link stalled\nauthority link recovered\n','authority link stalled\nauthority link lost: no replica progress\n','KCP link lost session=x\n',windows)
local ok,gates=check.check(good)
assert(ok,json.encode(gates))
-- A desync, a missing recovery or a missing server verdict each fail the run.
assert(not check.check(fixture('desync','authority link stalled\nauthority link recovered\n','authority link stalled\nauthority link lost\n','KCP link lost\nobserver mismatch team=1\n',windows)))
assert(not check.check(fixture('stuck','authority link stalled\n','authority link stalled\nauthority link lost\n','KCP link lost\n',windows)))
assert(not check.check(fixture('silent-server','authority link stalled\nauthority link recovered\n','authority link stalled\nauthority link lost\n','',windows)))
print('netem outage verdict: gates passed')
