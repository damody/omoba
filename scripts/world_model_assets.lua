local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local m={root='scripts/art/world_models/v1',destination='/Game/Omoba/WorldModelsV1/'}
function m.plan()
 local manifest=j.decode(p.read(p.join(b.root,m.root,'manifest.json'),true))
 assert(manifest.schema=='omoba.world-model-source.v1'and manifest.coordinate_policy=='obj_y_up_meters_to_unreal_cm_v1')
 assert(#manifest.models>0 and #manifest.models<=16)
 for _,source in ipairs(manifest.sources)do
  assert(source.license=='CC0-1.0'and source.license_file==source.id..'-License.txt')
  assert(h.sha256(p.join(b.root,m.root,source.license_file))==source.license_sha256,'original CC0 license changed')
 end
 local plan={schema='omoba.world-model-plan.v1',models={}};local ids={}
 for _,model in ipairs(manifest.models)do
  assert(model.id:match('^[a-z][a-z0-9_]*$')and #model.id<=64 and not ids[model.id]);ids[model.id]=true
  assert(model.source_stem:match('^[%w_-]+$'))
  local row={id=model.id,texture='',texture_sha256='',source_vertices=model.vertices,source_triangles=model.triangles};local seen={}
  for _,file in ipairs(model.files)do
   assert(file.path:sub(1,#model.id+1)==model.id..'/'and not file.path:find('..',1,true)and not file.path:find('\\',1,true)and not seen[file.path]);seen[file.path]=true
   local source=p.join(b.root,m.root,file.path);assert(p.is_file(source)and h.sha256(source)==file.sha256)
   if file.path==model.id..'/'..model.source_stem..'.obj'then row.source=p.absolute(source):gsub('\\','/');row.source_sha256=file.sha256
   elseif file.path==model.id..'/'..model.source_stem..'.mtl'then row.mtl=p.absolute(source):gsub('\\','/');row.mtl_sha256=file.sha256
   elseif file.path==model.id..'/Textures/colormap.png'then row.texture=p.absolute(source):gsub('\\','/');row.texture_sha256=file.sha256
   else error('unexpected world model source file')end
  end
  assert(row.source and row.mtl);plan.models[#plan.models+1]=row
 end
 return plan
end
function m.verify_result(plan,result)
 assert(result.schema=='omoba.world-model-result.v1'and result.success and result.error==''and result.render_verified==false)
 assert(#result.models==#plan.models);local seen={}
 for _,row in ipairs(result.models)do
  assert(not seen[row.id]);seen[row.id]=true
  local expected;for _,model in ipairs(plan.models)do if row.id==model.id then expected=model;break end end
  assert(expected and row.path==m.destination..'SM_'..row.id..'.SM_'..row.id)
  assert(row.vertices==expected.source_vertices and row.source_triangles==expected.source_triangles,'original geometry counts mismatch')
  for _,key in ipairs({'vertices','triangles'})do assert(math.type(row[key])=='integer'and row[key]>=3 and row[key]<=200000)end
  assert(math.type(row.source_triangles)=='integer'and row.source_triangles<=200000)
  assert(math.type(row.degenerate_triangles)=='integer'and row.degenerate_triangles>=0)
  assert(row.triangles+row.degenerate_triangles==row.source_triangles,'source face accounting mismatch')
  assert(type(row.bounds_cm)=='string'and #row.bounds_cm<1024 and not row.bounds_cm:lower():find('nan',1,true))
 end
 for _,key in ipairs({'imported','verified','saved'})do assert(math.type(result[key])=='integer'and result[key]>=0)end
 assert(result.imported+result.verified==#plan.models and result.saved<=#plan.models*18)
 if result.upgraded~=nil then assert(math.type(result.upgraded)=='integer'and result.upgraded>=0 and result.upgraded<=result.imported)end
 return true
end
return m
