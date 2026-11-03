-- Hero spotlight stage for a hero whose abilities pierce (A12 大谷吉繼): a held enemy 240 behind the stand-in, the three
-- in a line across the frame (y runs across the screen). Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
-- Held, never striking: what the arrow reaches after the stand-in.
stage.showcase.placements['10']={2300,2880}
stage.showcase.strike_players={7,8,9}
return stage
