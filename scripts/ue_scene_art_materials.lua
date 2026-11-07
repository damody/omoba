-- Shared master material and per-model instances for the Tripo scene art (OmSceneArtMaterials):
-- one shader map for every model, with the instanced-static-mesh usage the map's HISMs need.
--   tools/lua/lua.exe scripts/ue_scene_art_materials.lua --ue-root D:/UE5.8 --run-id FRESH [--ids a,b,...]
-- Runs the commandlet twice from a binaries snapshot (E116); the second pass must save nothing.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
local M={}
-- The models the map draws as instances (OmWorldBridgeActor scene art).
M.ids={'cliff_straight','cliff_corner','bamboo_tree','broadleaf_tree','fountain_honjin','shop_stall'}

function M.main(args)
 local engine,id,ids;local i=1
 while i<=#args do
  if args[i]=='--ue-root'then i=i+1;engine=assert(args[i])
  elseif args[i]=='--run-id'then i=i+1;id=assert(args[i])
  elseif args[i]=='--ids'then i=i+1;ids=assert(args[i])
  elseif args[i]=='--help'then print('Usage: tools/lua/lua.exe scripts/ue_scene_art_materials.lua --ue-root PATH --run-id FRESH [--ids a,b,...]');return
  else error('unknown argument: '..tostring(args[i]))end;i=i+1
 end
 engine=p.absolute(assert(engine,'--ue-root is required'),b.root)
 assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh --run-id required')
 ids=ids or table.concat(M.ids,',');assert(ids:match('^[%w_,]+$'),'invalid --ids')
 local out=p.join(b.root,'omfue/Saved/Automation/SceneArtMaterials',id);assert(not p.exists(out),'fresh evidence directory required');p.mkdir_p(out)
 local snapshot=require('ue_run_snapshot').prepare(b.root)
 require('ue_binary_preflight').require_ready(engine,snapshot.uproject)
 local function once(label)
  local result_file=p.join(out,label..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{snapshot.uproject,'-run=OmSceneArtMaterials',
   '-ids='..ids,'-result='..result_file,'-unattended','-nop4','-nosplash','-nopause','-NullRHI','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(out,label..'-engine.log')},{cwd=snapshot.project_dir,check=false})
  p.write(p.join(out,label..'-process.log'),(result.stdout or'')..(result.stderr or''),false,true)
  assert(result.exit_code==0,label..' scene art material commandlet failed; see '..out)
  local value=j.read(result_file)
  assert(value.schema=='omoba.scene-art-materials/1'and value.success==true,label..' result is not a success')
  local count=0;for _ in ids:gmatch('[^,]+')do count=count+1 end
  assert(#value.materials==count,label..' handled '..#value.materials..' of '..count..' models')
  return value
 end
 local first=once('first');local second=once('second')
 assert(second.created==0 and second.updated==0 and second.saved==0,'repeat pass changed materials')
 print(('Scene art materials PASS: %d created / %d updated / %d saved, repeat saved 0; SM5 precompile %.0f s then %.0f s; evidence: %s')
  :format(first.created,first.updated,first.saved,first.precompile_seconds or 0,second.precompile_seconds or 0,out))
 return first
end
if ...=='ue_scene_art_materials'then return M end
M.main(arg)
