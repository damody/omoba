-- Hero spotlight stage with the enemy and ally stand-ins swapped: the enemy stands on the side away
-- from the north river wall (y 2700+), for a hero who throws the enemy a long way. Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
local places=stage.showcase.placements
places['2'],places['6']=places['6'],places['2']
return stage
