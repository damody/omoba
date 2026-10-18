-- Reject an entire malformed rigid-part batch before processing or saving outputs.
local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs');local m=require('character_rigid_parts')
local engine,id;local i=1;while i<=#arg do if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])else error('unknown rigid contract argument')end;i=i+1 end
engine=p.absolute(assert(engine),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local plan=m.validate(b.root);local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/character-rigid-native-contract',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local stamps={};local count=0
local function walk(dir)for name in lfs.dir(dir)do if name~='.'and name~='..'then local file=p.join(dir,name)
 if lfs.attributes(file,'mode')=='directory'then walk(file)elseif name:match('%.uasset$')then count=count+1;stamps[file]={sha256=h.sha256(file),mtime=lfs.attributes(file,'modification')}end
end end end
for _,name in ipairs({'QuaterniusV2','RigidPartsV1'})do walk(p.join(b.root,'omfue/Content/Omoba/Characters',name))end
local files={'scripts/character_rigid_parts.lua','scripts/build_character_rigid_parts.lua','scripts/tests/character_rigid_parts_test.lua',
 'scripts/tests/character_rigid_parts_native_contract_test.lua','omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmRigidCharacterPartCommandlet.cpp',
 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmRigidCharacterPartCommandlet.h'}
local producers={};for _,file in ipairs(files)do producers[file]=h.sha256(p.join(b.root,file))end
local report={success=false,run_id=id,authority_games=0,package_count=count,producer_sources=producers,cases={}}
local ok,err=xpcall(function()
 local cases={
  {name='stale-source-registry',change=function(v)v.source_registry_sha256=string.rep('0',64)end},
  {name='last-duplicate-id',change=function(v)v.assets[#v.assets].id=v.assets[1].id end},
  {name='last-invalid-source',change=function(v)v.assets[#v.assets].source_id='unknown_source'end},
  {name='last-invalid-pivot',change=function(v)v.assets[#v.assets].pivot_bone='UnknownBone'end},
  {name='last-traversal-id',change=function(v)v.assets[#v.assets].id='../escape'end},
  {name='last-out-of-range-weight',change=function(v)v.assets[#v.assets].min_weight=1.5 end},
 }
 for _,case in ipairs(cases)do
  local value=j.decode(j.encode(plan));case.change(value);local input=p.join(evidence,case.name..'-plan.json');local output=p.join(evidence,case.name..'-result.json');j.write(input,value,false)
  local process=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmRigidCharacterPart','-OmPartPlan='..input,
   '-OmPartResult='..output,'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,case.name..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,case.name..'-stdout.log'),process.stdout or'',false,true);p.write(p.join(evidence,case.name..'-stderr.log'),process.stderr or'',false,true)
  local result=j.read(output);assert(process.exit_code==1 and result.success==false and result.error~=''and result.imported==0 and result.verified==0 and result.saved==0 and #result.assets==0,'malformed batch reached asset processing')
  for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and lfs.attributes(file,'modification')==stamp.mtime,'native rejection changed package')end
  report.cases[#report.cases+1]={name=case.name,error=result.error,exit_code=process.exit_code,saved=0}
 end
 assert(j.encode(m.validate(b.root))==j.encode(plan),'source changed during native rejection tests')
 for file,sha in pairs(producers)do assert(h.sha256(p.join(b.root,file))==sha,'producer changed during native rejection tests')end
 report.success=true;report.packages=stamps
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false);assert(ok,err)
print('PASS '..#report.cases..' actual native rigid-part preflight rejections; '..count..' packages unchanged')
