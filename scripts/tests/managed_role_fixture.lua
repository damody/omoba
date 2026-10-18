-- Neutral processes exercise the production role owner, never a server, runtime or Unreal simulation.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,t=b.lib('path'),b.lib('json'),b.lib('time')
local mode=arg[1]
if mode=='server' or mode=='runtime' then
 if mode=='runtime' then print(('client-runtime ready player_id=%s team_id=%s neutral-fixture'):format(arg[2],arg[3]));io.stdout:flush()end
 t.sleep_ms(30000);error('neutral lifetime exceeded cleanup deadline')
end
assert(mode=='renderer','unknown neutral mode')
local locator=assert(arg[3]):match('^%-om%-next%-match%-control%-hex=(.*)$')
assert(locator and #locator%2==0)
local file=locator:gsub('..',function(pair)return string.char(tonumber(pair,16))end)
local capability=j.read(file);local folder=p.parent(file)
local request={}
for _,key in ipairs({'session_id','generation','catalog_identity_hash','content_hash','map_catalog_hash','player_id'})do request[key]=capability[key]end
request.schema='omoba-managed-match-request/1';request.hud_source_epoch=7;request.match_phase=2;request.action='ready'
local behavior=arg[2]
if behavior=='manual' then os.exit(0,true)end
if behavior=='crash' then os.exit(3,true)end
if behavior=='cancel' then request.action='cancel';j.write(p.join(folder,'cancel.json'),request,false);os.exit(0,true)end
j.write(p.join(folder,'ready.json'),request,false)
assert(t.poll(15000,20,function()
 if p.is_file(p.join(folder,'abort.json'))then os.exit(0,true)end
 if not p.is_file(p.join(folder,'resolution.json'))then return nil end
 local ok,resolution=pcall(j.read,p.join(folder,'resolution.json'))
 return ok and resolution
end),'neutral coordinator response timed out')
local retired={};for _,key in ipairs({'session_id','generation','catalog_identity_hash','content_hash','map_catalog_hash','player_id','hud_source_epoch'})do retired[key]=request[key]end
retired.schema='omoba-managed-match-retired/1';retired.inputs_retired=true;retired.hud_retired=true;retired.runtime_stopped=true
j.write(p.join(folder,'retired.json'),retired,false)
os.exit(0,true)
