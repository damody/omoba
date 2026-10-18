-- Independently imported source rigs; names alone never establish retarget compatibility.
local b=require('_bootstrap')
local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local M={schema='omoba.character-source-plan.v1',destination='/Game/Omoba/Characters/QuaterniusV2',coordinate_policy='fbx_to_unreal_centimeters_v2'}
function M.manifest(root)return p.join(root,'omfue/SourceArt/Characters/QuaterniusV1/manifest.json')end
function M.registry(root)return p.join(root,'omfue/Content/Omoba/Characters/QuaterniusV2/source-registry.json')end
local function canonical(file,root)return p.absolute(file,root):gsub('\\','/'):lower()end
local function integer(n,low,high)return type(n)=='number'and n==math.floor(n)and n>=low and n<=high end
function M.validate(root)
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
  local size=p.attributes(expected).size;assert(size>=27 and size<=64*1024*1024,'bounded FBX required')
  local bytes=assert(p.read(expected,true));assert(#bytes==size and bytes:sub(1,23)=='Kaydara FBX Binary  \0\26\0','binary FBX header required')
  local version=string.unpack('<I4',bytes,24);assert(version>=7100 and version<=7700,'unsupported FBX version')
  assert(h.sha256(expected)==a.sha256,'FBX SHA256 differs')
  assets[#assets+1]={id=a.id,source=p.absolute(expected,root),sha256=a.sha256,bones=a.bones,animations=a.animations,provenance=a.provenance}
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
 local objects,packages,package_seen={},{},{};local imported,verified,saved=0,0,0
 for _,row in ipairs(result.assets)do
  local a=assert(by_id[row.id],'unknown or repeated result id');by_id[row.id]=nil
  assert(row.source_sha256==a.sha256 and(row.action=='imported'or row.action=='verified'))
  if row.action=='imported'then imported=imported+1;saved=saved+#row.objects else verified=verified+1 end
  assert(type(row.objects)=='table'and #row.objects==2+a.animations)
  assert(type(row.reference_pose)=='table'and #row.reference_pose==a.bones)
  local names={};for index,bone in ipairs(row.reference_pose)do
   assert(type(bone.name)=='string'and bone.name~=''and not names[bone.name]);names[bone.name]=true
   assert(integer(bone.parent,-1,index-2)and(index~=1 or bone.parent==-1),'invalid imported bone hierarchy')
   assert(type(bone.local_transform)=='string'and not bone.local_transform:lower():find('nan',1,true)
    and not bone.local_transform:lower():find('inf',1,true),'invalid reference pose')
  end
  local count={SkeletalMesh=0,Skeleton=0,AnimSequence=0};local skeleton,mesh_skeleton
  for _,obj in ipairs(row.objects)do
   assert(type(obj.object_path)=='string'and obj.object_path:sub(1,#M.destination+#a.id+2)==M.destination..'/'..a.id..'/'
    and not obj.object_path:find('..',1,true)and not objects[obj.object_path],'object path escaped job ownership')
   objects[obj.object_path]=true;assert(count[obj.class]~=nil,'unexpected imported class');count[obj.class]=count[obj.class]+1
   local package,name=obj.object_path:match('^(.-)%.([A-Za-z0-9_]+)$');assert(package and package:match('/([A-Za-z0-9_]+)$')==name)
   assert(not package_seen[package],'multiple imported exports share one package');package_seen[package]=true;packages[#packages+1]=package
   if obj.class=='Skeleton'then skeleton=obj.object_path
   elseif obj.class=='SkeletalMesh'then assert(obj.bones==a.bones and type(obj.height_cm)=='number'and obj.height_cm>=50 and obj.height_cm<=500);mesh_skeleton=obj.skeleton
   else assert(type(obj.seconds)=='number'and obj.seconds>0 and obj.seconds<600 and integer(obj.sampled_keys,2,100000))end
  end
  assert(count.SkeletalMesh==1 and count.Skeleton==1 and count.AnimSequence==a.animations and skeleton==mesh_skeleton)
  for _,obj in ipairs(row.objects)do if obj.class=='AnimSequence'then assert(obj.skeleton==skeleton,'wrong imported animation source skeleton')end end
 end
 assert(not next(by_id)and result.imported==imported and result.verified==verified and result.saved==saved,'result counters differ from actual asset actions');return packages
end
return M
