local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local j=b.lib('json');local m=require('character_retarget')
local total=0
local function fixture()
 local plan={source_registry_sha256=string.rep('a',64),source_id='animation_library',reference_clip='calibration',targets={'male_base'}}
 local skeleton='/Game/Source/male.Skeleton';local mesh='/Game/Source/male.Mesh'
 local registry={assets={{id='animation_library',objects={{class='AnimSequence',object_path='/Game/Source/Idle.Idle'},
  {class='AnimSequence',object_path='/Game/Source/Walk.Walk'}}},{id='male_base',objects={{class='SkeletalMesh',object_path=mesh,bones=65},
  {class='Skeleton',object_path=skeleton}}}}}
 local row={id='male_base',signature=plan.source_registry_sha256..'|animation_library|male_base|calibration|auto_ik_calibrated_centimeters_v4',action='imported',reference_clip='calibration',source_root_correction='finite calibrated transform',source_facing_correction_deg=0,
  target_mesh=mesh,target_skeleton=skeleton,chains={},objects={},finite_transforms=130,moving_clips=1}
 for i=1,8 do row.chains[i]={source='chain'..i,target='chain'..i}end
 local function object(name,class)local o={object_path=m.destination..'/male_base/'..name..'.'..name,class=class};row.objects[#row.objects+1]=o;return o end
 object('source_rig','IKRigDefinition');object('target_rig','IKRigDefinition');object('retargeter','IKRetargeter')
 for _,name in ipairs({'Idle','Walk'})do local o=object('male_base_'..name,'AnimSequence');o.seconds=1;o.keys=2;o.tracks=65;o.finite_transforms=65;o.skeleton=skeleton;o.has_motion=name=='Walk';o.upright_permille=960;o.pose_error_deg=12 end
 return plan,registry,{schema='omoba.character-retarget-result.v1',success=true,error='',render_verified=false,imported=1,verified=0,saved=5,targets={row}}
end
local function test(name,change,valid)
 local plan,registry,result=fixture();change(plan,registry,result)
 local ok,packages=pcall(m.verify_result,plan,registry,result);assert(ok==valid,name..': '..tostring(packages));total=total+1
end
test('complete actual source identity',function()end,true)
test('facing-corrected target signs its quarter turn',function(p,r,v)v.targets[1].source_facing_correction_deg=180
 v.targets[1].signature=p.source_registry_sha256..'|animation_library|male_base|calibration|facing:180|auto_ik_calibrated_centimeters_v4'end,true)
test('no-save repeat',function(p,r,v)v.imported=0;v.verified=1;v.saved=0;v.targets[1].action='verified'end,true)
local cases={
 schema=function(v)v.schema='old'end, failure=function(v)v.success=false end,error=function(v)v.error='failure'end,
 false_render_claim=function(v)v.render_verified=true end, missing_target=function(v)v.targets={}end,
 unknown_target=function(v)v.targets[1].id='foreign'end, stale_signature=function(v)v.targets[1].signature='old'end,
 wrong_reference_clip=function(v)v.targets[1].reference_clip='unknown'end,
 missing_root_correction=function(v)v.targets[1].source_root_correction=nil end,
 nonfinite_root_correction=function(v)v.targets[1].source_root_correction='NaN'end,
 wrong_mesh=function(v)v.targets[1].target_mesh='wrong'end,wrong_skeleton=function(v)v.targets[1].target_skeleton='wrong'end,
 unmapped_chain=function(v)v.targets[1].chains[1].source=''end,duplicate_chain=function(v)v.targets[1].chains[2]=v.targets[1].chains[1]end,
 missing_chains=function(v)v.targets[1].chains={}end, foreign_package=function(v)v.targets[1].objects[1].object_path='/Game/Other/source_rig.source_rig'end,
 duplicate_package=function(v)v.targets[1].objects[2]=v.targets[1].objects[1]end,unsupported_class=function(v)v.targets[1].objects[1].class='Texture2D'end,
 missing_clip=function(v)table.remove(v.targets[1].objects)end,wrong_clip=function(v)v.targets[1].objects[5].object_path=m.destination..'/male_base/male_base_Unknown.male_base_Unknown'end,
 animation_skeleton=function(v)v.targets[1].objects[4].skeleton='wrong'end,zero_duration=function(v)v.targets[1].objects[4].seconds=0 end,
 infinite_duration=function(v)v.targets[1].objects[4].seconds=math.huge end,zero_keys=function(v)v.targets[1].objects[4].keys=0 end,
 fractional_keys=function(v)v.targets[1].objects[4].keys=1.5 end,excessive_keys=function(v)v.targets[1].objects[4].keys=20001 end,
 unknown_track=function(v)v.targets[1].objects[4].tracks=66 end,zero_transforms=function(v)v.targets[1].objects[4].finite_transforms=0 end,
 wrong_motion_type=function(v)v.targets[1].objects[4].has_motion='yes'end,wrong_transform_sum=function(v)v.targets[1].finite_transforms=129 end,
 wrong_motion_sum=function(v)v.targets[1].moving_clips=2 end,frozen_library=function(v)v.targets[1].objects[5].has_motion=false;v.targets[1].moving_clips=0 end,
 wrong_saved_count=function(v)v.saved=4 end,wrong_imported_count=function(v)v.imported=0;v.verified=1 end,
 fractional_counter=function(v)v.saved=0.5 end,invalid_action=function(v)v.targets[1].action='skipped'end,
 verified_write=function(v)v.targets[1].action='verified';v.imported=0;v.verified=1 end,
 duplicate_target=function(v)v.targets[2]=v.targets[1]end,
 missing_upright=function(v)v.targets[1].objects[4].upright_permille=nil end,
 lying_library=function(v)v.targets[1].objects[4].upright_permille=180;v.targets[1].objects[5].upright_permille=200 end,
 missing_pose_error=function(v)v.targets[1].objects[4].pose_error_deg=nil end,
 folded_limbs=function(v)v.targets[1].objects[5].pose_error_deg=172 end,
 fractional_facing=function(v)v.targets[1].source_facing_correction_deg=45 end,
 unsigned_facing=function(v)v.targets[1].source_facing_correction_deg=180 end,
 }
for name,change in pairs(cases)do test(name,function(p,r,v)change(v)end,false)end
-- A generated target set writes under its own root and pins the target rig's own source hash.
local function tripo(p,r,v)
 p.target_set='tripo_v1';p.target_registry_sha256=string.rep('b',64);r.assets[2].source_sha256=string.rep('c',64)
 local row=v.targets[1];row.signature=p.source_registry_sha256..'|animation_library|male_base|calibration|tripo_v1:'..string.rep('c',64)..'|auto_ik_calibrated_centimeters_v4'
 for _,o in ipairs(row.objects)do o.object_path=o.object_path:gsub('^'..m.destination,m.target_sets.tripo_v1.destination)end
end
test('tripo target set',tripo,true)
test('tripo stale target source',function(p,r,v)tripo(p,r,v);r.assets[2].source_sha256=string.rep('d',64)end,false)
test('tripo output under cc0 root',function(p,r,v)tripo(p,r,v);p.target_set=nil end,false)
test('unknown target set',function(p,r,v)tripo(p,r,v);p.target_set='other'end,false)
print('PASS '..total..' character retarget contract cases; no render acceptance claimed')
