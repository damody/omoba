-- Supervised build-time recolour of generated hero bodies (scripts/character_recolor.lua), followed by
-- a second, verify-only commandlet pass that must save nothing new.
--   tools/lua/lua.exe scripts/recolor_characters.lua --ue-root D:/UE5.8 --run-id FRESH [--source-set SET] [--validate-only]
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local recolor=require('character_recolor');local engine,id,validate_only,set_id
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--validate-only'then validate_only=true
 elseif arg[i]=='--source-set'then i=i+1;set_id=assert(arg[i])
 elseif arg[i]=='--help'then print('tools/lua/lua.exe scripts/recolor_characters.lua --ue-root PATH --run-id FRESH [--source-set tripo_v1|tripo_static_v1] [--validate-only]');return
 else error('unknown recolor argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh run-id required')
local jobs=recolor.jobs(b.root,nil,set_id);assert(#jobs>=1,'no hero manifest declares a recolor block')
if validate_only then print('Validated '..#jobs..' recolor jobs; no texture written, no Unreal import');return end
r.supervise_workflow(source,arg)
local evidence=p.join(b.root,'.ai-collab/character-recolor',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local plan={schema=recolor.schema,variants={}}
for _,job in ipairs(jobs)do plan.variants[#plan.variants+1]=recolor.prepare(job)end
local plan_file=p.join(evidence,'plan.json');j.write(plan_file,plan,true)
-- The commandlet runs from a binaries snapshot (E116); variant packages land in live Content.
local snapshot=require('ue_run_snapshot').prepare(b.root)
local project=snapshot.uproject;require('ue_binary_preflight').require_ready(engine,project)
local report={schema='omoba.character-recolor-summary.v1',success=false,run_id=id}
local ok,err=xpcall(function()
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmCharacterRecolor',
   '-OmRecolorPlan='..plan_file,'-OmRecolorResult='..output,'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(evidence,label..'-engine.log')},{cwd=snapshot.project_dir,check=false})
  p.write(p.join(evidence,label..'-stdout.log'),result.stdout or'',false,true)
  assert(result.exit_code==0,label..' recolor commandlet failed: '..tostring(result.exit_code))
  return recolor.verify_result(plan,j.read(output))
 end
 report.first=once('first');report.second=once('second')
 assert(report.second.imported==0 and report.second.verified==#plan.variants,'repeat recolor imported again')
 report.materials={};for _,row in ipairs(plan.variants)do report.materials[row.id]=recolor.material_path(row)end
 report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
for hero,path in pairs(report.materials)do print('recolor '..hero..': '..path)end
