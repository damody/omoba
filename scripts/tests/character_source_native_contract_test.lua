-- Integration gate: malformed LAST job must prevent a valid first job from writing.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs')
local a=require('character_source_import');local engine,id
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 else error('unknown native character contract argument')end;i=i+1
end
engine=p.absolute(assert(engine),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'))
r.supervise_workflow(source,arg)
local plan=a.validate(b.root);assert(#plan.assets>=2,'integration gate requires at least two real source rigs')
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/character-source-native-contract',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local directory=p.join(b.root,'omfue/Content/Omoba/Characters/QuaterniusV2');local stamps={};local package_count=0
local function walk(dir)
 for name in lfs.dir(dir)do if name~='.'and name~='..'then
  local file=p.join(dir,name);if lfs.attributes(file,'mode')=='directory'then walk(file)
  elseif name:match('%.uasset$')then stamps[file]={sha256=h.sha256(file),mtime=lfs.attributes(file,'modification')};package_count=package_count+1 end
 end end
end
walk(directory);local expected=0;for _,asset in ipairs(plan.assets)do expected=expected+2+asset.animations end
assert(package_count==expected,'import real source rigs before running this gate')
local producer_files={
 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmCharacterSourceImportCommandlet.h',
 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmCharacterSourceImportCommandlet.cpp',
 'omfue/Plugins/OmRuntime/Source/OmEditor/OmEditor.Build.cs',
 'scripts/character_source_import.lua','scripts/import_character_sources.lua',
 'scripts/tests/character_source_import_test.lua','scripts/tests/character_source_native_contract_test.lua'}
local producer={};for _,file in ipairs(producer_files)do producer[file]=h.sha256(p.join(b.root,file))end
local probe_id='probe_native_contract';local probe=p.join(b.root,'omfue/SourceArt/Characters/QuaterniusV1',probe_id..'.fbx')
local probe_destination=p.join(directory,probe_id);assert(not p.exists(probe)and not p.exists(probe_destination))
local report={success=false,run_id=id,cases={},package_count=package_count,producer_sources=producer,authority_games=0}
local ok,err=xpcall(function()
 p.write(probe,assert(p.read(plan.assets[1].source,true)),false,true)
 local mutations={
  {name='last-source-sha',change=function(value)value.sha256=string.rep('0',64)end},
  {name='last-source-path',change=function(value)value.source=plan.assets[1].source end},
  {name='last-fractional-bones',change=function(value)value.bones=65.25 end},
  {name='last-duplicate-id',change=function(value)value.id=probe_id;value.source=probe end},
 }
 for _,case in ipairs(mutations)do
  local value=j.decode(j.encode(plan));local first=j.decode(j.encode(plan.assets[1]));first.id=probe_id;first.source=probe
  local last=j.decode(j.encode(plan.assets[2]));case.change(last);value.assets={first,last}
  local input=p.join(evidence,case.name..'-plan.json');local output=p.join(evidence,case.name..'-result.json');j.write(input,value,false)
  local process=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmCharacterSourceImport',
   '-OmCharacterPlan='..input,'-OmCharacterResult='..output,'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(evidence,case.name..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,case.name..'-stdout.log'),process.stdout or'',false,true);p.write(p.join(evidence,case.name..'-stderr.log'),process.stderr or'',false,true)
  local result=j.read(output);assert(process.exit_code==1 and result.success==false and result.error~=''
   and result.imported==0 and result.saved==0 and #result.assets==0,'invalid last job was not rejected before first import')
  assert(not p.exists(probe_destination),'valid first probe wrote despite invalid last job')
  for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and lfs.attributes(file,'modification')==stamp.mtime,'native rejection changed existing package')end
  report.cases[#report.cases+1]={name=case.name,exit_code=process.exit_code,error=result.error,saved=0,probe_not_created=true}
 end
 assert(j.encode(a.validate(b.root))==j.encode(plan),'real sources changed during negative gates')
 for file,sha in pairs(producer)do assert(h.sha256(p.join(b.root,file))==sha,'producer changed during native gate')end
 report.success=true;report.packages=stamps
end,debug.traceback)
if p.exists(probe)then assert(os.remove(probe),'failed to retire owned probe source')end
report.probe_source_retired=not p.exists(probe);report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false)
assert(ok,err);print('PASS '..#report.cases..' real native preflight rejections; '..package_count..' owned packages unchanged and probe retired')
