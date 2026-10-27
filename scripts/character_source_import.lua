-- Independently imported source rigs; names alone never establish retarget compatibility.
local b=require('_bootstrap')
local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local M={schema='omoba.character-source-plan.v1',destination='/Game/Omoba/Characters/QuaterniusV2',coordinate_policy='fbx_to_unreal_centimeters_v2'}
-- Owned source sets (mirrors FSourceSet in OmCharacterSourceImportCommandlet.cpp). quaternius_v1 is the
-- original CC0 rig set and its plan carries no source_set field; tripo_v1 holds one generated hero rig per
-- directory, <dir>/<id>/<id>.fbx plus that directory's own manifest.json, imported with its embedded texture.
M.sets={
 quaternius_v1={source_dir='omfue/SourceArt/Characters/QuaterniusV1',destination=M.destination,
  registry='omfue/Content/Omoba/Characters/QuaterniusV2/source-registry.json',materials=false},
 tripo_v1={source_dir='omfue/SourceArt/Characters/TripoV1',destination='/Game/Omoba/Characters/TripoV1',
  registry='omfue/Content/Omoba/Characters/TripoV1/source-registry.json',materials=true,tripo_schema='omoba.tripo-character-source.v1'},
 -- Generated unrigged world models (towers, keeps): one StaticMesh per directory, no skeleton.
 tripo_static_v1={source_dir='omfue/SourceArt/Structures/TripoStaticV1',destination='/Game/Omoba/Structures/TripoStaticV1',
  registry='omfue/Content/Omoba/Structures/TripoStaticV1/source-registry.json',materials=true,tripo_schema='omoba.tripo-static-source.v1',static=true},
}
-- Chains UE auto-characterizes on the CC0 animation library's source rig.
M.retarget_chain_names={}
for _,side in ipairs({'Left','Right'})do for _,part in ipairs({'Leg','Foot','Clavicle','Arm','Thumb','Index','Middle','Ring','Pinky'})do M.retarget_chain_names[side..part]=true end end
for _,name in ipairs({'Spine','Neck','Head'})do M.retarget_chain_names[name]=true end
local function set_of(id)return assert(M.sets[id or'quaternius_v1'],'unknown character source set: '..tostring(id))end
function M.manifest(root)return p.join(root,'omfue/SourceArt/Characters/QuaterniusV1/manifest.json')end
function M.registry(root,set)return p.join(root,set_of(set).registry)end
local function canonical(file,root)return p.absolute(file,root):gsub('\\','/'):lower()end
local function integer(n,low,high)return type(n)=='number'and n==math.floor(n)and n>=low and n<=high end
local function sha(s)return type(s)=='string'and #s==64 and s:match('^[a-f0-9]+$')~=nil end
local function fbx_bytes(file)
 local size=p.attributes(file).size;assert(size>=27 and size<=64*1024*1024,'bounded FBX required')
 local bytes=assert(p.read(file,true));assert(#bytes==size and bytes:sub(1,23)=='Kaydara FBX Binary  \0\26\0','binary FBX header required')
 local version=string.unpack('<I4',bytes,24);assert(version>=7100 and version<=7700,'unsupported FBX version')
end
-- One generated hero rig: identity, bounded skeleton facts, source hash and the Tripo/Grok provenance
-- the asset ledger records. Downloaded bytes are untrusted: only the hash-pinned FBX is ever imported.
local function validate_tripo(root,set,project_dir,set_id)
 local base=p.join(root,set.source_dir);local assets={};local ids={}
 if p.is_directory(base)then for name in require('lfs').dir(base)do
  if name~='.'and name~='..'and p.is_file(p.join(base,name,'manifest.json'))then ids[#ids+1]=name end
 end end
 table.sort(ids);assert(#ids>=1 and #ids<=32,'bounded tripo hero source set required')
 local manifests={}
 for _,id in ipairs(ids)do
  assert(id:match('^[a-z][a-z0-9_]*$')and #id<=64,'invalid tripo hero directory')
  local file=p.join(base,id,'manifest.json');local raw=assert(p.read(file,true));assert(#raw<=128*1024,'bounded tripo manifest required')
  local m=j.decode(raw);assert(m.schema==set.tripo_schema and m.id==id,'tripo manifest identity differs')
  assert(m.source==set.source_dir..'/'..id..'/'..id..'.fbx','tripo FBX escaped its hero directory')
  assert(sha(m.sha256)and(set.static and m.bones==0 or not set.static and integer(m.bones,1,256))and m.animations==0,'tripo rig facts required')
  assert(m.import_scale==nil or(type(m.import_scale)=='number'and m.import_scale>=0.001 and m.import_scale<=1000),'invalid import_scale')
  local v=m.provenance;assert(type(v)=='table'and v.service=='tripo3d.ai'and type(v.task_id)=='string'and v.task_id~=''
   and type(v.model_version)=='string'and type(v.options)=='table'and integer(v.credits_spent,0,100000),'tripo provenance required')
  assert(type(v.reference_images)=='table'and #v.reference_images>=1,'reference image provenance required')
  for _,ref in ipairs(v.reference_images)do assert(type(ref.file)=='string'and sha(ref.sha256)and type(ref.generator)=='string')end
  -- Optional explicit characterization for rigs UE's auto characterization does not recognize;
  -- chain names must be the CC0 source library's auto chain names so exact mapping applies.
  if m.retarget~=nil then
   local r=m.retarget;assert(type(r)=='table'and type(r.root)=='string'and r.root~=''and type(r.chains)=='table'and #r.chains>=8 and #r.chains<=32,'invalid retarget characterization')
   local names={};for _,c in ipairs(r.chains)do
    assert(M.retarget_chain_names[c.name]and not names[c.name]and type(c.start)=='string'and c.start~=''and type(c['end'])=='string'and c['end']~='','invalid retarget chain')
    names[c.name]=true
   end
  end
  local expected=p.join(root,m.source);fbx_bytes(expected);assert(h.sha256(expected)==m.sha256,'FBX SHA256 differs')
  local source=project_dir and p.join(project_dir,set.source_dir:sub(#'omfue/'+1),id,id..'.fbx')or p.absolute(expected,root)
  assets[#assets+1]={id=id,source=source,sha256=m.sha256,bones=m.bones,animations=0,import_scale=m.import_scale,provenance=v,retarget=m.retarget}
  manifests[id]=h.sha256(file)
 end
 return {schema=M.schema,coordinate_policy=M.coordinate_policy,source_set=set_id,assets=assets,manifests=manifests}
end
-- project_dir (optional) is the Unreal project directory the commandlet runs from (a run snapshot,
-- E116); source paths in the plan must be under that directory's own SourceArt.
function M.validate(root,set_id,project_dir)
 if set_id and set_id~='quaternius_v1'then return validate_tripo(root,set_of(set_id),project_dir,set_id)end
 local file=M.manifest(root);local raw=assert(p.read(file,true));assert(#raw<=512*1024,'bounded character manifest required')
 local m=j.decode(raw);assert(m.schema==M.schema and m.coordinate_policy==M.coordinate_policy and type(m.assets)=='table'and #m.assets>=1 and #m.assets<=32)
 local source_root=p.join(root,'omfue/SourceArt/Characters/QuaterniusV1')
 assert(type(m.licenses)=='table'and #m.licenses>=1 and #m.licenses<=16,'included licenses required')
 local licenses={};local license_ids={}
 for _,row in ipairs(m.licenses)do
  assert(type(row.id)=='string'and row.id:match('^[a-z][a-z0-9_]*$')and #row.id<=64 and not license_ids[row.id]);license_ids[row.id]=true
  local expected=p.join(source_root,'licenses',row.id..'.txt')
  assert(canonical(row.path,root)==canonical(expected,root),'license path escaped owned source directory')
  local text=assert(p.read(expected,true));assert(#text>0 and #text<=65536 and text:find('CC0',1,true),'included CC0 license required')
  assert(h.sha256(expected)==row.sha256,'license SHA256 differs');licenses[#licenses+1]=row
 end
 local assets,seen={},{}
 for _,a in ipairs(m.assets)do
  assert(type(a.id)=='string'and a.id:match('^[a-z][a-z0-9_]*$')and #a.id<=64 and not seen[a.id],'invalid or duplicate character id');seen[a.id]=true
  assert(integer(a.bones,1,256)and integer(a.animations,0,128),'bounded expected skeleton/clip counts required')
  assert(type(a.sha256)=='string'and #a.sha256==64 and a.sha256:match('^[a-f0-9]+$'),'SHA256 required')
  assert(type(a.provenance)=='table'and a.provenance.author=='Quaternius'and a.provenance.license=='CC0-1.0'
   and license_ids[a.provenance.license_id]and type(a.provenance.source_page)=='string'
   and a.provenance.source_page:match('^https://quaternius%.itch%.io/[a-z-]+$'),'source provenance required')
  local expected=p.join(source_root,a.id..'.fbx');assert(canonical(a.source,root)==canonical(expected,root),'FBX escaped owned source directory')
  fbx_bytes(expected);assert(h.sha256(expected)==a.sha256,'FBX SHA256 differs')
  local source=project_dir and p.join(project_dir,'SourceArt/Characters/QuaterniusV1',a.id..'.fbx')or p.absolute(expected,root)
  assets[#assets+1]={id=a.id,source=source,sha256=a.sha256,bones=a.bones,animations=a.animations,provenance=a.provenance}
 end
 return {schema=M.schema,coordinate_policy=M.coordinate_policy,assets=assets,licenses=licenses,manifest_sha256=h.sha256(file)}
end
function M.verify_result(plan,result)
 assert(result.schema=='omoba.character-source-result.v1'and result.success==true and result.error=='','commandlet failed')
 assert(result.coordinate_policy==M.coordinate_policy,'source coordinate policy differs')
 assert(result.retarget_verified==false and result.render_verified==false,'source import cannot claim retarget or visual acceptance')
 assert(integer(result.imported,0,#plan.assets)and integer(result.verified,0,#plan.assets)
  and result.imported+result.verified==#plan.assets and integer(result.saved,0,8192),'invalid import counters')
 assert(type(result.assets)=='table'and #result.assets==#plan.assets,'wrong result asset count')
 local by_id={};for _,a in ipairs(plan.assets)do by_id[a.id]=a end
 local set=set_of(plan.source_set);local destination=set.destination
 local objects,packages,package_seen={},{},{};local imported,verified,saved=0,0,0
 for _,row in ipairs(result.assets)do
  local a=assert(by_id[row.id],'unknown or repeated result id');by_id[row.id]=nil
  assert(row.source_sha256==a.sha256 and(row.action=='imported'or row.action=='verified'))
  if row.action=='imported'then imported=imported+1;saved=saved+#row.objects else verified=verified+1 end
  -- A set with embedded materials adds its own bounded Material/Texture2D exports.
  local base=set.static and 1 or 2+a.animations -- a static model has no Skeleton or clips
  assert(type(row.objects)=='table'and(#row.objects==base or(set.materials and #row.objects>base and #row.objects<=32+base)))
  assert(type(row.reference_pose)=='table'and #row.reference_pose==a.bones)
  local names={};for index,bone in ipairs(row.reference_pose)do
   assert(type(bone.name)=='string'and bone.name~=''and not names[bone.name]);names[bone.name]=true
   assert(integer(bone.parent,-1,index-2)and(index~=1 or bone.parent==-1),'invalid imported bone hierarchy')
   assert(type(bone.local_transform)=='string'and not bone.local_transform:lower():find('nan',1,true)
    and not bone.local_transform:lower():find('inf',1,true),'invalid reference pose')
  end
  local count={SkeletalMesh=0,Skeleton=0,AnimSequence=0,StaticMesh=0};local skeleton,mesh_skeleton
  if set.materials then count.Material=0;count.MaterialInstanceConstant=0;count.Texture2D=0 end
  for _,obj in ipairs(row.objects)do
   assert(type(obj.object_path)=='string'and obj.object_path:sub(1,#destination+#a.id+2)==destination..'/'..a.id..'/'
    and not obj.object_path:find('..',1,true)and not objects[obj.object_path],'object path escaped job ownership')
   objects[obj.object_path]=true;assert(count[obj.class]~=nil,'unexpected imported class');count[obj.class]=count[obj.class]+1
   local package,name=obj.object_path:match('^(.-)%.([A-Za-z0-9_]+)$');assert(package and package:match('/([A-Za-z0-9_]+)$')==name)
   assert(not package_seen[package],'multiple imported exports share one package');package_seen[package]=true;packages[#packages+1]=package
   if obj.class=='Skeleton'then skeleton=obj.object_path
   elseif obj.class=='SkeletalMesh'then assert(obj.bones==a.bones and type(obj.height_cm)=='number'and obj.height_cm>=50 and obj.height_cm<=500);mesh_skeleton=obj.skeleton
   elseif obj.class=='StaticMesh'then assert(set.static and type(obj.height_cm)=='number'and obj.height_cm>=10 and obj.height_cm<=5000
    and integer(obj.triangles,1,2000000),'invalid imported static model')
   elseif obj.class=='AnimSequence'then assert(type(obj.seconds)=='number'and obj.seconds>0 and obj.seconds<600 and integer(obj.sampled_keys,2,100000))
   elseif obj.class=='Texture2D'then assert(integer(obj.width,1,8192)and integer(obj.height,1,8192),'bounded embedded texture required')end
  end
  if set.static then assert(count.StaticMesh==1 and count.SkeletalMesh==0 and count.Skeleton==0 and count.AnimSequence==0,'static set imports one StaticMesh only')
  else assert(count.SkeletalMesh==1 and count.Skeleton==1 and count.StaticMesh==0 and count.AnimSequence==a.animations and skeleton==mesh_skeleton)end
  assert(not set.materials or(count.Material+count.MaterialInstanceConstant<=16 and count.Texture2D<=16),'bounded embedded materials required')
  for _,obj in ipairs(row.objects)do if obj.class=='AnimSequence'then assert(obj.skeleton==skeleton,'wrong imported animation source skeleton')end end
 end
 assert(not next(by_id)and result.imported==imported and result.verified==verified and result.saved==saved,'result counters differ from actual asset actions');return packages
end
return M
