local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local launch=require('moba_role_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','selection-handoff-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local baseline=launch.load_recipe(launch.options({}))
local function reply()
  return {protocol_version=1,admitted_player_id=1,error=json.null,
    selection={catalog_data_hash='testhash',ready=true,finalized=true},
    plan=launch.select_heroes(baseline,{[1]='training_ranger'})}
end
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end

test('finalized handoff changes only the bound human hero',function()
  local value=reply()
  local selected=launch.selection_result(value,baseline,1,'testhash')
  assert(selected.players[1].hero=='training_ranger')
  assert(baseline.players[1].hero~='training_ranger')
  for _,mutate in ipairs({
    function(r) r.protocol_version=2 end,
    function(r) r.admitted_player_id=2 end,
    function(r) r.error='rejected' end,
    function(r) r.selection.ready=false end,
    function(r) r.selection.finalized=false end,
    function(r) r.selection.catalog_data_hash='wrong' end,
    function(r) r.plan.players[2].hero='training_ranger' end,
    function(r) r.plan.players[1].team_id=2 end,
    function(r) r.plan.think_hz=60 end,
  }) do local r=reply();mutate(r);rejects(function() launch.selection_result(r,baseline,1,'testhash') end) end
end)

local function execute(name,mode)
  local options=launch.options({'--interactive-selection','--profile','debug','--hero','1=training_vanguard',
    '--output',path.join(root,name)})
  if mode=='completion-timeout' then options.selection_timeout_seconds=1 end
  local elapsed=0
  local slept,spawned,stopped=false,false,false
  local fake={run=function(_,args)
    assert(args[1]=='--lock-plan')
    local candidate=json.read(args[2])
    assert(candidate.players[1].hero=='training_vanguard')
    return {stdout=json.encode({scope='host-prepared-selection',plan=candidate,
      selection={catalog_data_hash='testhash'}})}
  end,spawn=function(_,args)
    spawned=true
    local result
    assert(args[4]=='-om-hero-selection')
    for _,a in ipairs(args) do
      assert(not a:find('om-presentation=',1,true),'gameplay runtime started during selection')
      result=result or a:match('^%-om%-selection%-result=(.*)$')
      local log=a:match('^%-abslog=(.*)$')
      if log and mode~='old' then path.write(log,'OM_SELECTION_READY player=1 protocol=1\n') end
    end
    assert(result)
    if mode~='cancel' and mode~='old' and mode~='completion-timeout' then
      local r=reply()
      if mode=='tamper' then r.plan.players[2].bot=false end
      path.write(result,json.encode(r))
    end
    return 123
  end,inspect=function() return ((mode=='old' or mode=='completion-timeout') and not stopped) or not slept end,
    wait=function() assert(slept);return true end,
    stop=function() assert(mode=='old' or mode=='completion-timeout','must not stop an unrelated or exited process');stopped=true end}
  local ok=pcall(launch.interactive_select,options,'mock-editor.exe',fake,
    {sleep_ms=function(ms) assert(ms==250);slept=true;elapsed=elapsed+ms end,
      monotonic_ms=function() return slept and mode=='old' and 50000 or elapsed end})
  assert(spawned and slept)
  if mode=='success' then
    assert(ok and next(options.hero_selections)==nil)
    assert(path.is_file(options.recipe))
    assert(json.read(options.recipe).players[1].hero=='training_ranger')
  else
    assert(not ok)
    assert(path.is_file(options.output..'-selection/errors.md'))
    assert(not path.exists(options.output),'cancelled selection created match output')
    if mode=='completion-timeout' then
      assert(stopped and elapsed==1000)
      assert(path.read(options.output..'-selection/errors.md'):find('selection completion timed out',1,true))
    end
  end
end
test('selection process waits for exit before handoff and clears CLI overrides',function() execute('success','success') end)
test('window cancellation never starts or auto-locks a match',function() execute('cancel','cancel') end)
test('tampered roster fails closed and records an error',function() execute('tamper','tamper') end)
test('old renderer cannot hang forever or bypass selection handshake',function() execute('old','old') end)
test('ready renderer without finalization is bounded and never launches gameplay',function()
  execute('completion-timeout','completion-timeout')
end)
test('prepare-only and interactive mode cannot be combined',function()
  rejects(function() launch.options({'--prepare-only','--interactive-selection'}) end)
end)
print(('selection handoff functionality: %d/%d passed; mocked renderer, no match started'):format(total,total))
