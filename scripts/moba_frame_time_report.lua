-- Frame-time percentiles of one Unreal renderer log (P0 #5, E144).
--
--   tools\lua\lua.exe scripts\moba_frame_time_report.lua UE_LOG [--warmup-seconds N] [--json OUT]
--
-- The renderer logs OM_PERF_HIST lines: cumulative histograms (0.25 ms buckets) of the
-- game-thread frame interval and of the presentation work, every 600 frames and when the
-- presentation ends. Subtracting the last line at or before the end of warm-up from the last
-- line gives the distribution of the measured span only, without loading and shader warm-up.
-- The measurement conditions come from the same log (OS, CPU, GPU, command line: resolution,
-- RHI, off-screen or a real window), because off-screen numbers differ from a real window: no
-- swap-chain present, no desktop composition, no vsync wait.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.METRICS={'game_frame_interval','presentation_work'}

local function seconds_of(stamp)
  -- [2026.10.27-04.13.01:196]
  local y,mo,d,h,mi,s,ms=stamp:match('^%[(%d+)%.(%d+)%.(%d+)%-(%d+)%.(%d+)%.(%d+):(%d+)%]')
  if not y then return nil end
  return os.time({year=tonumber(y),month=tonumber(mo),day=tonumber(d),hour=tonumber(h),min=tonumber(mi),sec=tonumber(s)})+tonumber(ms)/1000
end

local function counts_of(text)
  local counts={}
  for index,count in tostring(text or ''):gmatch('(%d+):(%d+)') do counts[tonumber(index)]=tonumber(count) end
  return counts
end

-- {metric={{t=seconds, samples, max, bucket_ns, buckets, counts}, ...}, conditions={...}}
function M.parse(text,json)
  json=json or require('_bootstrap').lib('json')
  local result={series={},conditions={}}
  for _,metric in ipairs(M.METRICS) do result.series[metric]={} end
  for line in text:gmatch('[^\r\n]+') do
    local body=line:match('OM_PERF_HIST (%b{})')
    if body then
      local ok,value=pcall(json.decode,body)
      if ok and type(value)=='table' and result.series[value.metric] then
        local list=result.series[value.metric]
        list[#list+1]={t=seconds_of(line),samples=value.samples,max=value.max,bucket_ns=value.bucket_ns,
          buckets=value.buckets,counts=counts_of(value.counts)}
      end
    end
    local os_name,cpu,gpu=line:match('LogInit: OS: (.-), CPU: (.-), GPU: (.+)$')
    if os_name then result.conditions.os=os_name;result.conditions.cpu=cpu;result.conditions.gpu=gpu end
    local command=line:match('LogInit: Command Line: (.+)$')
    if command then
      local c=result.conditions
      c.command_line=command
      c.off_screen=command:find('-RenderOffScreen',1,true)~=nil
      c.rhi=command:match('%-(d3d1[12])') or command:match('%-(vulkan)') or 'default'
      local x,y=command:match('%-ResX=(%d+)%s+%-ResY=(%d+)')
      c.resolution=x and (x..'x'..y) or nil
      c.forced_resolution=command:match('r%.SetRes (%d+x%d+)')
      c.windowed=command:find('-windowed',1,true)~=nil
      c.frame_rate_cap=command:match('t%.MaxFPS[ =](%d+)')
    end
  end
  return result
end

-- Distribution between two cumulative lines (BASE may be nil: from the start).
function M.span(base,last)
  local counts,samples={},0
  for index,count in pairs(last.counts) do
    local value=count-((base and base.counts[index]) or 0)
    if value>0 then counts[index]=value;samples=samples+value end
  end
  return {counts=counts,samples=samples,bucket_ns=last.bucket_ns,buckets=last.buckets,max_upper_bound=last.max}
end

-- Upper edge of the bucket holding the sample at FRACTION; nil when that sample overflowed.
function M.percentile_ns(span,fraction)
  if span.samples==0 then return nil end
  local rank=math.max(1,math.ceil(fraction*span.samples))
  local seen=0
  for index=0,span.buckets-1 do
    seen=seen+(span.counts[index] or 0)
    if seen>=rank then return (index+1)*span.bucket_ns end
  end
  return nil
end

function M.report(text,options,json)
  options=options or {}
  local warmup=options.warmup_seconds or 30
  local parsed=M.parse(text,json)
  local out={schema='omoba.frame-time-report.v1',warmup_seconds=warmup,conditions=parsed.conditions,metrics={},
    percentile_is='upper edge of a 0.25 ms bucket (so at most 0.25 ms high); overflow is > buckets * bucket_ns'}
  for _,metric in ipairs(M.METRICS) do
    local list=parsed.series[metric]
    if #list>0 then
      local first,last=list[1],list[#list]
      local base
      for _,line in ipairs(list) do
        if first.t and line.t and line.t<=first.t+warmup then base=line end
      end
      if base==last then base=nil end -- a run shorter than warm-up: report everything, and say so
      local span=M.span(base,last)
      local function ms(fraction) local ns=M.percentile_ns(span,fraction);return ns and ns/1e6 or nil end
      local row={samples=span.samples,p50_ms=ms(0.50),p95_ms=ms(0.95),p99_ms=ms(0.99),
        max_ms_upper_bound=span.max_upper_bound and span.max_upper_bound/1e6,
        span_seconds=(base and base.t and last.t) and (last.t-base.t) or (first.t and last.t and last.t-first.t),
        warmup_excluded=base~=nil}
      local overflow=span.counts[span.buckets] or 0
      row.over_250ms=overflow
      if row.span_seconds and row.span_seconds>0 then row.mean_fps=span.samples/row.span_seconds end
      out.metrics[metric]=row
    end
  end
  return out
end

function M.format(report)
  local c=report.conditions
  local lines={
    ('conditions: %s | CPU %s | GPU %s'):format(c.os or '?',c.cpu or '?',c.gpu or '?'),
    ('  resolution %s%s, RHI %s, %s, frame cap %s'):format(c.resolution or '?',
      c.forced_resolution and (' (r.SetRes '..c.forced_resolution..')') or '',c.rhi or '?',
      c.off_screen and 'OFF-SCREEN (no present/vsync/composition; not a real window)' or 'real window',
      c.frame_rate_cap or 'engine default'),
  }
  for _,metric in ipairs(M.METRICS) do
    local r=report.metrics[metric]
    if r then
      lines[#lines+1]=('%s: samples %d over %.0f s%s | p50 %s ms p95 %s ms p99 %s ms | max <= %.1f ms | >250 ms %d%s'):format(
        metric,r.samples,r.span_seconds or 0,r.warmup_excluded and (' after '..report.warmup_seconds..' s warm-up') or ' (no warm-up excluded)',
        r.p50_ms and ('%.2f'):format(r.p50_ms) or '>250',r.p95_ms and ('%.2f'):format(r.p95_ms) or '>250',
        r.p99_ms and ('%.2f'):format(r.p99_ms) or '>250',r.max_ms_upper_bound or 0,r.over_250ms,
        r.mean_fps and (' | mean %.1f fps'):format(r.mean_fps) or '')
    else
      lines[#lines+1]=metric..': no OM_PERF_HIST lines (renderer built before E144?)'
    end
  end
  return table.concat(lines,'\n')
end

function M.main(argv)
  local b=require('_bootstrap')
  local path,json=b.lib('path'),b.lib('json')
  local file,options,out=nil,{},nil
  local i=1
  while i<=#argv do
    local v=argv[i]
    if v=='--warmup-seconds' then i=i+1;options.warmup_seconds=assert(tonumber(argv[i]),'--warmup-seconds needs a number')
    elseif v=='--json' then i=i+1;out=assert(argv[i],'--json needs a path')
    elseif v=='--help' or v=='-h' then print('Usage: tools/lua/lua.exe scripts/moba_frame_time_report.lua UE_LOG [--warmup-seconds N] [--json OUT]');return
    else assert(not file,'one UE log');file=v end
    i=i+1
  end
  local report=M.report(path.read(assert(file,'UE log path required')),options,json)
  report.log=path.absolute(file,b.root)
  if out then path.write(out,json.encode(report),true) end
  print(M.format(report))
  return report
end

if not pcall(debug.getlocal,4,1) then M.main(arg) end
return M
