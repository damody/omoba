-- Hero spotlight stage for tree-target abilities (B22 自然之力, 大自然的呼喚): the hero spotlight
-- stage (moba_hero_spotlight.lua) on the original nobu map, whose trees the live-check map does not
-- have. The demonstrator stands at (-1400,-600): three public trees within 169-265 of it (in reach
-- of a 300 cast range), at least 1600 from every tower, barracks, shop, boss and camp and 300 clear
-- of terrain; the stand-ins keep the 240 spacing of the base stage. Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
stage.map_id='nobu'
stage.showcase.placements={['1']={-1400,-600},['2']={-1400,-840},['6']={-1400,-360}}
return stage
