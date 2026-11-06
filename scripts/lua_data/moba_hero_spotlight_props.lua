-- Hero spotlight stage with props (E630): the hero spotlight stage (moba_hero_spotlight.lua) plus
-- what an ability's own cast condition needs on the stage. record_hero_spotlight.recipe_for picks
-- it for a hero with an aim marked `props` (item_spotlight.AIM: ally_anchor_point, self_area_corpses).
--   * an allied creep (player 1's team) that stands still and never fights: C19 奇襲 must target a
--     point with an allied non-hero unit within 300; the renderer aims at the nearest allied creep.
--     400 up and 240 left of the demonstrator, on screen, 400 from the ally stand-in and 625 from the
--     enemy stand-in (far outside its 128 attack range).
--   * an enemy creep (player 6's team), the allied one mirrored 400 down: an enemy cast that only
--     works on non-hero units (c06_old 殺人越貨 kills a creep or summon, a hero sees only its effect)
--     is aimed at it (team enemy_anchor, record_hero_spotlight.ENEMY_ANCHOR). 466 from the
--     demonstrator, inside that cast's 500; props are public, so the demonstrator sees it.
--   * a body (player 6's team): B23 亡靈之契 needs a corpse within 600 of the caster. 200 up of the
--     demonstrator; the authority keeps it findable by dead selectors for the whole match.
-- Server-owned recipe data only.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local stage=assert(loadfile(dir..'/moba_hero_spotlight.lua'))()
stage.showcase.props={
  {kind='unit',team_of=1,at={1900,2160}},
  {kind='corpse',team_of=6,at={2100,2400}},
  {kind='unit',team_of=6,at={2700,2160}},
}
return stage
