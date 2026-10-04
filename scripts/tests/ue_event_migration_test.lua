local source=debug.getinfo(1,'S').source:sub(2)
package.path=source:match('^(.*)[/\\]tests[/\\]')..'/?.lua;'..package.path
local b=require('_bootstrap');local json=b.lib('json');local m=require('ue_event_migration')
local rule={legacy_event='Old',replacement_event='New',replacement_payload='Generic',sink_function='Apply',
 required_fields={'State'},allowed_calls={'Apply'},allowed_ops={'event','call','break_struct'},max_removed_nodes=3}
local function pin(name,to) return {name=name,direction='output',connected_to=to and {{node_id=to,pin='input'}} or {}} end
local snapshot={ok=true,truncated=false,node_count=6,total_node_count=6,nodes={
 {id='old',op='event',detail={name='Old'},content_hash='1',exec_to={{to='old_call'}},pins={pin('then','old_call'),pin('Payload','old_break')}},
 {id='old_break',op='break_struct',detail={struct='OldPayload'},content_hash='2',pins={pin('State','old_call')}},
 {id='old_call',op='call',detail={['function']='Apply'},content_hash='3',pins={}},
 {id='new',op='event',detail={name='New'},content_hash='4',exec_to={{to='new_call'}},pins={pin('then','new_call'),pin('Payload','new_break')}},
 {id='new_break',op='break_struct',detail={struct='Generic'},content_hash='5',pins={pin('State','new_call')}},
 {id='new_call',op='call',detail={['function']='Apply'},content_hash='6',pins={}},
}}
local count=0
local function rejects(fn) count=count+1;assert(not pcall(fn),'expected rejection') end
local plan=m.plan(snapshot,rule);assert(#plan.removed==3 and #plan.patch==2 and plan.root=='old');count=count+1
local function clone() return json.decode(json.encode(snapshot)) end
local after=clone();table.remove(after.nodes,3);table.remove(after.nodes,2);table.remove(after.nodes,1)
after.node_count,after.total_node_count=3,3;m.verify(plan,after,rule);assert(m.plan(after,rule).already_migrated);count=count+1
local shared=clone();shared.nodes[1].pins[#shared.nodes[1].pins+1]=pin('shared','new_call');rejects(function() m.plan(shared,rule) end)
local side_effect=clone();side_effect.nodes[3].detail['function']='DestroyActor';rejects(function() m.plan(side_effect,rule) end)
local missing=clone();missing.nodes[5].pins={};rejects(function() m.plan(missing,rule) end)
local truncated=clone();truncated.truncated=true;rejects(function() m.plan(truncated,rule) end)
local altered=json.decode(json.encode(after));altered.nodes[1].content_hash='changed';rejects(function() m.verify(plan,altered,rule) end)
rejects(function() m.patch_result({ok=true,applied=false,results={{status='rejected',reason='CAS'}}},1,true) end)
rejects(function() m.patch_result({ok=true,applied=false,results={{status='ok'}}},1,false) end)
m.patch_result({ok=true,results={{status='ok',would_apply=true}}},1,true);count=count+1
rule.field_bindings={State={sink_pin='input'}}
assert(m.plan(snapshot,rule));count=count+1
rule.field_bindings.State.sink_pin='wrong'
rejects(function() m.plan(snapshot,rule) end)
local rewired=json.decode(json.encode(after));rewired.nodes[1].pins={}
rejects(function() m.verify(plan,rewired,{legacy_event='Old'}) end)
if #arg==3 then
  local function detail(file) return json.decode(json.read(file).content[1].text) end
  local actual_rule=json.read(arg[3]);local before=detail(arg[1]);local after_actual=detail(arg[2])
  m.verify(m.plan(before,actual_rule),after_actual,actual_rule)
  print('[ue-event-migration-tests] actual preserved graph verified')
end
print('[ue-event-migration-tests] passed '..count..' cases')
