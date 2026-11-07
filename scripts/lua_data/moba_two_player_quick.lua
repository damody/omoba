-- Same two-human seats as moba_two_player.lua on the live-check map: identical layout and
-- rules with harder-hitting waves and lighter base structures, so towers, the base and the
-- end-of-match path are exercised inside a 10-minute run (run_moba_split_local --max-seconds 600).
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_two_player.lua'))()
plan.map_id='three_lane_quick_training'
return plan
