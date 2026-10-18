local b=require('_bootstrap');local j=b.lib('json');local m={schema='omoba.rigid-character-part-plan.v1',destination='/Game/Omoba/Characters/RigidPartsV1'}
local function integer(n,a,z)return type(n)=='number'and n==math.floor(n)and n>=a and n<=z end
function m.validate(root)
 local retarget=require('character_retarget');local plan=retarget.validate(root)
 return {schema=m.schema,source_registry_sha256=plan.source_registry_sha256,assets={
  {id='male_head',source_id='male_base',root_bone='Head',pivot_bone='Head',min_weight=0.5},
  {id='female_head',source_id='female_base',root_bone='Head',pivot_bone='Head',min_weight=0.5},
  {id='male_head_neck',source_id='male_base',root_bone='neck_01',pivot_bone='Head',min_weight=0.5},
  {id='female_head_neck',source_id='female_base',root_bone='neck_01',pivot_bone='Head',min_weight=0.5}}}
end
function m.verify_result(plan,value)
 assert(value.schema=='omoba.rigid-character-part-result.v1'and value.success==true and value.error==''and value.render_verified==false)
 assert(type(value.assets)=='table'and #value.assets==#plan.assets);local by_id={};for _,row in ipairs(plan.assets)do by_id[row.id]=row end
 local imported,verified,saved=0,0,0;local packages={}
 for _,row in ipairs(value.assets)do
  local job=assert(by_id[row.id]);by_id[row.id]=nil
  assert(row.object_path==m.destination..'/'..job.id..'.'..job.id and row.source_mesh=='/Game/Omoba/Characters/QuaterniusV2/'..job.source_id..'/'..job.source_id..'.'..job.source_id)
  assert(row.root_bone==job.root_bone and row.pivot_bone==job.pivot_bone and row.min_weight==job.min_weight and row.rigid_extraction_verified==true)
  assert(row.coordinate_policy=='unreal_centimeters_normalized_pivot_v1'and row.requires_reference_bone_scale_normalization==true)
  assert(integer(row.vertices,50,200000)and integer(row.triangles,100,200000))
  for _,field in ipairs({'reference_pivot','bounds_cm'})do local text=row[field];assert(type(text)=='string'and #text>0 and #text<=2048 and not text:lower():find('nan',1,true)and not text:lower():find('inf',1,true))end
  assert(type(row.material_slots)=='table'and #row.material_slots>=1 and #row.material_slots<=32);local slots={}
  for _,slot in ipairs(row.material_slots)do assert(type(slot)=='string'and slot:match('^[A-Za-z0-9_]+$')and not slots[slot]);slots[slot]=true end
  if row.action=='imported'then imported=imported+1;saved=saved+1 else assert(row.action=='verified');verified=verified+1 end
  packages[#packages+1]=m.destination..'/'..job.id
 end
 assert(not next(by_id)and value.imported==imported and value.verified==verified and value.saved==saved);return packages
end
return m
