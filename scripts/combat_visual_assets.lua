-- Original procedural shader inputs. No downloaded or game-owned artwork.
local M={schema='omoba.combat-visual-assets-plan.v1',destination='/Game/Omoba/CombatVfxV1'}
function M.plan()
 return {schema=M.schema,destination=M.destination,graph_version='unlit_soft_shapes_v1',
  provenance='Original Omoba procedural shader graph; Engine BasicShapes/Plane reused as engine content',
  materials={
   {id='ring',object_path=M.destination..'/M_Ring.M_Ring',shape='ring'},
   {id='ribbon',object_path=M.destination..'/M_Ribbon.M_Ribbon',shape='ribbon'},
   {id='disc',object_path=M.destination..'/M_Disc.M_Disc',shape='disc'},
  }}
end
function M.verify_result(plan,value)
 assert(value.schema=='omoba.combat-visual-assets-result.v1'and value.success==true and value.error=='')
 assert(value.graph_version==plan.graph_version and value.render_verified==false)
 assert(type(value.materials)=='table'and #value.materials==#plan.materials)
 local wanted={};for _,row in ipairs(plan.materials)do wanted[row.id]=row end
 local packages,imported,verified={},0,0
 for _,row in ipairs(value.materials)do
  local job=assert(wanted[row.id],'unexpected or duplicate material');wanted[row.id]=nil
  assert(row.object_path==job.object_path and row.shape==job.shape and row.graph_verified==true and row.compiled_shader==true)
  assert(row.unlit==true and row.two_sided==true and row.translucent==true and row.instance_fade==true)
  assert(row.signature==plan.graph_version..'|'..row.id)
  if row.action=='imported'then imported=imported+1 else assert(row.action=='verified');verified=verified+1 end
  packages[#packages+1]=assert(row.object_path:match('^(.-)%.M_[A-Za-z]+$'))
 end
 assert(next(wanted)==nil and value.imported==imported and value.verified==verified and value.saved==imported)
 return packages
end
return M
