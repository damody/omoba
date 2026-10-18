local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local m=require('character_rigid_parts')
local engine,id,validate_only;local i=1;while i<=#arg do if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])elseif arg[i]=='--validate-only'then validate_only=true else error('unknown rigid parts argument')end;i=i+1 end
engine=p.absolute(assert(engine),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'));local plan=m.validate(b.root);r.supervise_workflow(script,arg)
if validate_only then print('Validated '..#plan.assets..' weighted bone subtree/pivot definitions and source packages');return end
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/character-rigid-parts',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local report={schema='omoba.rigid-character-part-summary.v1',success=false,run_id=id,authority_games=0,render_verified=false}
local ok,err=xpcall(function()
 local input=p.join(evidence,'plan.json');j.write(input,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local process=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmRigidCharacterPart','-OmPartPlan='..input,
   '-OmPartResult='..output,'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,label..'-stdout.log'),process.stdout or'',false,true);p.write(p.join(evidence,label..'-stderr.log'),process.stderr or'',false,true)
  assert(process.exit_code==0,label..' native rigid part extraction failed: '..tostring(process.exit_code));local value=j.read(output);return value,m.verify_result(plan,value)
 end
 local packages;report.first,packages=once('first');local stamps={}
 for _,package in ipairs(packages)do local file=p.join(b.root,'omfue/Content',package:sub(7)..'.uasset');assert(p.is_file(file));stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end
 report.second=once('second');assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#packages,'repeat rigid part build wrote packages')
 for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'repeat rigid part build changed package')end
 assert(j.encode(m.validate(b.root))==j.encode(plan),'source changed during extraction');report.packages=stamps;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
print('Verified rigid weighted parts, pivot-local geometry, material slots and independent no-save reload; render remains pending')
