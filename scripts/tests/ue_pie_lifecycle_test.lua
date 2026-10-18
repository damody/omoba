local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local lifecycle=require('ue_pie_lifecycle')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end

local function empty_args(args)
  assert(type(args)=='table' and next(args)==nil)
end

local function accepted_stop(observed)
  return {success=true,request_accepted=true,transition_observed=observed==true}
end

local function run(options)
  local calls={}
  local retired=0
  local killed=false
  local clock=options.clock or 0
  local alive_checks=0
  local result=lifecycle.await_pie_stop({
    owned_alive=options.owned_alive or function()
      alive_checks=alive_checks+1
      if options.die_at_check and alive_checks>=options.die_at_check then return false end
      if options.alive==false then return false end
      return true
    end,
    call=function(name,args)
      calls[#calls+1]={name=name,args=args}
      empty_args(args)
      if options.fail and options.fail[name] then error(options.fail[name]) end
      if name=='stop_play_in_editor' then return options.stop end
      if name=='get_pie_status' then
        if options.missing_status then return nil end
        local index=0
        for _,call in ipairs(calls) do if call.name==name then index=index+1 end end
        local statuses=assert(options.statuses,'missing status script')
        if index>#statuses then error('unexpected get_pie_status '..index) end
        return statuses[index]
      end
      error('unexpected PIE tool '..tostring(name))
    end,
    now_ms=options.now_ms or function() return clock end,
    sleep_ms=options.sleep_ms or function(ms)
      assert(ms==(options.interval or lifecycle.poll_interval_ms))
      if options.freeze_clock then return end
      clock=clock+ms
    end,
    retire=function()
      retired=retired+1
      killed=true
      if options.retire_error then error(options.retire_error) end
      return options.retire_errors
    end,
    stop_tool='stop_play_in_editor',
    status_tool='get_pie_status',
    deadline_ms=options.deadline_ms,
    poll_interval_ms=options.poll_interval_ms,
  })
  return result,calls,{retired=retired,killed=killed,clock=clock}
end

local function ops_of(result)
  return table.concat(result.operations,',')
end

test('already stopped polls once and retires only after idle status',function()
  local result,calls,side=run({
    stop=accepted_stop(true),
    statuses={{success=true,pie_running=false}},
  })
  assert(result.observed and result.pie_cleanup_error==nil,tostring(result.pie_cleanup_error))
  assert(result.polls==1 and side.retired==1 and #result.cleanup_errors==0)
  assert(ops_of(result)=='owned_alive,stop_play_in_editor,owned_alive,get_pie_status,retire')
  assert(#calls==2 and calls[1].name=='stop_play_in_editor' and calls[2].name=='get_pie_status')
  assert(side.clock==0)
end)

test('legacy positive stop without accepted flag still requires idle evidence',function()
  local result,_,side=run({
    stop={success=true,transition_observed=true},
    statuses={{success=true,pie_running=true},{success=true,pie_running=false}},
  })
  assert(result.observed and result.polls==2 and side.retired==1 and side.clock==250)
end)

test('malformed accepted flag rejects even when legacy success is positive',function()
  for _,value in ipairs{0,'true',{}} do
    local result,_,side=run({stop={success=true,request_accepted=value}})
    assert(not result.observed and result.polls==0 and side.retired==1)
    assert(result.pie_cleanup_error=='PIE stop request was not accepted')
  end
end)

test('async stop stays busy for two reads then becomes idle',function()
  local result,calls,side=run({
    stop=accepted_stop(false),
    statuses={
      {success=true,pie_running=true},
      {success=true,pie_running=true},
      {success=true,pie_running=false},
    },
  })
  assert(result.observed and result.pie_cleanup_error==nil and result.polls==3)
  assert(side.retired==1 and side.clock==500)
  assert(ops_of(result)==table.concat({
    'owned_alive','stop_play_in_editor','owned_alive','get_pie_status','sleep',
    'owned_alive','get_pie_status','sleep','owned_alive','get_pie_status','retire',
  },','))
  assert(calls[1].name=='stop_play_in_editor')
  assert(calls[2].name=='get_pie_status' and calls[3].name=='get_pie_status' and calls[4].name=='get_pie_status')
end)

test('still running at the 10s deadline is not success and kill is not evidence',function()
  local running={success=true,pie_running=true}
  local statuses={}
  for _=1,41 do statuses[#statuses+1]=running end
  local result,_,side=run({stop=accepted_stop(false),statuses=statuses})
  assert(not result.observed)
  assert(result.pie_cleanup_error=='PIE stop was not observed before deadline')
  assert(result.polls==41 and side.clock==10000 and side.killed and side.retired==1)
  assert(ops_of(result):sub(-6)=='retire')
  assert(not ops_of(result):find('sleep,retire',1,true))
  local sleeps=0
  for _,op in ipairs(result.operations) do if op=='sleep' then sleeps=sleeps+1 end end
  assert(sleeps==40)
end)

test('idle arriving after the deadline is not accepted as timely stop',function()
  local reads=0
  local result,_,side=run({
    stop=accepted_stop(false),
    statuses={{success=true,pie_running=false}},
    now_ms=function() reads=reads+1;return reads==1 and 0 or 10000 end,
  })
  assert(not result.observed and result.polls==1 and side.retired==1)
  assert(result.pie_cleanup_error=='PIE stop was not observed before deadline')
end)

test('missing and non-boolean pie status are rejected before another poll',function()
  for _,status in ipairs{
    {success=true},
    {success=true,pie_running=0},
    {success=true,pie_running='false'},
    {success=false,pie_running=false},
  } do
    local result,calls,side=run({stop=accepted_stop(false),statuses={status}})
    assert(not result.observed and result.pie_cleanup_error=='PIE status malformed',tostring(result.pie_cleanup_error))
    assert(result.polls==1 and side.retired==1 and side.clock==0)
    assert(#calls==2)
  end
  local missing,missing_calls,missing_side=run({
    stop=accepted_stop(false),
    missing_status=true,
  })
  assert(not missing.observed and missing.pie_cleanup_error=='PIE status malformed')
  assert(missing.polls==1 and missing_side.retired==1 and #missing_calls==2)
end)

test('owner already retired records the error and still retires',function()
  local result,calls,side=run({alive=false,stop=accepted_stop(true),statuses={{success=true,pie_running=false}}})
  assert(not result.observed)
  assert(result.pie_cleanup_error=='original PIE Editor retired')
  assert(#calls==0 and side.retired==1 and result.polls==0)
  assert(ops_of(result)=='owned_alive,retire')
end)

test('owner exit during the poll is not a successful stop',function()
  local result,calls,side=run({
    die_at_check=3,
    stop=accepted_stop(false),
    statuses={{success=true,pie_running=true},{success=true,pie_running=false}},
  })
  assert(not result.observed and result.pie_cleanup_error=='original PIE Editor retired')
  assert(result.polls==1 and #calls==2 and side.retired==1 and side.clock==250)
  assert(ops_of(result)=='owned_alive,stop_play_in_editor,owned_alive,get_pie_status,sleep,owned_alive,retire')
end)

test('stop and status mcp errors retire the owner without observing idle',function()
  local stop_result,stop_calls,stop_side=run({fail={stop_play_in_editor='owned editor socket closed'}})
  assert(not stop_result.observed and stop_result.polls==0 and stop_side.retired==1)
  assert(stop_result.pie_cleanup_error:find('owned editor socket closed',1,true))
  assert(#stop_calls==1 and stop_calls[1].name=='stop_play_in_editor')
  assert(ops_of(stop_result)=='owned_alive,stop_play_in_editor,retire')

  local status_result,status_calls,status_side=run({
    stop=accepted_stop(false),
    fail={get_pie_status='original PIE Editor retired'},
  })
  assert(not status_result.observed and status_result.polls==1 and status_side.retired==1 and status_side.clock==0)
  assert(status_result.pie_cleanup_error:find('original PIE Editor retired',1,true))
  assert(status_calls[1].name=='stop_play_in_editor' and status_calls[2].name=='get_pie_status')
end)

test('unaccepted stop does not treat transition_observed as idle',function()
  local result,calls,side=run({
    stop={success=true,request_accepted=false,transition_observed=true},
    statuses={{success=true,pie_running=false}},
  })
  assert(not result.observed and result.polls==0 and #calls==1 and side.retired==1)
  assert(result.pie_cleanup_error=='PIE stop request was not accepted')
end)

test('a frozen clock cannot spin and still retires',function()
  local result,_,side=run({
    freeze_clock=true,
    stop=accepted_stop(false),
    statuses={{success=true,pie_running=true},{success=true,pie_running=false}},
  })
  assert(not result.observed and result.polls==1 and side.retired==1 and side.clock==0)
  assert(result.pie_cleanup_error=='PIE stop clock did not advance')
end)

test('retire failure stays separate from an observed stop',function()
  local result,_,side=run({
    stop=accepted_stop(true),
    statuses={{success=true,pie_running=false}},
    retire_error='original PIE child remained after stop',
  })
  assert(result.observed and result.pie_cleanup_error==nil and side.retired==1)
  assert(#result.cleanup_errors==1)
  assert(result.cleanup_errors[1]:find('original PIE child remained after stop',1,true))
end)

test('poll bounds reject before any editor call',function()
  local calls=0
  local function callbacks(extra)
    local spec={
      owned_alive=function() calls=calls+1;return true end,
      call=function() calls=calls+1 end,
      now_ms=function() calls=calls+1;return 0 end,
      sleep_ms=function() calls=calls+1 end,
      retire=function() calls=calls+1;return {} end,
      stop_tool='stop_play_in_editor',
      status_tool='get_pie_status',
    }
    for key,value in pairs(extra) do spec[key]=value end
    return spec
  end
  assert(not pcall(lifecycle.await_pie_stop,callbacks({poll_interval_ms=1000})))
  assert(not pcall(lifecycle.await_pie_stop,callbacks({poll_interval_ms=0})))
  assert(not pcall(lifecycle.await_pie_stop,callbacks({deadline_ms=10001})))
  assert(calls==0)
  assert(lifecycle.deadline_ms==10000 and lifecycle.poll_interval_ms==250)
end)

test('runner wires the fixed clock, original editor, and conditional success',function()
  local runner=path.read(path.join(b.root,'scripts/run_moba_pie_ue.lua'))
  local prepare_at=assert(runner:find('pie_preflight.prepare',1,true))
  local begin_at=assert(runner:find("mcp('begin_play_in_editor'",1,true))
  local await_at=assert(runner:find('lifecycle.await_pie_stop',1,true))
  assert(prepare_at<begin_at and begin_at<await_at)
  assert(runner:find("require('ue_pie_lifecycle')",1,true))
  assert(runner:find('now_ms=time.monotonic_ms',1,true))
  assert(runner:find('sleep_ms=time.sleep_ms',1,true))
  assert(runner:find("stop_tool='stop_play_in_editor'",1,true))
  assert(runner:find("status_tool='get_pie_status'",1,true))
  assert(runner:find("num_clients=1,net_mode='standalone'",1,true))
  assert(runner:find('selection_smoke_hero',1,true))
  assert(runner:find('finish_timeout_seconds',1,true))
  assert(runner:find('report.pie_cleanup_error',1,true))
  assert(runner:find('report.success=ok and report.success and report.cleanup_verified',1,true))
  assert(runner:find('Decision: preserve all evidence and retire only original owned children; no forced winner, fallback or unrelated Editor cleanup.',1,true))
  assert(runner:find('for i=#owned,1,-1 do',1,true))
end)

print(('PIE lifecycle: %d scenarios passed; injected clock and editor, no Editor/MCP/build'):format(count))
