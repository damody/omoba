-- Native IK retarget contracts. Matching bone names never imply matching rest poses.
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local source=require('character_source_import')
local M={schema='omoba.character-retarget-plan.v1',destination='/Game/Omoba/Characters/RetargetV4'}
local function integer(n,a,z)return type(n)=='number'and n==math.floor(n)and n>=a and n<=z end
local function id(s)return type(s)=='string'and #s<=64 and s:match('^[a-z][a-z0-9_]*$')end
function M.validate(root,targets)
 local source_plan=source.validate(root);local file=source.registry(root)
 local raw=assert(p.read(file,true));assert(#raw<=2*1024*1024,'bounded source registry required')
 local registry=j.decode(raw)
 assert(registry.schema=='omoba.character-source-registry.v1'and registry.coordinate_policy==source.coordinate_policy)
 assert(registry.source_manifest_sha256==source_plan.manifest_sha256,'registry source manifest differs')
 local packages=source.verify_result(source_plan,{schema='omoba.character-source-result.v1',success=true,error='',
  coordinate_policy=registry.coordinate_policy,retarget_verified=false,render_verified=false,imported=0,verified=#source_plan.assets,
  saved=0,assets=registry.assets})
 local count=0;for package,sha in pairs(registry.package_sha256)do
  count=count+1;assert(type(sha)=='string'and #sha==64 and sha:match('^[a-f0-9]+$'))
  assert(type(package)=='string'and package:sub(1,#source.destination+1)==source.destination..'/'and not package:find('..',1,true))
  local asset=p.join(root,'omfue/Content',package:sub(7)..'.uasset');assert(h.sha256(asset)==sha,'source package SHA256 differs')
 end
 assert(count==#packages,'source package count differs');for _,package in ipairs(packages)do assert(registry.package_sha256[package])end
 local by_id={};for _,row in ipairs(registry.assets)do by_id[row.id]=row end
 local library=assert(by_id.animation_library);local clips=0;for _,obj in ipairs(library.objects)do if obj.class=='AnimSequence'then clips=clips+1 end end
 assert(clips>0 and clips<=128,'bounded source animation library required')
 targets=targets or {'male_base','female_base','male_ranger','female_ranger','male_peasant','female_peasant'}
 assert(type(targets)=='table'and #targets>=1 and #targets<=16);local seen={}
 for _,target in ipairs(targets)do assert(id(target)and target~='animation_library'and by_id[target]and not seen[target],'invalid target id');seen[target]=true end
 return {schema=M.schema,source_registry_sha256=h.sha256(file),source_id='animation_library',reference_clip='animation_library_Anim_Armature_A_TPose',targets=targets},registry,clips
end
function M.verify_result(plan,registry,result)
 assert(result.schema=='omoba.character-retarget-result.v1'and result.success==true and result.error=='','native retarget failed')
 assert(result.render_verified==false,'retarget is not render acceptance')
 assert(integer(result.imported,0,#plan.targets)and integer(result.verified,0,#plan.targets)and result.imported+result.verified==#plan.targets)
 assert(integer(result.saved,0,4096)and type(result.targets)=='table'and #result.targets==#plan.targets)
 local rigs={};for _,row in ipairs(registry.assets)do rigs[row.id]=row end
 local source_clips={};for _,obj in ipairs(assert(rigs[plan.source_id]).objects)do if obj.class=='AnimSequence'then
  local name=assert(obj.object_path:match('%.([A-Za-z0-9_]+)$'));source_clips[name]=obj
 end end
 local expected={};for _,target in ipairs(plan.targets)do expected[target]=true end
 local packages,seen,imported,verified,saved={},{},0,0,0
 for _,row in ipairs(result.targets)do
  assert(expected[row.id],'unknown or repeated retarget target');expected[row.id]=nil
  assert(row.signature==plan.source_registry_sha256..'|'..plan.source_id..'|'..row.id..'|'..plan.reference_clip..'|auto_ik_calibrated_centimeters_v4')
  assert(row.reference_clip==plan.reference_clip and type(row.source_root_correction)=='string'and #row.source_root_correction>0 and #row.source_root_correction<=2048)
  assert(not row.source_root_correction:lower():find('nan',1,true)and not row.source_root_correction:lower():find('inf',1,true),'nonfinite root correction')
  if row.action=='imported'then imported=imported+1;saved=saved+#row.objects else assert(row.action=='verified');verified=verified+1 end
  local mesh,skeleton,bones;for _,obj in ipairs(rigs[row.id].objects)do
   if obj.class=='SkeletalMesh'then mesh=obj.object_path;bones=obj.bones elseif obj.class=='Skeleton'then skeleton=obj.object_path end
  end
  assert(row.target_mesh==mesh and row.target_skeleton==skeleton,'wrong target rig')
  assert(type(row.chains)=='table'and #row.chains>=8 and #row.chains<=256);local chains={}
  for _,chain in ipairs(row.chains)do assert(type(chain.target)=='string'and chain.target~=''and not chains[chain.target]
   and type(chain.source)=='string'and chain.source~='');chains[chain.target]=true end
  local clips,transforms,moving={},0,0;local counts={IKRigDefinition=0,IKRetargeter=0,AnimSequence=0}
  for _,obj in ipairs(row.objects)do
   local package,name=obj.object_path:match('^(.-)%.([A-Za-z0-9_]+)$')
   assert(package and package==M.destination..'/'..row.id..'/'..name and not seen[package],'output escaped target ownership')
   seen[package]=true;packages[#packages+1]=package;assert(counts[obj.class]~=nil,'unexpected output class');counts[obj.class]=counts[obj.class]+1
   if obj.class=='AnimSequence'then
    local source_name=assert(name:match('^'..row.id..'_(.+)$'));assert(source_clips[source_name]and not clips[source_name],'wrong or duplicated source clip');clips[source_name]=true
    assert(obj.skeleton==skeleton and integer(obj.keys,1,20000)and integer(obj.tracks,1,bones)
     and type(obj.seconds)=='number'and obj.seconds>0 and obj.seconds<=600 and integer(obj.finite_transforms,1,10000000))
    assert(type(obj.has_motion)=='boolean');transforms=transforms+obj.finite_transforms;moving=moving+(obj.has_motion and 1 or 0)
   elseif obj.class=='IKRigDefinition'then assert(name=='source_rig'or name=='target_rig') else assert(name=='retargeter')end
  end
  local clip_count=0;for name in pairs(source_clips)do clip_count=clip_count+1;assert(clips[name],'missing retargeted source clip')end
  assert(counts.IKRigDefinition==2 and counts.IKRetargeter==1 and counts.AnimSequence==clip_count)
  assert(transforms==row.finite_transforms and transforms<=10000000 and moving>0 and moving==row.moving_clips,'retarget motion accounting differs')
 end
 assert(not next(expected)and imported==result.imported and verified==result.verified and saved==result.saved,'retarget action counters differ')
 return packages
end
return M
