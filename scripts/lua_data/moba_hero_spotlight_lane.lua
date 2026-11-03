-- Hero spotlight stage for a hero whose abilities cross a long way (A14 前田利家): the usual 240 spacing
-- along y, at the end of the one clear run of the map (x 2800, y 1350 to 3200, walls 80 away at their
-- nearest), so a 1400 move stays on flat ground. The death's stand-ins strike whoever comes near, so they
-- stand 800 off that run, where the recorder's LANE_DEATH_AT walks the hero; the ally is off the run too,
-- within sight of the death, so its enemies stay seen once the hero is down. Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
local places=stage.showcase.placements
places['1']={2800,3050}
places['6']={2800,2810}
places['2']={2620,2810}
places['7']={2000,2230}
places['8']={2000,2570}
places['9']={1870,2190}
places['10']={1870,2610}
return stage
