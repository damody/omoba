local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash');local M=require('ue_map_biome_materials')
local root=p.join(b.root,'.ai-collab','biome-material-state-test-'..b.lib('time').monotonic_ms()..'-'..math.random(100000,999999));assert(not p.exists(root))
local count=0;local function check(value,why)assert(value,why);count=count+1 end
local function write(file,text)p.mkdir_p(p.parent(p.join(root,file)));p.write(p.join(root,file),text,true,true)end
local producer='omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmMapBiomeMaterialsCommandlet.cpp'
local folder='omfue/Content/Omoba/MapBiomesV1';local provenance=folder..'/provenance.json'
local textures={grass='omfue/Content/Omoba/WorldSurfacesV1/T_grass.uasset',stone='omfue/Content/Omoba/WorldSurfacesV1/T_stone.uasset'}
write(producer,'producer-v1');local materials={};local world={}
for _,id in ipairs({'jungle','bank','river'})do local file=folder..'/M_'..id..'.uasset';write(file,id);materials[#materials+1]={id=id,package_file=file,sha256=h.sha256(p.join(root,file))}end
for id,file in pairs(textures)do write(file,id);world[#world+1]={package_file=file,sha256=h.sha256(p.join(root,file))}end
local created={schema='omoba.map-biome-material-provenance/1',producer_sha256=h.sha256(p.join(root,producer)),materials=materials,world_textures=world}
j.write(p.join(root,provenance),created,false)
check(M.ready(root),'portable creation provenance validates the complete required inputs')
write(folder..'/M_river.uasset','tampered');check(not M.ready(root),'changed package rejected')
check(not pcall(M.require_ready,root),'no-build requirement fails explicitly')
check(p.read(p.join(root,folder..'/M_river.uasset'),true)=='tampered','readiness check never repairs a changed package')
write(folder..'/M_river.uasset','river')
created.materials[1].package_file='../outside';j.write(p.join(root,provenance),created,true)
check(not M.ready(root),'foreign metadata path rejected before reading it')
created.materials[1].package_file=folder..'/M_jungle.uasset';j.write(p.join(root,provenance),created,true)
write(producer,'producer-v2');check(not M.ready(root),'changed producer requires actual native verification')
local verified={schema='omoba.map-biome-verification/1',producer_sha256=h.sha256(p.join(root,producer)),materials={},textures={}}
for _,row in ipairs(materials)do verified.materials[row.id]=row.sha256 end
for id,file in pairs(textures)do verified.textures[id]=h.sha256(p.join(root,file))end
local receipt='omfue/Saved/BuildTemp/map-biome-materials.json';p.mkdir_p(p.parent(p.join(root,receipt)));j.write(p.join(root,receipt),verified,false)
check(M.ready(root),'native verification receipt can bind a new producer without rewriting assets')
write(textures.grass,'changed grass');check(not M.ready(root),'receipt does not hide changed texture input')
check(p.read(p.join(root,textures.grass),true)=='changed grass','failed check does not mutate original texture')
write(textures.grass,'grass');write(producer,'producer-v1');write(receipt,'{invalid-json')
check(M.ready(root),'corrupt optional cache falls back to permanent creation provenance')
assert(os.rename(p.join(root,folder..'/M_bank.uasset'),p.join(root,folder..'/M_bank.disabled')))
check(not M.ready(root),'missing required package rejected')
print(('PASS map biome material state: %d checks'):format(count))
