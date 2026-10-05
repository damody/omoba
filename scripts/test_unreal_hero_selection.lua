-- One feature confirmation, not a match/full acceptance run.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,time=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
local launch=require('moba_role_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','unreal-selection-tests'))
local reserved
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then reserved=p;break end end
assert(reserved)
local options=launch.options({'--profile','debug','--output',path.join(reserved,'match')})
options.selection_smoke_hero=arg[1] or 'training_ranger'
local editor=launch.editor(options)
local deadline=time.monotonic_ms()+120000
local bounded_time={monotonic_ms=time.monotonic_ms,sleep_ms=function(ms)
  assert(time.monotonic_ms()<deadline,'selection feature confirmation timed out')
  time.sleep_ms(ms)
end}
local ok,err=xpcall(function()
  launch.interactive_select(options,editor,process,bounded_time)
  local plan=json.read(options.recipe)
  local chosen
  for _,p in ipairs(plan.players) do if p.player_id==1 then chosen=p.hero end end
  assert(chosen==options.selection_smoke_hero,'final plan did not use the selected hero')
  local log=path.read(options.output..'-selection/ue.log')
  local count=0
  for kind,id in log:gmatch('OM_SELECTION_BUTTON player=1 kind=(%w+) pressed=1 released=1 request=(%d+) submitted_once=1') do
    count=count+1
    assert(kind==({'select','lock','finalize'})[count] and tonumber(id)==count,'unexpected selection pointer sequence')
  end
  assert(count==3,'expected exactly three native pointer callbacks')
  assert(not log:find('Started bridge runtime:',1,true),'gameplay bridge started during selection')
  json.write(path.join(reserved,'report.json'),{scope='selection-feature-confirmation',success=true,
    hero=chosen,native_pointer_callbacks=count,final_plan=options.recipe,gameplay_started=false})
  print('PASS Unreal selection -> Rust select/lock/finalize -> final plan; no gameplay match started')
  print(reserved)
end,debug.traceback)
if not ok then
  path.write(path.join(reserved,'errors.md'),'# Unreal selection feature error\n\n'..tostring(err)..
    '\n\nDecision: preserve evidence and fix the current feature; do not launch a match or rerun full acceptance.\n')
  error(err,0)
end
