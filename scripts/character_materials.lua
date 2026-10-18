local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local M={schema='omoba.character-material-plan.v1',destination='/Game/Omoba/Characters/MaterialsV1'}
local root_relative='omfue/SourceArt/Characters/MaterialsV1'
function M.validate(root)
 local manifest_file=p.join(root,root_relative,'manifest.json');local raw=assert(p.read(manifest_file,true));assert(#raw<=256*1024)
 local manifest=j.decode(raw);assert(manifest.schema=='omoba.character-material-source.v1'and manifest.render_verified==false)
 assert(type(manifest.licenses)=='table'and #manifest.licenses==2 and type(manifest.textures)=='table'and #manifest.textures>=1 and #manifest.textures<=64)
 local licenses={};for _,row in ipairs(manifest.licenses)do
  assert((row.id=='base'or row.id=='outfit')and not licenses[row.id]and row.author=='Quaternius'and row.license=='CC0-1.0')
  assert(row.path==root_relative..'/licenses/'..row.id..'.txt'and row.source_page:match('^https://quaternius%.itch%.io/[a-z-]+$'))
  local file=p.join(root,row.path);local text=assert(p.read(file,true));assert(#text<=65536 and text:find('CC0',1,true)and h.sha256(file)==row.sha256)
  licenses[row.id]=true
 end
 local textures,by_id={},{}
 for _,row in ipairs(manifest.textures)do
  assert(type(row.id)=='string'and #row.id<=96 and row.id:match('^T_[A-Za-z0-9_]+$')and not by_id[row.id]and licenses[row.license_id])
  assert(row.source==root_relative..'/'..row.id..'.png'and type(row.sha256)=='string'and #row.sha256==64 and row.sha256:match('^[a-f0-9]+$'))
  local file=p.join(root,row.source);assert(p.attributes(file).size<=32*1024*1024);local bytes=assert(p.read(file,true))
  assert(bytes:sub(1,8)=='\137PNG\r\n\26\n'and bytes:sub(13,16)=='IHDR');local width,height=string.unpack('>I4I4',bytes,17)
  assert(width==row.width and height==row.height and width>=1 and height>=1 and width<=8192 and height<=8192 and h.sha256(file)==row.sha256)
  local normal=row.id:match('_Normal$')~=nil;local mask=row.id:match('_ORM$')~=nil or row.id:match('_Roughness$')~=nil
  assert(row.normal==normal and row.packed_orm==(row.id:match('_ORM$')~=nil))
  local value={id=row.id,source=p.absolute(file,root),sha256=row.sha256,width=width,height=height,kind=normal and'normal'or(mask and'mask'or'color'),flip_green=false}
  textures[#textures+1]=value;by_id[row.id]=value
 end
 local definitions={
  {'eyes','T_Eye_Brown','T_Eye_Normal'}, {'hair_1','T_Hair_1_BaseColor','T_Hair_1_Normal'}, {'hair_2','T_Hair_2_BaseColor','T_Hair_2_Normal'},
  {'superhero_male','T_Superhero_Male_Dark','T_Superhero_Male_Normal','T_Superhero_Male_Roughness'},
  {'superhero_female','T_Superhero_Female_Dark_BaseColor','T_Superhero_Female_Normal','T_Superhero_Female_Roughness'},
  {'regular_male','T_Regular_Male_Dark_BaseColor','T_Regular_Male_Normal','T_Regular_Male_Roughness'},
  {'regular_female','T_Regular_Female_Dark_BaseColor','T_Regular_Female_Normal','T_Regular_Female_Roughness'},
  {'ranger','T_Ranger_BaseColor','T_Ranger_Normal','T_Ranger_ORM'}, {'peasant','T_Peasant_BaseColor','T_Peasant_Normal','T_Peasant_ORM'},
 }
 local materials={};for _,row in ipairs(definitions)do
  assert(by_id[row[2]].kind=='color'and by_id[row[3]].kind=='normal');if row[4]then assert(by_id[row[4]].kind=='mask')end
  materials[#materials+1]={id=row[1],base_color=row[2],normal=row[3],mask=row[4]or'',packed_orm=row[4]and row[4]:match('_ORM$')~=nil or false}
 end
 return {schema=M.schema,manifest_sha256=h.sha256(manifest_file),textures=textures,materials=materials}
end
function M.verify_result(plan,result)
 assert(result.schema=='omoba.character-material-result.v1'and result.success==true and result.error==''and result.render_verified==false)
 assert(type(result.assets)=='table'and #result.assets==#plan.textures+#plan.materials)
 local expected={};for _,row in ipairs(plan.textures)do expected[row.id]={class='Texture2D',value=row}end
 for _,row in ipairs(plan.materials)do expected['M_'..row.id]={class='Material',value=row}end
 local packages,seen,imported,verified,saved={},{},0,0,0
 for _,row in ipairs(result.assets)do
  local value=assert(expected[row.id]);expected[row.id]=nil;assert(row.class==value.class)
  local directory=value.class=='Texture2D'and'textures'or'materials';local package=M.destination..'/'..directory..'/'..row.id
  assert(row.object_path==package..'.'..row.id and not seen[package]);seen[package]=true;packages[#packages+1]=package
  if row.action=='imported'then imported=imported+1;saved=saved+1 else assert(row.action=='verified');verified=verified+1 end
  if value.class=='Texture2D'then local texture=value.value
   assert(row.width==texture.width and row.height==texture.height and row.kind==texture.kind and row.srgb==(texture.kind=='color')
    and row.flip_green==texture.flip_green and row.max_texture_size==2048 and row.streaming==true and row.mipmaps==true)
  else assert(row.graph_verified==true and row.skeletal_usage==true and row.shader_errors==0)end
 end
 assert(not next(expected)and result.imported==imported and result.verified==verified and result.saved==saved)
 return packages
end
return M
