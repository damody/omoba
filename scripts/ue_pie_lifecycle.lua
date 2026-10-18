-- Bounded stop for the original owned PIE Editor.
-- Callbacks are injected. Production passes the fixed time module's monotonic_ms and sleep_ms.
-- An accepted stop is not an observed stop. Retiring processes is not stop evidence.
local M={}
M.deadline_ms=10000
M.poll_interval_ms=250

local function clock_reading(now)
  return type(now)=='number' and now==now and now~=math.huge and now~=-math.huge
end

local function copy_errors(value)
  local errors={}
  if type(value)=='table' then
    for _,item in ipairs(value) do errors[#errors+1]=tostring(item) end
  elseif value~=nil then
    errors[1]=tostring(value)
  end
  return errors
end

function M.await_pie_stop(spec)
  assert(type(spec)=='table','PIE stop requires callbacks')
  assert(type(spec.owned_alive)=='function','PIE stop requires owned_alive')
  assert(type(spec.call)=='function','PIE stop requires call')
  assert(type(spec.now_ms)=='function','PIE stop requires now_ms')
  assert(type(spec.sleep_ms)=='function','PIE stop requires sleep_ms')
  assert(type(spec.retire)=='function','PIE stop requires retire')
  local stop_tool=spec.stop_tool or 'stop_play_in_editor'
  local status_tool=spec.status_tool or 'get_pie_status'
  assert(stop_tool=='stop_play_in_editor' and status_tool=='get_pie_status',
    'PIE stop polls the original owned Editor')
  local interval=spec.poll_interval_ms or M.poll_interval_ms
  local deadline_ms=spec.deadline_ms or M.deadline_ms
  assert(type(interval)=='number' and interval>=1 and interval<=M.poll_interval_ms and interval%1==0,
    'PIE stop poll interval must be an integer from 1 to 250 ms')
  assert(type(deadline_ms)=='number' and deadline_ms>=interval and deadline_ms<=M.deadline_ms and deadline_ms%1==0,
    'PIE stop deadline must be an integer from the poll interval through 10000 ms')

  local operations={}
  local function record(op) operations[#operations+1]=op end
  local observed=false
  local pie_cleanup_error
  local polls=0
  local function note(message)
    if pie_cleanup_error==nil then pie_cleanup_error=message end
  end

  local work_ok,work_err=pcall(function()
    record('owned_alive')
    if spec.owned_alive()~=true then
      note('original PIE Editor retired')
      return
    end
    record(stop_tool)
    local stop_ok,stop_result=pcall(spec.call,stop_tool,{})
    if not stop_ok then
      note(tostring(stop_result))
      return
    end
    -- Legacy native positive replies lack request_accepted. They still need idle
    -- evidence from this same Editor; an explicit negative or malformed flag rejects.
    if type(stop_result)~='table' or stop_result.success~=true or stop_result.ok==false
      or (stop_result.request_accepted~=nil and stop_result.request_accepted~=true) then
      note('PIE stop request was not accepted')
      return
    end
    local started=spec.now_ms()
    if not clock_reading(started) then
      note('PIE stop clock malformed')
      return
    end
    local deadline=started+deadline_ms
    while true do
      record('owned_alive')
      if spec.owned_alive()~=true then
        note('original PIE Editor retired')
        return
      end
      record(status_tool)
      local status_ok,status=pcall(spec.call,status_tool,{})
      polls=polls+1
      if not status_ok then
        note(tostring(status))
        return
      end
      if type(status)~='table' or status.success==false or status.ok==false or type(status.pie_running)~='boolean' then
        note('PIE status malformed')
        return
      end
      local now=spec.now_ms()
      if not clock_reading(now) then
        note('PIE stop clock malformed')
        return
      end
      if now>=deadline then
        note('PIE stop was not observed before deadline')
        return
      end
      if status.pie_running==false then
        observed=true
        return
      end
      record('sleep')
      spec.sleep_ms(interval)
      local after=spec.now_ms()
      if not clock_reading(after) or after<=now then
        note('PIE stop clock did not advance')
        return
      end
    end
  end)
  if not work_ok then note(tostring(work_err)) end

  record('retire')
  local retire_ok,retire_result=pcall(spec.retire)
  local cleanup_errors
  if not retire_ok then cleanup_errors={tostring(retire_result)}
  else cleanup_errors=copy_errors(retire_result) end
  return {
    observed=observed,
    pie_cleanup_error=pie_cleanup_error,
    cleanup_errors=cleanup_errors,
    operations=operations,
    polls=polls,
  }
end

return M
