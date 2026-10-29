-- Match summary from a split run's authority log (E154): how long the match took, who won, when each
-- structure fell and how many deaths each side took. The authority writes one "[match]" line per
-- structure loss, hero death and the result (omoba-core moba_match.rs).
--
--   tools\lua\lua.exe scripts\match_summary.lua --run target\mvp-a3\<run> [--tick-rate 60] [--json]
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

function M.parse(text)
  local out={structures={},deaths={[0]=0,[1]=0},death_ticks={},finished=nil,winner=nil,mismatches=0}
  for line in text:gmatch('[^\r\n]+') do
    local tick,side,role,lane=line:match('%[match%] structure_destroyed tick=(%d+) side=(%d+) role=(%w+) lane=(%d+)')
    if tick then
      out.structures[#out.structures+1]={tick=tonumber(tick),side=tonumber(side),role=role,lane=tonumber(lane)}
    end
    local dtick,player,dside=line:match('%[match%] hero_death tick=(%d+) player=(%d+) side=(%d+)')
    if dtick then
      dside=tonumber(dside)
      out.deaths[dside]=(out.deaths[dside] or 0)+1
      out.death_ticks[#out.death_ticks+1]={tick=tonumber(dtick),player=tonumber(player),side=dside}
    end
    local ftick,winner=line:match('%[match%] finished tick=(%d+) winner_side=(.+)$')
    if ftick and not out.finished then
      out.finished=tonumber(ftick)
      out.winner=winner:match('Some%((%d)%)')
      out.winner=out.winner and tonumber(out.winner) or 'none'
    end
    if line:find('observer mismatch',1,true) then out.mismatches=out.mismatches+1 end
  end
  table.sort(out.structures,function(a,b) return a.tick<b.tick end)
  return out
end

local function read(path)
  local file=io.open(path,'rb'); if not file then return nil end
  local text=file:read('a'); file:close(); return text
end

function M.summary(run,tick_rate)
  tick_rate=tick_rate or 60
  local text
  for _,name in ipairs({'server.log','server/server.stdout.log'}) do
    text=read(run..'/'..name)
    if text and text:find('[match]',1,true) then break end
  end
  assert(text,'no authority log under '..run)
  local s=M.parse(text)
  local function seconds(tick) return tick and tick/tick_rate end
  s.duration_seconds=seconds(s.finished)
  s.first_tower_seconds=nil
  for _,row in ipairs(s.structures) do
    row.seconds=seconds(row.tick)
    if row.role=='Tower' and not s.first_tower_seconds then s.first_tower_seconds=row.seconds end
  end
  return s
end

function M.format(s)
  local lines={}
  local function add(fmt,...) lines[#lines+1]=string.format(fmt,...) end
  add('duration: %s   winner side: %s   first tower: %s',
    s.duration_seconds and string.format('%.0fs (%d:%02d)',s.duration_seconds,s.duration_seconds//60,math.floor(s.duration_seconds%60)) or 'not finished',
    tostring(s.winner), s.first_tower_seconds and string.format('%.0fs',s.first_tower_seconds) or 'none')
  add('hero deaths: side0=%d side1=%d   observer mismatch: %d', s.deaths[0] or 0, s.deaths[1] or 0, s.mismatches)
  for _,row in ipairs(s.structures) do add('  %6.0fs side%d %-10s lane %d',row.seconds,row.side,row.role,row.lane) end
  return table.concat(lines,'\n')
end

if arg and arg[0] and arg[0]:match('match_summary%.lua$') then
  local run,rate,json
  local i=1
  while i<=#arg do
    if arg[i]=='--run' then i=i+1;run=arg[i]
    elseif arg[i]=='--tick-rate' then i=i+1;rate=tonumber(arg[i])
    elseif arg[i]=='--json' then json=true
    else error('unknown argument: '..arg[i]) end
    i=i+1
  end
  assert(run,'usage: match_summary.lua --run DIR [--tick-rate 60] [--json]')
  local s=M.summary(run,rate)
  if json then
    local b=require('_bootstrap');print(b.lib('json').encode(s))
  else print(M.format(s)) end
end
return M
