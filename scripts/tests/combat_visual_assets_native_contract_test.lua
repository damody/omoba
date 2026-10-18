local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local m=require('combat_visual_assets')
local engine,id;local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])else error('unknown native combat contract argument')end;i=i+1
end
engine=p.absolute(assert(engine),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/combat-visual-native-contract',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local report={schema='omoba.combat-visual-native-contract.v1',success=false,authority_games=0,cases={}}
local ok,err=xpcall(function()
 local plan=m.plan();local stamps={}
 for _,row in ipairs(plan.materials)do
  local package=assert(row.object_path:match('^(.-)%.'));local file=p.join(b.root,'omfue/Content',package:sub(7)..'.uasset')
  assert(p.is_file(file),'build owned combat materials before native rejection tests')
  stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end
 local cases={
  {id='foreign-destination',mutate=function(v)v.destination='/Game/Foreign' end},
  {id='unknown-version',mutate=function(v)v.graph_version='unlit_soft_shapes_v999' end},
  {id='duplicate-shape',mutate=function(v)v.materials[2]=j.decode(j.encode(v.materials[1]))end},
  {id='escaped-owned-path',mutate=function(v)v.materials[1].object_path='/Game/Foreign/M_Ring.M_Ring'end},
  {id='mismatched-shape',mutate=function(v)v.materials[1].shape='disc'end},
  {id='missing-material',mutate=function(v)table.remove(v.materials)end},
 }
 for _,case in ipairs(cases)do
  local bad=j.decode(j.encode(plan));case.mutate(bad);local input=p.join(evidence,case.id..'-plan.json');j.write(input,bad,false)
  local output=p.join(evidence,case.id..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmCombatVisualAssets','-OmCombatPlan='..input,
   '-OmCombatResult='..output,'-NullRHI','-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,case.id..'-engine.log')},
   {cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,case.id..'-stdout.log'),result.stdout or'',false,true);p.write(p.join(evidence,case.id..'-stderr.log'),result.stderr or'',false,true)
  local value=j.read(output)
  assert(result.exit_code~=0 and value.schema=='omoba.combat-visual-assets-result.v1'and value.success==false
   and type(value.error)=='string'and value.error~=''and value.saved==0 and value.imported==0 and value.verified==0,'native preflight did not reject before mutation')
  for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'native rejection changed owned package')end
  report.cases[#report.cases+1]={id=case.id,exit_code=result.exit_code,error=value.error,saved=0}
 end
 report.packages=stamps;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err)
if not ok then p.write(p.join(evidence,'errors.md'),'# Combat visual native contract failure\n\n```text\n'..tostring(err)..'\n```\n',false,true)end
j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
print('PASS six actual native preflight rejections; three owned material packages unchanged; zero authority games')
