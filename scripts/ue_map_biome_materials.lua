local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs')
local M={}
local producer='omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmMapBiomeMaterialsCommandlet.cpp'
local content='omfue/Content/Omoba/MapBiomesV1'
local receipt='omfue/Saved/BuildTemp/map-biome-materials.json'
local ids={'jungle','bank','river'}
local textures={grass='omfue/Content/Omoba/WorldSurfacesV1/T_grass.uasset',stone='omfue/Content/Omoba/WorldSurfacesV1/T_stone.uasset'}
local function current(root)
 local state={producer_sha256=h.sha256(p.join(root,producer)),materials={},textures={}}
 for _,id in ipairs(ids)do state.materials[id]=h.sha256(p.join(root,content,'M_'..id..'.uasset'))end
 for id,file in pairs(textures)do state.textures[id]=h.sha256(p.join(root,file))end
 return state
end
local function provenance(root)
 local value=j.read(p.join(root,content,'provenance.json'));assert(value.schema=='omoba.map-biome-material-provenance/1'and #value.materials==3,'invalid biome provenance')
 local seen={}
 for _,row in ipairs(value.materials)do
  assert((row.id=='jungle'or row.id=='bank'or row.id=='river')and not seen[row.id],'invalid biome material ID');seen[row.id]=true
  local file=content..'/M_'..row.id..'.uasset';assert(row.package_file==file,'invalid biome package path')
  assert(h.sha256(p.join(root,file))==row.sha256,'managed biome material changed: '..row.id)
 end
 return value
end
local function matches(a,c)
 if not a or a.producer_sha256~=c.producer_sha256 then return false end
 for _,id in ipairs(ids)do if not a.materials or a.materials[id]~=c.materials[id]then return false end end
 for id in pairs(textures)do if not a.textures or a.textures[id]~=c.textures[id]then return false end end
 return true
end
function M.ready(root)
 local ok,err=pcall(function()
  local created=provenance(root);local c=current(root)
  if p.exists(p.join(root,receipt))then
   local readable,verified=pcall(j.read,p.join(root,receipt))
   if readable and verified.schema=='omoba.map-biome-verification/1'and matches(verified,c)then return end
  end
  assert(created.producer_sha256==c.producer_sha256,'biome producer changed; rebuild and verify materials')
  local original={};for _,row in ipairs(created.world_textures or{})do original[row.package_file]=row.sha256 end
  for id,file in pairs(textures)do assert(original[file]==c.textures[id],'biome texture changed; rebuild and verify materials')end
 end)
 if ok then return true end;return false,tostring(err)
end
function M.require_ready(root)
 local ok,err=M.ready(root);assert(ok,'map biome assets are missing, stale or modified: '..tostring(err));return true
end
function M.main(args)
 local engine,id,ensure,check_only;local i=1
 while i<=#args do
  if args[i]=='--ue-root'then i=i+1;engine=assert(args[i])
  elseif args[i]=='--run-id'then i=i+1;id=assert(args[i])
  elseif args[i]=='--ensure'then ensure=true
  elseif args[i]=='--check-only'then check_only=true
  elseif args[i]=='--help'then print('Usage: tools/lua/lua.exe scripts/ue_map_biome_materials.lua --ue-root PATH [--run-id UNIQUE] [--ensure|--check-only]');return
  else error('unknown argument: '..tostring(args[i]))end;i=i+1
 end
 if check_only then M.require_ready(b.root);print('Map biome source/package SHA checks passed');return end
 if ensure and not id and M.ready(b.root)then print('Map biome materials already match verified source/package SHAs');return end
 engine=p.absolute(assert(engine,'--ue-root is required'),b.root)
 id=id or(os.date('!%Y%m%dT%H%M%SZ')..'-'..math.random(100000,999999));assert(id:match('^[%w_-]+$')and #id<=74)
 local out=p.join(b.root,'omfue/Saved/Automation/MapBiomeMaterials',id);assert(not p.exists(out),'fresh material evidence required');p.mkdir_p(out)
 local report={success=false,authority_games=0,authority_ack=false,run_id=id}
 local ok,err=xpcall(function()
  local target=p.join(b.root,content);p.mkdir_p(target)
  if p.exists(p.join(target,'provenance.json'))then provenance(b.root)end
  local original={};local folder=p.join(b.root,'omfue/Content/Omoba/WorldSurfacesV1')
  for name in lfs.dir(folder)do if name:match('%.uasset$')then local file=p.join(folder,name)
   original[#original+1]={name=name,path=file,sha256=h.sha256(file),mtime=lfs.attributes(file,'modification')}
  end end;table.sort(original,function(a,c)return a.name<c.name end);assert(#original>0)
  report.original_packages=original
  local before={};for _,name in ipairs(ids)do local file=p.join(target,'M_'..name..'.uasset');if p.exists(file)then before[file]={sha256=h.sha256(file),mtime=lfs.attributes(file,'modification')}end end
  local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{project,'-run=OmMapBiomeMaterials','-result='..p.join(out,'native-result.json'),
   '-unattended','-nop4','-NullRHI','-nosound','-NoLiveCoding','-abslog='..p.join(out,'engine.log')},{cwd=b.root,check=false})
  p.write(p.join(out,'process.log'),(result.stdout or'')..(result.stderr or''),false,true);report.exit_code=result.exit_code
  for _,row in ipairs(original)do assert(h.sha256(row.path)==row.sha256 and lfs.attributes(row.path,'modification')==row.mtime,'original surface package changed')end
  assert(result.exit_code==0,'native biome material producer failed; inspect engine/process logs')
  local native=j.read(p.join(out,'native-result.json'));assert(native.success and native.schema=='omoba.map-biome-materials/1'and #native.materials==3)
  report.created=native.created;report.saved=native.saved;report.verified=native.verified;report.materials={}
  local portable,world={},{}
  for _,name in ipairs(ids)do
   local file=p.join(target,'M_'..name..'.uasset');local sha=h.sha256(file);local mtime=lfs.attributes(file,'modification')
   if before[file]then assert(sha==before[file].sha256 and mtime==before[file].mtime,'existing biome package changed')end
   report.materials[#report.materials+1]={id=name,path=file,sha256=sha,mtime=mtime}
   portable[#portable+1]={id=name,package_file=content..'/M_'..name..'.uasset',sha256=sha}
  end
  for _,row in ipairs(original)do world[#world+1]={package_file='omfue/Content/Omoba/WorldSurfacesV1/'..row.name,sha256=row.sha256}end
  if not p.exists(p.join(target,'provenance.json'))then j.write(p.join(target,'provenance.json'),{schema='omoba.map-biome-material-provenance/1',
   source='Original palette and shader math; unchanged repository grass/stone textures. No downloaded or Riot assets.',
   producer=producer,producer_sha256=h.sha256(p.join(b.root,producer)),materials=portable,world_textures=world,
   source_record='docs/plans/2026-10-12-world-surface-presentation.md',gameplay='Visual only; no collision, navigation, lane or fog changes.'},false)end
  local verified=current(b.root);verified.schema='omoba.map-biome-verification/1';verified.evidence='omfue/Saved/Automation/MapBiomeMaterials/'..id
  p.mkdir_p(p.parent(p.join(b.root,receipt)));j.write(p.join(b.root,receipt),verified,true)
  M.require_ready(b.root);report.success=true
 end,debug.traceback)
 if not ok then report.error=tostring(err)end
 j.write(p.join(out,'summary.json'),report,false)
 assert(ok and report.success,'biome material workflow failed; evidence: '..out..' / '..tostring(err))
 print(('Map biome materials PASS: %d created / %d saved / %d verified; evidence: %s'):format(report.created,report.saved,report.verified,out))
 return report
end
if ...=='ue_map_biome_materials'then return M end
M.main(arg)
