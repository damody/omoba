local script=debug.getinfo(1,'S').source:sub(2)
local dir=script:match('^(.*)[/\\]')
package.path=dir..'/../?.lua;'..package.path
local summary=require('match_summary')

local s=summary.parse(table.concat({
  '10-27 [INFO] - [match] structure_destroyed tick=6000 side=1 role=Tower lane=0   (omoba_core x)',
  '10-27 [INFO] - [match] hero_death tick=3600 player=6 side=1 deaths=1   (omoba_core x)',
  '10-27 [INFO] - [match] hero_death tick=4200 player=2 side=0 deaths=1   (omoba_core x)',
  '10-27 [INFO] - [match] hero_death tick=4800 player=7 side=1 deaths=1   (omoba_core x)',
  '10-27 [INFO] - [match] structure_destroyed tick=3000 side=0 role=Tower lane=2   (omoba_core x)',
  '10-27 [ERROR] - observer mismatch team=2 replica_tick=1',
  '10-27 [INFO] - [match] finished tick=30000 winner_side=Some(1)   (omoba_core x)',
},'\n'))
assert(s.deaths[0]==1 and s.deaths[1]==2,'deaths per side')
assert(#s.structures==2 and s.structures[1].tick==3000,'structures sorted by tick')
assert(s.finished==30000 and s.winner==1,'result')
assert(s.mismatches==1,'mismatch count')
local none=summary.parse('[match] finished tick=10 winner_side=None')
assert(none.winner=='none','a draw has no winner side')
print('match summary: 5 checks passed')
