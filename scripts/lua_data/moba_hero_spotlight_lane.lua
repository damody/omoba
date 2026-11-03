-- Hero spotlight stage for a hero whose abilities cross a long way (A14 前田利家): the usual 240 spacing
-- along y, at the end of the one clear run of the map (x 2800, y 1350 to 3200, walls 80 away at their
-- nearest), so a 1400 move stays on flat ground, out of reach (800) of the mid towers at (3244,3244) and (1555,1555). The death's stand-ins strike whoever comes near, so they
-- stand 800 off that run, where the recorder's LANE_DEATH_AT walks the hero; the ally is off the run too,
-- within sight of the death, so its enemies stay seen once the hero is down. Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
local places=stage.showcase.placements
places['1']={2800,2500}
places['6']={2800,2260}
places['2']={2620,2260}
places['7']={2100,2230}
places['8']={2100,2570}
places['9']={1980,2600}
places['10']={2320,2640}
return stage
