-- Linked, read-only diagnostics; never evaluates or changes performance gates.
local stage=require('moba_replica_stage_report')
local json=require('tools.lua.lib.json')
local M={}
local MAX=9007199254740991
M.phases={'preparation_ns','dispatcher_ns','runtime_event_boundary_ns',
  'hero_command_clears_ns','tower_spawns_ns','tower_sells_ns','tower_target_priorities_ns',
  'item_uses_ns','ability_upgrades_ns','ability_casts_ns','moves_ns','pre_script_outcomes_ns',
  'tower_upgrades_ns','tower_ability_casts_ns','tower_ability_scheduler_ns',
  'tower_ability_callbacks_ns','script_dispatch_ns','creep_wave_ns',
  'post_script_outcomes_ns','finalization_ns'}
local fixed={v=1,component='client_runtime',metric='fixed_step_detail',unit='ns',
  scope='fixed_step',phases_are='same_outer_slowest_sample',samples=60,
  cpu_scope='all_process_threads_kernel_plus_user',cpu_not='wait_time_or_phase_attribution'}
local numbers={'player_id','team_id','replica_tick','team_sequence','outer_ns','fixed_step_ns','residual_ns'}
local allowed={process_cpu_ns=true}
for k in pairs(fixed) do allowed[k]=true end
for _,k in ipairs(numbers) do allowed[k]=true end
for _,k in ipairs(M.phases) do allowed[k]=true end
local function integer(n) return type(n)=='number' and n>=0 and n<=MAX and n%1==0 end

function M.parse(line,player,team)
  assert(integer(player) and player>0 and player<=4294967295,'invalid diagnostic player_id')
  assert(integer(team) and team>0 and team<=4294967295,'invalid diagnostic team_id')
  local at=line:find('OM_FIXED_STEP ',1,true)
  if not at then return nil end
  assert(#line<=65536,'fixed-step line exceeds diagnostic buffer')
  local body=line:sub(at+#'OM_FIXED_STEP '):gsub('%s+$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$','')
  body=body:gsub('%s+%([%w_:]+%s+%d+%)$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$','')
  local r=json.decode(body)
  assert(type(r)=='table' and r~=json.null,'fixed-step record is not an object')
  local keys,raw_count,decoded_count={},0,0
  for key in body:gmatch('"([%w_]+)"%s*:') do
    assert(allowed[key] and not keys[key],'unknown or duplicate fixed-step key: '..key)
    keys[key]=true;raw_count=raw_count+1
  end
  for key in pairs(r) do
    assert(allowed[key] and keys[key],'unknown or noncanonical fixed-step key')
    decoded_count=decoded_count+1
  end
  assert(raw_count==decoded_count,'non-flat fixed-step record')
  for k,v in pairs(fixed) do assert(r[k]==v,'fixed-step contract mismatch: '..k) end
  for _,k in ipairs(numbers) do assert(integer(r[k]),'invalid fixed-step integer: '..k) end
  assert(r.player_id==player and r.team_id==team,'fixed-step log identity mismatch')
  assert(r.fixed_step_ns<=r.outer_ns,'fixed-step exceeds linked outer duration')
  assert(r.process_cpu_ns==json.null or integer(r.process_cpu_ns),'invalid/missing process CPU duration')
  local sum=0
  for _,k in ipairs(M.phases) do
    assert(integer(r[k]),'invalid fixed-step phase: '..k)
    assert(r[k]<=r.fixed_step_ns-sum,'fixed-step phase sum exceeds wall duration')
    sum=sum+r[k]
  end
  assert(r.residual_ns==r.fixed_step_ns-sum,'fixed-step residual does not reconcile')
  return r
end

function M.accumulator(player,team)
  local stages=stage.accumulator(player,team)
  local pending,peak,linked=nil,nil,0
  return {
    observe=function(_,line)
      local s=stage.parse(line,player,team)
      if s then
        assert(not pending,'missing fixed-step detail before next stage window')
        stages:observe(line);pending=s;return
      end
      local d=M.parse(line,player,team)
      if not d then return end
      assert(pending,'fixed-step detail has no preceding stage window')
      for _,k in ipairs({'player_id','team_id','replica_tick','team_sequence','samples','outer_ns','fixed_step_ns'}) do
        assert(d[k]==pending[k],'fixed-step link mismatch: '..k)
      end
      linked=linked+1
      if not peak or d.outer_ns>peak.outer_ns then peak=d end
      pending=nil
    end,
    finish=function()
      assert(not pending,'missing fixed-step detail at end of log')
      local summary=stages:finish()
      assert(peak and linked==summary.windows,'incomplete fixed-step window linkage')
      assert(peak.replica_tick==summary.peak.replica_tick and peak.team_sequence==summary.peak.team_sequence,
        'fixed-step peak does not match whole outer peak')
      local largest,value='residual_ns',peak.residual_ns
      for _,k in ipairs(M.phases) do if peak[k]>value then largest,value=k,peak[k] end end
      summary.kind='linked-fixed-step-diagnostic'
      summary.fixed_step_peak=peak
      summary.largest_fixed_step_wall_bucket=largest
      summary.caveat='completed windows only; CPU is all process threads, may exceed wall, not wait time or phase attribution; partial windows absent'
      return summary
    end,
  }
end

function M.read_file(filename,player,team)
  local acc=M.accumulator(player,team)
  stage.scan_file(filename,function(line) acc:observe(line) end)
  return acc:finish()
end

function M.main(args)
  local ok,result=pcall(function()
    assert(#args==3,'Usage: collect_fixed_step.lua RUNTIME_LOG PLAYER_ID TEAM_ID')
    return M.read_file(args[1],tonumber(args[2]),tonumber(args[3]))
  end)
  if not ok then io.stderr:write(tostring(result)..'\n');return 1 end
  io.write(json.encode(result)..'\n');return 0
end
return M
