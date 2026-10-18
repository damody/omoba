local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash');local a=require('character_source_import')
local count=0;local function check(fn)fn();count=count+1 end
local function rejected(fn)check(function()assert(not pcall(fn),'invalid character source accepted')end)end
local root=p.join(b.root,'.ai-collab/character-source-contract-tests',tostring(os.time())..'-'..tostring({}):gsub('[^%w]',''))
local relative='omfue/SourceArt/Characters/QuaterniusV1';local dir=p.join(root,relative);p.mkdir_p(p.join(dir,'licenses'))
local source_file=p.join(dir,'sample.fbx');local bytes='Kaydara FBX Binary  \0\26\0'..string.pack('<I4',7400)
p.write(source_file,bytes,false,true);local license_file=p.join(dir,'licenses/sample.txt');p.write(license_file,'Contract fixture CC0-1.0',false,true)
local manifest={schema=a.schema,coordinate_policy=a.coordinate_policy,licenses={{id='sample',path=relative..'/licenses/sample.txt',sha256=h.sha256(license_file)}},assets={{id='sample',source=relative..'/sample.fbx',sha256=h.sha256(source_file),bones=2,animations=1,
 provenance={author='Quaternius',license='CC0-1.0',license_id='sample',source_page='https://quaternius.itch.io/universal-animation-library'}}}}
local file=p.join(dir,'manifest.json');j.write(file,manifest,false)
local plan=a.validate(root);check(function()assert(#plan.assets==1 and #plan.licenses==1 and plan.assets[1].source==p.absolute(source_file,root))end)
local function bad_manifest(fn)local value=j.decode(j.encode(manifest));fn(value);j.write(file,value,true);rejected(function()a.validate(root)end);j.write(file,manifest,true)end
for _,fn in ipairs({
 function(x)x.schema='other'end,function(x)x.assets={}end,function(x)x.assets[2]=x.assets[1]end,
 function(x)x.assets[1].id='../sample'end,function(x)x.assets[1].source='../sample.fbx'end,
 function(x)x.assets[1].sha256=string.rep('0',64)end,function(x)x.assets[1].sha256='short'end,
 function(x)x.assets[1].bones=0 end,function(x)x.assets[1].bones=257 end,function(x)x.assets[1].bones=2.5 end,
 function(x)x.assets[1].animations=-1 end,function(x)x.assets[1].animations=129 end,function(x)x.assets[1].animations=1.5 end,
 function(x)x.assets[1].provenance=nil end,function(x)x.assets[1].provenance.author='other'end,
 function(x)x.assets[1].provenance.source_page='https://example.com/art'end,function(x)x.assets[1].provenance.license='unknown'end,
 function(x)x.assets[1].provenance.license_id='missing'end,function(x)x.licenses={}end,
 function(x)x.licenses[1].path='../license.txt'end,function(x)x.licenses[1].sha256=string.rep('0',64)end,
 function(x)x.licenses[2]=x.licenses[1]end,
 function(x)x.coordinate_policy=nil end,
})do bad_manifest(fn)end
p.write(source_file,string.rep('\0',#bytes),true,true);rejected(function()a.validate(root)end);p.write(source_file,bytes,true,true)
local prefix=a.destination..'/sample/'
local result={schema='omoba.character-source-result.v1',coordinate_policy=a.coordinate_policy,success=true,error='',imported=1,verified=0,saved=3,retarget_verified=false,render_verified=false,
 assets={{id='sample',source_sha256=plan.assets[1].sha256,action='imported',reference_pose={{name='root',parent=-1,local_transform='Identity'},{name='child',parent=0,local_transform='Identity'}},
 objects={{object_path=prefix..'mesh.mesh',class='SkeletalMesh',bones=2,height_cm=180,skeleton=prefix..'rig.rig'},
 {object_path=prefix..'rig.rig',class='Skeleton'},{object_path=prefix..'idle.idle',class='AnimSequence',seconds=1,sampled_keys=31,skeleton=prefix..'rig.rig'}}}}}
check(function()assert(#a.verify_result(plan,result)==3)end)
local function bad_result(fn)local value=j.decode(j.encode(result));fn(value);rejected(function()a.verify_result(plan,value)end)end
for _,fn in ipairs({
 function(x)x.success=false end,function(x)x.schema='other'end,function(x)x.error='failed'end,
 function(x)x.retarget_verified=true end,function(x)x.render_verified=true end,
 function(x)x.assets={}end,function(x)x.assets[1].id='unknown'end,function(x)x.assets[1].source_sha256='wrong'end,
 function(x)x.assets[1].action='guessed'end,function(x)x.imported=0;x.verified=1 end,function(x)x.saved=2 end,
 function(x)x.assets[1].objects[1].object_path='/Game/Foreign/mesh.mesh'end,
 function(x)x.assets[1].objects[1].object_path=prefix..'../mesh.mesh'end,
 function(x)x.assets[1].objects[2].object_path=x.assets[1].objects[1].object_path end,
 function(x)x.assets[1].objects[1].class='StaticMesh'end,function(x)x.assets[1].objects[1].bones=3 end,
 function(x)x.assets[1].objects[3].skeleton='/Game/Foreign/rig.rig'end,
 function(x)x.assets[1].objects[3].seconds=0 end,function(x)x.assets[1].objects[3].sampled_keys=0 end,
 function(x)x.assets[1].reference_pose[1].parent=0 end,function(x)x.assets[1].reference_pose[2].parent=2 end,
 function(x)x.assets[1].reference_pose[2].name='root'end,function(x)x.assets[1].reference_pose[1].local_transform='nan'end,
 function(x)x.assets[1].reference_pose[1].local_transform='Inf'end,
 function(x)x.coordinate_policy='legacy_unconverted'end,
 function(x)x.assets[1].objects[1].height_cm=1.8 end,
 function(x)x.assets[1].objects[1].height_cm=18000 end,
})do bad_result(fn)end
check(function()local value=j.decode(j.encode(result));value.imported=0;value.verified=1;value.saved=0;value.assets[1].action='verified';assert(#a.verify_result(plan,value)==3)end)
print('PASS '..count..' self-contained character source/import result contract cases; no Unreal import or retarget claim')
