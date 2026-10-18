local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local m=require('combat_visual_assets');local engine,id,validate_only
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--validate-only'then validate_only=true
 else error('unknown combat visual assets argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'),'--run-id required')
local plan=m.plan();r.supervise_workflow(script,arg)
if validate_only then print('Original combat material plan ready; native import/render remain pending');return end
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/combat-visual-assets',id);assert(not p.exists(evidence),'fresh evidence required');p.mkdir_p(evidence)
local report={schema='omoba.combat-visual-assets-summary.v1',success=false,authority_games=0,render_verified=false}
local ok,err=xpcall(function()
 local input=p.join(evidence,'plan.json');j.write(input,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmCombatVisualAssets','-OmCombatPlan='..input,
   '-OmCombatResult='..output,'-AllowCommandletRendering','-d3d12','-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(evidence,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,label..'-stdout.log'),result.stdout or'',false,true);p.write(p.join(evidence,label..'-stderr.log'),result.stderr or'',false,true)
  assert(result.exit_code==0,'combat visual assets native commandlet failed: '..tostring(result.exit_code))
  local value=j.read(output);return value,m.verify_result(plan,value)
 end
 local packages;report.first,packages=once('first');local stamps={}
 for _,package in ipairs(packages)do
  local file=p.join(b.root,'omfue/Content',package:sub(7)..'.uasset');assert(p.is_file(file))
  stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end
 report.second=once('second');assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#packages,'idempotent run wrote packages')
 for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'repeat run changed material package')end
 report.packages=stamps;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err)
if not ok then p.write(p.join(evidence,'errors.md'),'# Combat visual asset workflow failure\n\n```text\n'..tostring(err)..'\n```\n',false,true)end
j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
print('Verified three original combat materials in two independent Editors; render acceptance remains pending')
