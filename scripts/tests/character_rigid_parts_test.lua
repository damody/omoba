local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local m=require('character_rigid_parts');local total=0
local function fixture()
 local plan={assets={{id='male_head',source_id='male_base',root_bone='Head',pivot_bone='Head',min_weight=0.5}}}
 return plan,{schema='omoba.rigid-character-part-result.v1',success=true,error='',render_verified=false,imported=1,verified=0,saved=1,assets={{
  id='male_head',object_path=m.destination..'/male_head.male_head',source_mesh='/Game/Omoba/Characters/QuaterniusV2/male_base/male_base.male_base',
  root_bone='Head',pivot_bone='Head',min_weight=0.5,rigid_extraction_verified=true,vertices=500,triangles=600,
  coordinate_policy='unreal_centimeters_normalized_pivot_v1',requires_reference_bone_scale_normalization=true,
  reference_pivot='0,0,170|0,0,0|1,1,1',bounds_cm='Min=-10,-10,-10 Max=10,10,10',material_slots={'Eyes','Hair','Skin'},action='imported'}}}
end
local function test(name,change,valid)local p,v=fixture();change(v);local ok,value=pcall(m.verify_result,p,v);assert(ok==valid,name..': '..tostring(value));total=total+1 end
test('bounded source geometry/pivot',function()end,true)
test('repeat no-save',function(v)v.assets[1].action='verified';v.imported=0;v.verified=1;v.saved=0 end,true)
local cases={schema=function(v)v.schema='old'end,failed=function(v)v.success=false end,render_claim=function(v)v.render_verified=true end,
 missing_part=function(v)v.assets={}end,wrong_id=function(v)v.assets[1].id='foreign'end,wrong_package=function(v)v.assets[1].object_path='/Game/Other/Head.Head'end,
 wrong_source=function(v)v.assets[1].source_mesh='wrong'end,wrong_root=function(v)v.assets[1].root_bone='hips'end,wrong_pivot=function(v)v.assets[1].pivot_bone='hips'end,
 wrong_threshold=function(v)v.assets[1].min_weight=0.1 end,unverified_geometry=function(v)v.assets[1].rigid_extraction_verified=false end,
 empty_vertices=function(v)v.assets[1].vertices=0 end,fractional_triangles=function(v)v.assets[1].triangles=100.5 end,
 wrong_units=function(v)v.assets[1].coordinate_policy='bone_units'end,missing_scale_normalization=function(v)v.assets[1].requires_reference_bone_scale_normalization=false end,
 geometry_budget=function(v)v.assets[1].vertices=200001 end,nonfinite_pivot=function(v)v.assets[1].reference_pivot='nan'end,
 nonfinite_bounds=function(v)v.assets[1].bounds_cm='inf'end,missing_materials=function(v)v.assets[1].material_slots={}end,
 duplicate_materials=function(v)v.assets[1].material_slots={'Eyes','Eyes'}end,bad_material_name=function(v)v.assets[1].material_slots={'bad name'}end,
 skipped_part=function(v)v.assets[1].action='skipped'end,wrong_save_count=function(v)v.saved=0 end,wrong_imported_count=function(v)v.imported=2 end,
 }
for name,change in pairs(cases)do test(name,change,false)end
print('PASS '..total..' rigid character part contract cases; deformation/render remains independent')
