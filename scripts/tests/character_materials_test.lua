local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local m=require('character_materials');local total=0
local function fixture(kind)
 kind=kind or'color';local plan={textures={{id='T_Test',width=1024,height=512,kind=kind,flip_green=false}},materials={{id='test'}}}
 local texture={id='T_Test',class='Texture2D',object_path=m.destination..'/textures/T_Test.T_Test',action='imported',width=1024,height=512,
  kind=kind,flip_green=false,srgb=kind=='color',max_texture_size=2048,streaming=true,mipmaps=true}
 local material={id='M_test',class='Material',object_path=m.destination..'/materials/M_test.M_test',action='imported',graph_verified=true,skeletal_usage=true,shader_errors=0}
 return plan,{schema='omoba.character-material-result.v1',success=true,error='',render_verified=false,imported=2,verified=0,saved=2,assets={texture,material}}
end
local function test(name,change,valid,kind)
 local plan,result=fixture(kind);change(plan,result);local ok,value=pcall(m.verify_result,plan,result)
 assert(ok==valid,name..': '..tostring(value));total=total+1
end
for _,kind in ipairs({'color','normal','mask'})do test(kind,function()end,true,kind)end
test('independent no-save verification',function(p,v)v.imported=0;v.verified=2;v.saved=0;for _,row in ipairs(v.assets)do row.action='verified'end end,true)
local cases={schema=function(v)v.schema='old'end,failure=function(v)v.success=false end,error=function(v)v.error='failed'end,
 render_claim=function(v)v.render_verified=true end,missing_asset=function(v)table.remove(v.assets)end,
 duplicate_asset=function(v)v.assets[2]=v.assets[1]end,foreign_asset=function(v)v.assets[1].id='unknown'end,
 wrong_class=function(v)v.assets[1].class='Material'end,foreign_package=function(v)v.assets[1].object_path='/Game/Other/T_Test.T_Test'end,
 texture_size=function(v)v.assets[1].width=512 end,texture_height=function(v)v.assets[1].height=1024 end,
 texture_kind=function(v)v.assets[1].kind='normal'end,wrong_srgb=function(v)v.assets[1].srgb=false end,
 green_convention=function(v)v.assets[1].flip_green=true end,full_resolution=function(v)v.assets[1].max_texture_size=0 end,
 no_streaming=function(v)v.assets[1].streaming=false end,no_mipmaps=function(v)v.assets[1].mipmaps=false end,
 graph_unverified=function(v)v.assets[2].graph_verified=false end,skeletal_usage=function(v)v.assets[2].skeletal_usage=false end,
 shader_error=function(v)v.assets[2].shader_errors=1 end,wrong_action=function(v)v.assets[1].action='skipped'end,
 wrong_import_count=function(v)v.imported=1 end,wrong_verify_count=function(v)v.verified=1 end,
 wrong_save_count=function(v)v.saved=1 end,verified_write=function(v)v.assets[1].action='verified';v.imported=1;v.verified=1 end,
 wrong_object_name=function(v)v.assets[1].object_path=m.destination..'/textures/T_Test.Other'end,
 }
for name,change in pairs(cases)do test(name,function(p,v)change(v)end,false)end
print('PASS '..total..' character material contract cases; native shader/render verification remains independent')
