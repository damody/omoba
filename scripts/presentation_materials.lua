-- Solid source materials are distinct from texture atlases. Values below are
-- original FBX Material 03 DiffuseColor, not sampled from a screenshot.
local M={}
function M.plan()
 local b=require('_bootstrap');local p,h=b.lib('path'),b.lib('hash')
 local source='scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx'
 local sha='e931732dd8516e5ce3075394438d7b5289f739c110587300810607d4ab86eeb0'
 assert(h.sha256(p.join(b.root,source))==sha,'source material provenance changed; re-inspect FBX before updating values')
 return {schema='omoba.presentation-materials-plan.v1',materials={{name='M_SaikaCloth',
  object_path='/Game/Omoba/PresentationMaterialsV1/M_SaikaCloth.M_SaikaCloth',
  provenance=source..'|sha256='..sha..'|Material:2171624864|DiffuseColor; no texture connection',
  linear_rgb={1,0.01176470797509,0.01176470797509},roughness=1,specular=0,two_sided=true}}}
end
return M
