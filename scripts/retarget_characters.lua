-- Supervised retarget plus independent editor reload and no-save verification.
local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local model=require('character_retarget');local engine,id,targets,validate_only
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--targets'then i=i+1;targets={};for v in assert(arg[i]):gmatch('[^,]+')do targets[#targets+1]=v end
 elseif arg[i]=='--validate-only'then validate_only=true
 else error('unknown retarget argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh run-id required')
local plan,registry=model.validate(b.root,targets);r.supervise_workflow(script,arg)
if validate_only then print('Validated source bytes, licenses, packages and '..#plan.targets..' target rigs');return end
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/character-retarget',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local report={schema='omoba.character-retarget-summary.v1',success=false,run_id=id,authority_games=0,render_verified=false}
local ok,err=xpcall(function()
 local plan_file=p.join(evidence,'plan.json');j.write(plan_file,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmCharacterRetarget',
   '-OmRetargetPlan='..plan_file,'-OmRetargetResult='..output,'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(evidence,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,label..'-stdout.log'),result.stdout or'',false,true);p.write(p.join(evidence,label..'-stderr.log'),result.stderr or'',false,true)
  assert(result.exit_code==0,label..' native retarget failed: '..tostring(result.exit_code))
  local value=j.read(output);return value,model.verify_result(plan,registry,value)
 end
 local packages;report.first,packages=once('first');local stamps={}
 for _,package in ipairs(packages)do local file=p.join(b.root,'omfue/Content',package:sub(7)..'.uasset')
  assert(p.is_file(file));stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end
 report.second=once('second');assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#plan.targets,'repeat retarget wrote assets')
 for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'repeat retarget changed package')end
 assert(j.encode(model.validate(b.root,targets))==j.encode(plan),'source changed during retarget')
 report.packages=stamps;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
print('Native retarget and independent no-save reload verified for '..#plan.targets..' targets; render remains pending')
