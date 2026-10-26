package.path='scripts/?.lua;'..package.path
local perf=require('summarize_moba_perf')
local b=require('_bootstrap');local path=b.lib('path')

local function window(component,metric,count,sum,max,duration)
  return ('OM_PERF {"v":1,"component":"%s","metric":"%s","unit":"ns","samples":%d,"count":%d,"sum":%d,"max":%d,"mean":%d,"window_duration_ns":%d}')
    :format(component,metric,count,count,sum,max,sum//count,duration)
end
local root=path.join(b.root,'target','tests','perf-summary-'..os.time())
for _,dir in ipairs({'server','p1','p6'}) do path.mkdir_p(path.join(root,dir)) end
-- Server: two 60-tick windows, 4 ms and 6 ms mean, each spanning exactly one second.
path.write(path.join(root,'server','server.stdout.log'),table.concat({
  window('server','tick_compute',60,240e6,9e6,1e9),window('server','tick_compute',60,360e6,12e6,1e9),
  'retargeted late InputSubmit player=6','observer mismatch team=2',
  '01-01 00:00:00 [INFO] - player_input_tick: pid=1 tick=100541 MoveTo target_raw=(1024324, 0)',
  '01-01 00:00:01 [WARN] - KCP recv_from failed: os error 10054',
  -- Per-phase records: dispatcher 3 ms, projection 1 ms (team 1 detail 0.6 ms).
  'OM_PERF {"phase":"dispatcher","v":1,"component":"server","metric":"tick_phase","count":60,"sum":180000000,"max":9000000,"window_duration_ns":1000000000}',
  'OM_PERF {"phase":"team_projection","v":1,"component":"server","metric":"tick_phase","count":60,"sum":60000000,"max":2000000,"window_duration_ns":1000000000}',
  'OM_PERF {"phase":"team_projection.team1","v":1,"component":"server","metric":"tick_phase","count":60,"sum":36000000,"max":1000000,"window_duration_ns":1000000000}',
  'OM_PERF {"v":1,"component":"server","metric":"system_time","system":"Creep_sys","count":120,"sum":120000000,"window_ticks":60,"window_duration_ns":1000000000}',
  window('server','observer_step',60,30e6,2e6,1e9):gsub('}$',',"team_id":2}')},'\n'),true)
-- p1 renders at 100 fps: a 400 ms load clamp during warmup is ignored, a
-- 150 ms hitch in gameplay is counted (1 of 2 gameplay windows); p6 at 50 fps.
path.write(path.join(root,'p1','ue-p1.log'),table.concat({
  window('unreal','game_frame_interval',60,600e6,400e6,60e9),window('unreal','game_frame_interval',60,600e6,150e6,6e8),
  window('unreal','game_frame_interval',60,600e6,10e6,6e8)},'\n'),true)
path.write(path.join(root,'p6','ue-p6.log'),window('unreal','game_frame_interval',60,1200e6,30e6,12e8),true)
path.write(path.join(root,'p1','runtime-p1.stderr.log'),window('client_runtime','replica_step',60,60e6,2e6,1e9),true)

local run=perf.summarize(root)
assert(math.abs(run.tick.mean_ms-5)<1e-9,'true mean is sum/count, not mean of means')
assert(math.abs(run.tick.p99_ms-6)<1e-9 and math.abs(run.tick.max_ms-12)<1e-9 and math.abs(run.tick.rate_hz-60)<1e-9)
assert(run.clients[1].hitched==1 and run.clients[1].gameplay_windows==2)
assert(#run.clients==2 and math.abs(run.clients[1].fps-100)<1e-9 and math.abs(run.clients[2].fps-50)<1e-9)
assert(run.late_retargeted==1 and run.late_rejected==0 and run.mismatches==1)
assert(run.wsa_10054==1,'only a warning/error line counts as a socket reset, not digits in a tick or sum')
local ok,gates=perf.verdict(run)
local failed={} for _,g in ipairs(gates) do if not g.ok then failed[g.name]=true end end
assert(not ok and failed['ue-p6-fps'] and failed['ue-p1-hitch'] and failed['no-desync'] and not failed['server-tick-mean'] and not failed['server-tick-rate'])
local b=run.breakdown
assert(b.phases[1].name=='dispatcher' and math.abs(b.phases[1].mean_ms-3)<1e-9,'heaviest phase first')
assert(b.systems[1].name=='Creep_sys' and math.abs(b.systems[1].ms_per_tick-2)<1e-9 and b.systems[1].runs_per_tick==2)
assert(b.observer[1].team=='2' and math.abs(b.observer[1].mean_ms-0.5)<1e-9)
local detail=perf.breakdown_markdown(run)
assert(detail:find('summed phases 4.00 ms',1,true) and detail:find('| dispatcher | 3.000',1,true) and detail:find('↳ team_projection.team1',1,true)
  and detail:find('75%',1,true) and detail:find('observer replica step team 2',1,true),detail)
local md=perf.markdown({run})
assert(md:find('5.00／6.00／12.0',1,true) and md:find('100.0／50.0',1,true) and md:find('FAIL: ',1,true),md)
print('moba perf summary: window aggregation, gates and markdown passed')
