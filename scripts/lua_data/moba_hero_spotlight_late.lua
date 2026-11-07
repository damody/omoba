-- Hero spotlight stage started at hero level 18 (record_hero_spotlight.LATE_RECIPE): the hero
-- spotlight stage (moba_hero_spotlight.lua) for a hero whose ability is learned only above its
-- start level 16. A04D_old 冰雪爆 and B19D_old 駿河守 are NOT_LEARNABLE in the source and set to
-- rank 1 by a timer at hero level 18 (hero.auto_learn with requires.hero_level 18): at level 16
-- their spotlight segments were skipped at rank 0.
-- Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
stage.showcase.start_level=18
return stage
