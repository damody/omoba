local M={destination='/Game/Omoba/WorldSurfacesV1/'}
function M.plan(projection)
 projection=projection or'world_xy_v1';assert(projection=='world_xy_v1'or projection=='world_triplanar_v1'or projection=='world_boundary_v1','unsupported projection')
 local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
 local manifest=j.read(p.join(b.root,'scripts/art/world_surfaces/v1/manifest.json'))
 assert(manifest.schema=='omoba.world-surface-art.v1'and #manifest.assets==2)
 local names={['grass-ground-v1']='grass',['stone-paving-v1']='stone'};local rows={}
 for _,row in ipairs(manifest.assets)do
  local id=assert(names[row.id]);names[row.id]=nil
  local file=p.absolute(row.source,b.root);assert(h.sha256(file)==row.sha256,'source art changed')
  local bytes=assert(p.read(file,true));assert(bytes:sub(1,8)=='\137PNG\13\10\26\10');local width,height=string.unpack('>I4I4',bytes,17)
  assert(width==row.width and height==row.height and width<=2048 and height<=2048 and row.generator=='built-in image_gen')
  rows[#rows+1]={id=id,source=file:gsub('\\','/'),sha256=row.sha256,width=width,height=height,repeat_cm=4000,
   texture=M.destination..'T_'..id..'.T_'..id,material=M.destination..'M_'..id..'.M_'..id}
 end
 assert(next(names)==nil)
 if projection=='world_triplanar_v1' then
  local stone;for _,row in ipairs(rows)do if row.id=='stone'then stone=row end end
  assert(stone);stone.projection=projection;stone.material=M.destination..'M_stone_terrain.M_stone_terrain';rows={stone}
 elseif projection=='world_boundary_v1' then
  local grass;for _,row in ipairs(rows)do if row.id=='grass'then grass=row end end
  assert(grass);grass.projection=projection;grass.material=M.destination..'M_grass_boundary.M_grass_boundary';rows={grass}
 end
 return {schema='omoba.world-surfaces-plan.v1',surfaces=rows}
end
return M
