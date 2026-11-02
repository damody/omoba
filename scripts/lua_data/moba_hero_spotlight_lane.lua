-- Hero spotlight stage for a hero whose abilities cross a long way (A14 前田利家): the stand-ins line up
-- along x, up the screen, where the top lane is open past the far end of the base stage's cross-lane line.
-- Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
local places=stage.showcase.placements
places['6']={2060,2400}
places['2']={2540,2400}
return stage
