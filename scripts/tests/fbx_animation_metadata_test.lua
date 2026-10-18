package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap');local p=b.lib('path');local reader=require('fbx_animation_metadata')
local ctx={};function ctx.include(file)local value=dofile(p.join(b.root,'scripts/lua_data',file));return type(value)=='function'and value(ctx)or value end
local heroes=ctx.include('templates/heroes.lua');local count=0
for _,hero in ipairs(heroes)do if hero.render and hero.render.animation_sources then
 for slot,source in pairs(hero.render.animation_sources)do
  local bytes=assert(p.read(p.join(b.root,'scripts/lua_data',source.model),true));local result=reader.read(bytes)
  assert(#result.stacks==1);local stack=result.stacks[1]
  assert(math.abs(stack.start_seconds-source.timeline_offset_ticks/source.ticks_per_second)<0.000001,'source timeline mismatch: '..slot)
  assert(math.abs(stack.seconds-source.duration_ticks/source.ticks_per_second)<0.000001,'source duration mismatch: '..slot);count=count+1
 end
end end
assert(count==11,'authored source metadata coverage changed')
local bytes=assert(p.read(p.join(b.root,'scripts/lua_data/templates/heroes/saika_magoichi/b01_ani_cast.fbx'),true))
for _,invalid in ipairs({'invalid',bytes:sub(1,200),bytes:sub(1,23)..string.pack('<I4',9000)..bytes:sub(28),bytes:sub(1,27)..string.pack('<I4',#bytes+100)..bytes:sub(32)})do
 assert(not pcall(reader.read,invalid),'malformed binary FBX must fail')
end
print('PASS actual authored source metadata: '..count..' bindings and 4 malformed inputs')
