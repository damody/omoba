-- Supervised source-rig import, followed by a fresh no-save editor verification.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local importer=require('character_source_import');local engine,id,validate_only,set,replace_stale
local i=1;while i<=#arg do
 if arg[i]=='--ue-root'then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id'then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--validate-only'then validate_only=true
 -- Re-import a source whose FBX changed (a new hash in its manifest): owned resources of the earlier
 -- import are removed first; resources without this importer's metadata are still refused.
 elseif arg[i]=='--replace-stale'then replace_stale=true
 elseif arg[i]=='--source-set'then i=i+1;set=assert(arg[i]);assert(importer.sets[set],'unknown --source-set')
 elseif arg[i]=='--help'then print('tools/lua/lua.exe scripts/import_character_sources.lua --ue-root PATH --run-id FRESH [--source-set quaternius_v1|tripo_v1|tripo_static_v1] [--validate-only] [--replace-stale]');return
 else error('unknown character source import argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root);assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh run-id required')
importer.validate(b.root,set);r.supervise_workflow(source,arg)
if validate_only then print('Validated '..#importer.validate(b.root,set).assets..' FBX sources and provenance; no Unreal import');return end
-- The commandlet runs from a binaries snapshot (E116); imported packages still land in live Content
-- through the snapshot's junction, and the plan's source paths name the snapshot's own SourceArt.
local snapshot=require('ue_run_snapshot').prepare(b.root)
local project=snapshot.uproject;require('ue_binary_preflight').require_ready(engine,project)
local plan=importer.validate(b.root,set,snapshot.project_dir);plan.replace_stale=replace_stale or nil
local evidence=p.join(b.root,'.ai-collab/character-source-import',id);assert(not p.exists(evidence));p.mkdir_p(evidence)
local report={schema='omoba.character-source-summary.v1',success=false,run_id=id,authority_games=0,retarget_verified=false,render_verified=false}
local ok,err=xpcall(function()
 local plan_file=p.join(evidence,'plan.json');j.write(plan_file,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local args={project,'-run=OmCharacterSourceImport','-OmCharacterPlan='..plan_file,'-OmCharacterResult='..output,
   '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,label..'-engine.log')}
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),args,{cwd=snapshot.project_dir,check=false})
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
 local again=importer.validate(b.root,set,snapshot.project_dir);again.replace_stale=plan.replace_stale
 assert(j.encode(again)==j.encode(plan),'source changed during import')
 -- Rigs whose earlier import this run replaced (--replace-stale) may change their registry row and packages.
 -- With --replace-stale, a rig whose source hash differs from its registered row was re-imported (in this
 -- run or an earlier interrupted one); its row and packages may change. Every other row must stay the same,
 -- except for result fields added since it was registered.
 local replaced={}
 if plan.replace_stale and p.exists(importer.registry(b.root,set))then
  local sha={};for _,a in ipairs(plan.assets)do sha[a.id]=a.sha256 end
  for _,row in ipairs(j.read(importer.registry(b.root,set)).assets or{})do if sha[row.id] and row.source_sha256~=sha[row.id]then replaced[row.id]=true end end
 end
 local function extends(old,new)
  -- Numbers come back through JSON at a different precision than they were registered with.
  if type(old)=='number'then return type(new)=='number'and math.abs(old-new)<=1e-9*math.max(1,math.abs(old))end
  if type(old)~='table'then return old==new end
  if type(new)~='table'then return false end
  if #old~=#new then return false end
  for k,v in pairs(old)do if not extends(v,new[k])then return false end end
  return true
 end
 local function replaced_package(package)for rid in pairs(replaced)do if package:find('/'..rid..'/',1,true)then return true end end;return false end
 local registry={schema='omoba.character-source-registry.v1',coordinate_policy=plan.coordinate_policy,
  source_manifest_sha256=plan.manifest_sha256,assets=report.second.assets,package_sha256={},retarget_verified=false,render_verified=false}
 if plan.source_set then registry.source_set=plan.source_set;registry.source_manifests=plan.manifests;registry.source_manifest_sha256=nil end
 for _,package in ipairs(packages)do
  local file=p.join(b.root,'omfue/Content',package:sub(#'/Game/'+1)..'.uasset');registry.package_sha256[package]=assert(stamps[file]).sha256
 end
 local registry_file=importer.registry(b.root,set)
 if p.exists(registry_file)and not plan.source_set then assert(j.encode(j.read(registry_file))==j.encode(registry),'existing owned source registry differs; use a new asset version')
 elseif p.exists(registry_file)then
  -- A growing generated set may add rigs; every rig already registered must be byte-for-byte unchanged.
  local old=j.read(registry_file);local rows={};for _,row in ipairs(registry.assets)do rows[row.id]=row end
  for _,row in ipairs(old.assets or{})do assert(replaced[row.id] or extends(row,rows[row.id]),'registered rig changed; use a new asset version: '..tostring(row.id))end
  for package,sha in pairs(old.package_sha256 or{})do assert(replaced_package(package) or registry.package_sha256[package]==sha,'registered package changed: '..package)end
  if j.encode(old)~=j.encode(registry)then j.write(registry_file,registry,true)end
 else p.mkdir_p(p.parent(registry_file));j.write(registry_file,registry,false)end
 report.source_registry=registry_file;report.source_registry_sha256=h.sha256(registry_file)
 report.packages=stamps;report.source_manifest_sha256=plan.manifest_sha256;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);j.write(p.join(evidence,'summary.json'),report,false)
assert(ok,err);print('Verified independent source rigs and animation clips; second editor saved zero packages; retarget/render remain pending')
