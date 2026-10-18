-- Supervised source-rig import, followed by a fresh no-save editor verification.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local importer=require('character_source_import');local engine,id,validate_only
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--validate-only'then validate_only=true
 elseif arg[i]=='--help'then print('tools/lua/lua.exe scripts/import_character_sources.lua --ue-root PATH --run-id FRESH [--validate-only]');return
 else error('unknown character source import argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh run-id required')
local plan=importer.validate(b.root);r.supervise_workflow(source,arg)
if validate_only then print('Validated '..#plan.assets..' FBX sources and included licenses; no Unreal import');return end
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/character-source-import',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local report={schema='omoba.character-source-summary.v1',success=false,run_id=id,authority_games=0,retarget_verified=false,render_verified=false}
local ok,err=xpcall(function()
 local plan_file=p.join(evidence,'plan.json');j.write(plan_file,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local args={project,'-run=OmCharacterSourceImport','-OmCharacterPlan='..plan_file,'-OmCharacterResult='..output,
   '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,label..'-engine.log')}
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),args,{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,label..'-stdout.log'),result.stdout or'',false,true);p.write(p.join(evidence,label..'-stderr.log'),result.stderr or'',false,true)
  assert(result.exit_code==0,label..' commandlet failed: '..tostring(result.exit_code))
  local value=j.read(output);local packages=importer.verify_result(plan,value);return value,packages
 end
 local packages;report.first,packages=once('first');local stamps={}
 for _,package in ipairs(packages)do
  local file=p.join(b.root,'omfue/Content',package:sub(#'/Game/'+1)..'.uasset');assert(p.is_file(file),'imported package missing')
  stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end
 report.second=once('second');assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#plan.assets,'repeat imported or saved')
 for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'repeat changed package: '..file)end
 assert(j.encode(importer.validate(b.root))==j.encode(plan),'source changed during import')
 local registry={schema='omoba.character-source-registry.v1',coordinate_policy=plan.coordinate_policy,
  source_manifest_sha256=plan.manifest_sha256,assets=report.second.assets,package_sha256={},retarget_verified=false,render_verified=false}
 for _,package in ipairs(packages)do
  local file=p.join(b.root,'omfue/Content',package:sub(#'/Game/'+1)..'.uasset');registry.package_sha256[package]=assert(stamps[file]).sha256
 end
 local registry_file=importer.registry(b.root)
 if p.exists(registry_file)then assert(j.encode(j.read(registry_file))==j.encode(registry),'existing owned source registry differs; use a new asset version')
 else j.write(registry_file,registry,false)end
 report.source_registry=registry_file;report.source_registry_sha256=h.sha256(registry_file)
 report.packages=stamps;report.source_manifest_sha256=plan.manifest_sha256;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false)
assert(ok,err);print('Verified independent source rigs and animation clips; second editor saved zero packages; retarget/render remain pending')
