-- J5 performance summary over existing split runs (no new match needed).
-- Reads OM_PERF windows from the authority (tick_compute), each Unreal client
-- (game_frame_interval, presentation_work) and each client runtime
-- (replica_step), plus late-input / 10054 / mismatch / termination counts, and
-- prints one markdown table with PASS/FAIL against the thresholds below.
-- OM_PERF records are 60-sample windows (count, sum, max, mean): the true mean
-- is sum/count; percentiles are over window means (not per-sample).
-- Usage: tools/lua/lua.exe scripts/summarize_moba_perf.lua --run DIR [--run DIR ...] [--output FILE.md]
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local lfs=require('lfs')
local M={}

-- Thresholds (perf-report.md explains each choice).
M.TICK_BUDGET_MS=1000/60          -- 60 Hz simulation step budget
M.TICK_MEAN_LIMIT_MS=8            -- half the budget: headroom for spikes and catch-up
M.TICK_RATE_MIN=57                -- >= 95% of the 60 Hz target is actually achieved
M.UE_FPS_MIN=57                   -- clients are frame-capped at 60: >= 95% of the cap
M.UE_HITCH_MS=100                 -- a frame interval above this is a visible hitch
M.UE_WARMUP_S=60                  -- loading and shader warmup are not gameplay hitches
M.UE_HITCH_WINDOW_PCT=2           -- at most 2% of gameplay windows may contain a hitch

local function percentile(values,pct)
  if #values==0 then return nil end
  local sorted={} for i,v in ipairs(values) do sorted[i]=v end
  table.sort(sorted)
  -- Nearest-rank: the smallest value with at least pct% of windows at or below it.
  return sorted[math.max(1,math.min(#sorted,math.ceil(#sorted*pct/100)))]
end

local function windows(file,component,metric)
  local rows={}
  if not path.is_file(file) then return rows end
  for line in io.lines(file) do
    local body=line:match('OM_PERF (%b{})')
    if body then
      local ok,row=pcall(json.decode,body)
      if ok and row.component==component and row.metric==metric and (row.count or 0)>0 then rows[#rows+1]=row end
    end
  end
  return rows
end

function M.aggregate(rows)
  if #rows==0 then return nil end
  local count,sum,max,duration,means=0,0,0,0,{}
  for _,row in ipairs(rows) do
    count=count+row.count;sum=sum+row.sum;max=math.max(max,row.max or 0)
    duration=duration+(row.window_duration_ns or 0);means[#means+1]=row.sum/row.count
  end
  return {windows=#rows,count=count,mean_ms=sum/count/1e6,max_ms=max/1e6,
    p50_ms=percentile(means,50)/1e6,p99_ms=percentile(means,99)/1e6,
    rate_hz=duration>0 and count/(duration/1e9) or nil,means=means}
end

-- Windows whose max exceeds the hitch limit, after the warmup period.
function M.hitch_windows(rows)
  local elapsed,total,hitched=0,0,0
  for _,row in ipairs(rows) do
    elapsed=elapsed+(row.window_duration_ns or 0)/1e9
    if elapsed>M.UE_WARMUP_S then
      total=total+1
      if (row.max or 0)/1e6>M.UE_HITCH_MS then hitched=hitched+1 end
    end
  end
  return hitched,total
end

-- Every OM_PERF row of one metric (any component), grouped by a key field.
local function grouped(file,metric,key)
  local groups={}
  if not path.is_file(file) then return groups end
  for line in io.lines(file) do
    local body=line:match('OM_PERF (%b{})')
    if body and body:find('"metric":"'..metric..'"',1,true) then
      local ok,row=pcall(json.decode,body)
      if ok and row.metric==metric then
        local name=tostring(row[key] or '?')
        groups[name]=groups[name] or {}
        table.insert(groups[name],row)
      end
    end
  end
  return groups
end

-- Formal tick phases (dotted names are breakdowns), heaviest systems (ms per
-- tick, parallel so they overlap) and observer replica steps per team.
function M.breakdown(server_log)
  local out={phases={},systems={},observer={}}
  for name,rows in pairs(grouped(server_log,'tick_phase','phase')) do
    local agg=M.aggregate(rows);agg.name=name;out.phases[#out.phases+1]=agg
  end
  table.sort(out.phases,function(a,b) return a.mean_ms>b.mean_ms end)
  for name,rows in pairs(grouped(server_log,'system_time','system')) do
    local sum,ticks,count=0,0,0
    for _,row in ipairs(rows) do sum=sum+row.sum;ticks=ticks+(row.window_ticks or 0);count=count+row.count end
    out.systems[#out.systems+1]={name=name,ms_per_tick=ticks>0 and sum/ticks/1e6 or 0,runs_per_tick=ticks>0 and count/ticks or 0}
  end
  table.sort(out.systems,function(a,b) return a.ms_per_tick>b.ms_per_tick end)
  for team,rows in pairs(grouped(server_log,'observer_step','team_id')) do
    local agg=M.aggregate(rows);agg.team=team;out.observer[#out.observer+1]=agg
  end
  table.sort(out.observer,function(a,b) return a.team<b.team end)
  return out
end

function M.breakdown_markdown(run)
  local b=run.breakdown
  if not b or #b.phases==0 then return '_'..run.name..': no tick_phase records (build without per-phase OM_PERF)._' end
  local total=0 for _,p in ipairs(b.phases) do if not p.name:find('.',1,true) then total=total+p.mean_ms end end
  local lines={('**%s** phases (mean ms per tick; share of the summed phases %.2f ms):'):format(run.name,total),'',
    '| phase | mean | p99 window | max | share |','|---|---|---|---|---|'}
  for _,p in ipairs(b.phases) do
    local detail=p.name:find('.',1,true)
    lines[#lines+1]=('| %s%s | %.3f | %.3f | %.1f | %s |'):format(detail and '↳ ' or '',p.name,p.mean_ms,p.p99_ms,p.max_ms,
      detail and '—' or ('%.0f%%'):format(total>0 and p.mean_ms*100/total or 0))
  end
  if #b.systems>0 then
    lines[#lines+1]='';lines[#lines+1]='| system (parallel, overlapping) | ms per tick | runs per tick |';lines[#lines+1]='|---|---|---|'
    for _,sys in ipairs(b.systems) do lines[#lines+1]=('| %s | %.3f | %.2f |'):format(sys.name,sys.ms_per_tick,sys.runs_per_tick) end
  end
  for _,o in ipairs(b.observer) do
    lines[#lines+1]=('\nobserver replica step team %s: mean %.3f ms, p99 window %.3f ms, max %.1f ms'):format(o.team,o.mean_ms,o.p99_ms,o.max_ms)
  end
  return table.concat(lines,'\n')
end

local function count_lines(file,needle)
  local n=0
  if not path.is_file(file) then return 0 end
  for line in io.lines(file) do if line:find(needle,1,true) then n=n+1 end end
  return n
end

-- WSAECONNRESET only on warning/error lines: plain digits '10054' also occur
-- inside OM_PERF nanosecond sums, ticks and coordinates.
function M.count_socket_resets(file)
  local n=0
  if not path.is_file(file) then return 0 end
  for line in io.lines(file) do
    if (line:find('[WARN]',1,true) or line:find('[ERROR]',1,true)) and (line:find('10054',1,true) or line:find('WSAECONNRESET',1,true)) then n=n+1 end
  end
  return n
end

local function players(root)
  local out={}
  for name in lfs.dir(root) do
    local id=name:match('^p(%d+)$')
    if id and path.is_directory(path.join(root,name)) then out[#out+1]=tonumber(id) end
  end
  table.sort(out)
  return out
end

function M.summarize(root)
  local server=path.join(root,'server','server.stdout.log')
  local run={name=root:match('([^/\\]+)$'),root=root,
    tick=M.aggregate(windows(server,'server','tick_compute')),
    late_rejected=count_lines(server,'rejected late InputSubmit'),late_retargeted=count_lines(server,'retargeted late InputSubmit'),
    wsa_10054=M.count_socket_resets(server),mismatches=count_lines(server,'observer mismatch'),
    terminated=count_lines(server,'secure match terminated'),clients={},breakdown=M.breakdown(server)}
  for _,id in ipairs(players(root)) do
    local dir=path.join(root,'p'..id)
    local ue=path.join(dir,'ue-p'..id..'.log')
    local runtime=path.join(dir,'runtime-p'..id..'.stderr.log')
    local frame_rows=windows(ue,'unreal','game_frame_interval')
    local frame=M.aggregate(frame_rows)
    local hitched,gameplay=M.hitch_windows(frame_rows)
    -- E078: a split without --autopilot leaves the human heroes idle, which
    -- lightens hero_command and is not comparable with an autopilot baseline.
    local trace=path.join(dir,'autopilot-p'..id..'.jsonl')
    run.clients[#run.clients+1]={player=id,driven=path.is_file(trace) and (lfs.attributes(trace,'size') or 0)>0,frame=frame,hitched=hitched,gameplay_windows=gameplay,work=M.aggregate(windows(ue,'unreal','presentation_work')),
      replica=M.aggregate(windows(runtime,'client_runtime','replica_step')),
      fps=frame and 1000/frame.mean_ms or nil}
  end
  return run
end

function M.verdict(run)
  local gates={}
  local function gate(name,ok,detail) gates[#gates+1]={name=name,ok=ok,detail=detail} end
  local t=run.tick
  if t then
    gate('server-tick-mean',t.mean_ms<=M.TICK_MEAN_LIMIT_MS,('%.2f ms (limit %.1f)'):format(t.mean_ms,M.TICK_MEAN_LIMIT_MS))
    gate('server-tick-p99-window',t.p99_ms<=M.TICK_BUDGET_MS,('%.2f ms (budget %.1f)'):format(t.p99_ms,M.TICK_BUDGET_MS))
    gate('server-tick-rate',(t.rate_hz or 0)>=M.TICK_RATE_MIN,('%.1f Hz (min %d)'):format(t.rate_hz or 0,M.TICK_RATE_MIN))
  end
  for _,c in ipairs(run.clients) do
    if c.frame then
      gate('ue-p'..c.player..'-fps',c.fps>=M.UE_FPS_MIN,('%.1f fps (min %d)'):format(c.fps,M.UE_FPS_MIN))
      gate('ue-p'..c.player..'-hitch',c.hitched*100<=c.gameplay_windows*M.UE_HITCH_WINDOW_PCT,
        ('%d of %d windows after warmup have a frame > %d ms (limit %d%%)'):format(c.hitched,c.gameplay_windows,M.UE_HITCH_MS,M.UE_HITCH_WINDOW_PCT))
    end
  end
  gate('no-late-rejection',run.late_rejected==0,tostring(run.late_rejected))
  gate('no-desync',run.mismatches==0 and run.terminated==0,('mismatch %d, terminated %d'):format(run.mismatches,run.terminated))
  local ok=true for _,g in ipairs(gates) do ok=ok and g.ok end
  return ok,gates
end

local function fmt(v,pattern) return v and pattern:format(v) or '—' end

function M.markdown(runs)
  local lines={'| run | server tick mean／p99 window／max (ms) | tick rate (Hz) | UE fps (p1／p6) | UE hitch windows (p1／p6) | replica step mean (ms) | late rejected／retargeted | 10054 | mismatch／terminated | verdict |',
    '|---|---|---|---|---|---|---|---|---|---|'}
  for _,run in ipairs(runs) do
    local t=run.tick
    local fps,hitch,replica={},{},{}
    for _,c in ipairs(run.clients) do
      fps[#fps+1]=fmt(c.fps,'%.1f');hitch[#hitch+1]=c.frame and ('%d/%d'):format(c.hitched,c.gameplay_windows) or '—';replica[#replica+1]=fmt(c.replica and c.replica.mean_ms,'%.2f')
    end
    local ok,gates=M.verdict(run)
    local failed={} for _,g in ipairs(gates) do if not g.ok then failed[#failed+1]=g.name end end
    local idle={} for _,c in ipairs(run.clients) do if not c.driven then idle[#idle+1]='p'..c.player end end
    local note=#idle>0 and (' (no autopilot trace: '..table.concat(idle,'／')..'; not comparable with autopilot runs)') or ''
    lines[#lines+1]=('| %s | %s | %s | %s | %s | %s | %d／%d | %d | %d／%d | %s |'):format(run.name,
      t and ('%.2f／%.2f／%.1f'):format(t.mean_ms,t.p99_ms,t.max_ms) or '—',fmt(t and t.rate_hz,'%.1f'),
      table.concat(fps,'／'),table.concat(hitch,'／'),table.concat(replica,'／'),run.late_rejected,run.late_retargeted,run.wsa_10054,
      run.mismatches,run.terminated,(ok and 'PASS' or ('FAIL: '..table.concat(failed,', ')))..note)
  end
  return table.concat(lines,'\n')
end

if arg and arg[0] and arg[0]:match('summarize_moba_perf%.lua$') then
  local runs,output={},nil
  local i=1
  while i<=#arg do
    if arg[i]=='--run' then runs[#runs+1]=M.summarize(path.absolute(assert(arg[i+1],'--run requires a directory'),b.root));i=i+2
    elseif arg[i]=='--output' then output=assert(arg[i+1],'--output requires a file');i=i+2
    else error('unknown argument: '..arg[i]) end
  end
  assert(#runs>0,'Usage: tools/lua/lua.exe scripts/summarize_moba_perf.lua --run DIR [--run DIR ...] [--output FILE.md]')
  local table_md=M.markdown(runs)
  for _,run in ipairs(runs) do table_md=table_md..'\n\n'..M.breakdown_markdown(run) end
  print(table_md)
  if output then path.write(output,table_md..'\n',true) end
end
return M
