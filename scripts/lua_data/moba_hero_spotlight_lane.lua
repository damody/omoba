-- Hero spotlight stage for a hero whose abilities cross a long way (A14 前田利家): the usual 240 spacing
-- along y, at the end of the one clear run of the map (x 2800, y 1350 to 3200, walls 80 away at their
-- nearest), so a 1400 move stays on flat ground. The death's stand-ins move off that line. Server-owned
-- recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
local places=stage.showcase.placements
places['1']={2800,3050}
places['6']={2800,2810}
places['2']={2800,3290}
places['7']={3030,2300}
places['8']={3030,2500}
places['9']={3030,2190}
places['10']={3030,2610}
return stage
