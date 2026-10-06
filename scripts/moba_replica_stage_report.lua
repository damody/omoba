-- Read-only diagnostic evidence. Never changes or evaluates OM_PERF thresholds.
local json = require('tools.lua.lib.json')
local M = {}
local EXACT_MAX = 9007199254740991
local LINE_MAX, CHUNK = 65536, 8192
M.phases = {'decode_ns','staging_ns','fixed_step_ns','pre_repair_ns',
  'post_step_repair_ns','post_repair_hash_ns','host_finalize_ns'}
local fixed = {
  v=1, component='client_runtime', metric='replica_stage', unit='ns', clock='wall',
  scope='apply_encoded_frame', samples=60, residual_meaning='unattributed_wall_time',
  phases_are='same_outer_slowest_sample', ['not']='independent_phase_maxima',
  staging_covers='preflight_pre_step_injection_baseline_staging',
  pre_repair_covers='restore_allowlist_pre_repair_hash',
  host_finalize_covers='fog_sha_report_bookkeeping',
}
local numbers = {'player_id','team_id','replica_tick','team_sequence','outer_ns','residual_ns'}
local allowed = {}
for k in pairs(fixed) do allowed[k]=true end
for _,k in ipairs(numbers) do allowed[k]=true end
for _,k in ipairs(M.phases) do allowed[k]=true end
local function integer(n, maximum)
  return type(n)=='number' and n>=0 and n<=maximum and n%1==0
end
local function identity(player, team)
  assert(integer(player,4294967295) and player>0, 'invalid diagnostic player_id')
  assert(integer(team,4294967295) and team>0, 'invalid diagnostic team_id')
end

function M.parse(line, player, team)
  identity(player,team)
  local at=line:find('OM_REPLICA_STAGE ',1,true)
  if not at then return nil end
  assert(#line<=LINE_MAX, 'replica stage line exceeds diagnostic buffer')
  local body=line:sub(at+#'OM_REPLICA_STAGE '):gsub('%s+$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$','')
  body=body:gsub('%s+%([%w_:]+%s+%d+%)$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$','')
  local r=json.decode(body)
  assert(type(r)=='table' and r~=json.null, 'stage record is not an object')
  -- Contract v1 is a flat object with canonical ASCII keys. Reject duplicate or
  -- escaped keys rather than accepting the shared JSON decoder's last value.
  local keys, raw_count, decoded_count={},0,0
  for key in body:gmatch('"([%w_]+)"%s*:') do
    assert(allowed[key] and not keys[key], 'unknown or duplicate stage key: '..key)
    keys[key]=true;raw_count=raw_count+1
  end
  for key in pairs(r) do
    assert(allowed[key] and keys[key], 'unknown or noncanonical stage key')
    decoded_count=decoded_count+1
  end
  assert(raw_count==decoded_count, 'non-flat stage record')
  for k,v in pairs(fixed) do assert(r[k]==v, 'stage contract mismatch: '..k) end
  for _,k in ipairs(numbers) do assert(integer(r[k],EXACT_MAX), 'invalid stage integer: '..k) end
  assert(r.player_id==player and r.team_id==team, 'stage log identity mismatch')
  local sum=0
  for _,key in ipairs(M.phases) do
    local value=r[key]
    assert(integer(value,EXACT_MAX), 'invalid phase: '..key)
    assert(value<=r.outer_ns-sum, 'phase sum exceeds same-sample outer')
    sum=sum+value
  end
  assert(r.residual_ns==r.outer_ns-sum, 'residual does not reconcile with outer')
  return r
end

function M.accumulator(player,team)
  identity(player,team)
  local windows, peak, last=0,nil,nil
  return {
    observe=function(_,line)
      local r=M.parse(line,player,team)
      if not r then return end
      -- Input is one runtime process timeline, not concatenated runs/rebases.
      if last then
        assert(r.replica_tick>last.replica_tick and r.team_sequence>last.team_sequence,
          'duplicate or nonadvancing stage timeline; use a separate log per runtime timeline')
      end
      assert(windows<EXACT_MAX//60, 'diagnostic sample counter overflow')
      windows=windows+1;last=r
      if not peak or r.outer_ns>peak.outer_ns then peak=r end
    end,
    finish=function()
      assert(peak, 'no complete replica stage window found')
      local largest, value='residual_ns',peak.residual_ns
      for _,key in ipairs(M.phases) do
        if peak[key]>value then largest,value=key,peak[key] end
      end
      return {schema_version=1,kind='replica-stage-diagnostic',
        scope='completed_windows_only_not_performance_acceptance',
        player_id=player,team_id=team,windows=windows,represented_samples=windows*60,
        peak=peak,largest_wall_bucket=largest,
        caveat='wall time is not CPU attribution; residual cause unknown; partial windows absent'}
    end,
  }
end

-- Share the bounded read/close discipline with linked diagnostic collectors.
-- The callback owns record validation; this function never interprets evidence.
function M.scan_file(filename,observe)
  assert(type(observe)=='function', 'diagnostic observer is required')
  local file=assert(io.open(filename,'rb'))
  local ok,result=pcall(function()
    local pending=''
    while true do
      local chunk,read_error=file:read(CHUNK)
      assert(not read_error, 'diagnostic log read failed: '..tostring(read_error))
      if not chunk then break end
      local data=pending..chunk
      local start=1
      while true do
        local stop=data:find('\n',start,true)
        if not stop then break end
        assert(stop-start<=LINE_MAX, 'log line exceeds diagnostic buffer')
        observe(data:sub(start,stop-1));start=stop+1
      end
      pending=data:sub(start)
      assert(#pending<=LINE_MAX, 'log line exceeds diagnostic buffer')
    end
    if #pending>0 then observe(pending) end
  end)
  file:close()
  if not ok then error(result,0) end
  return result
end

function M.read_file(filename,player,team)
  local acc=M.accumulator(player,team)
  M.scan_file(filename,function(line) acc:observe(line) end)
  return acc:finish()
end

function M.main(args)
  local ok,result=pcall(function()
    assert(#args==3, 'Usage: collect_replica_stage.lua RUNTIME_LOG PLAYER_ID TEAM_ID')
    local player,team=tonumber(args[2]),tonumber(args[3])
    return M.read_file(args[1],player,team)
  end)
  if not ok then io.stderr:write(tostring(result)..'\n');return 1 end
  io.write(json.encode(result)..'\n');return 0
end
return M
