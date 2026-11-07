package.path='scripts/?.lua;'..package.path
local dream=require('dream_assets')

local found=dream.orphans({'Content/Fx/Textures/T_a.uasset','Content/Fx/Textures/T_b.uasset'},{['Content/Fx/Textures/T_a.uasset']=true})
assert(#found==1 and found[1]:find('T_b.uasset',1,true) and found[1]:find('fx_family.lua',1,true))
assert(#dream.orphans({'Content/Fx/Textures/T_a.uasset'},{['Content/Fx/Textures/T_a.uasset']=true})==0)

-- Every image the list names is an owned asset, and every folder ignored as a whole is one the check reads.
local owned=dream.owned_assets('omfue')
for _,row in ipairs(require('fx_family').image_assets()) do assert(owned[row.asset],row.asset) end
local whole={}
for _,dir in ipairs(require('dream_untrack').WHOLE) do whole[dir]=true end
assert(whole['Content/Omoba/VFX/Fx/Textures'])

local platform=require('_bootstrap').lib('platform')
assert(platform.ue_root('E:/Engine')=='E:/Engine')
assert(type(platform.ue_root())=='string')
print('dream_orphans_test ok')
