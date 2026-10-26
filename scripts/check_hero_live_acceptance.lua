-- Per-ability live verdict for a split run: every ability of each human
-- (autopilot) hero must show up in the server log and in that player's
-- autopilot evidence. Expectations come from the hero's package
-- (docs/character_pipeline/packages/*), never from a hand-kept table:
--   active / toggle / ultimate: `[scripting] cast ok ability='<id>'` in the
--     server log and an accepted autopilot cast intent for its slot;
--   passive: `[scripting] learn ability='<id>'` (auto_learn passives at birth);
--   auto_learn gated above --reachable-level (default 12): NOT_REACHABLE_IN_10MIN
--     unless the run reached it anyway (then it is checked like the others).
--   never learned and its rank 1 needs a hero level above the level the hero
--     ended the match at (autopilot match_finished hero_level): INCONCLUSIVE_RUN_SHORT,
--     the run proved nothing about it; the verdict is INCONCLUSIVE (exit 2), rerun.
-- Run-wide gates: no observer mismatch, no secure-match termination.
-- Usage: tools/lua/lua.exe scripts/check_hero_live_acceptance.lua --run SPLIT_RUN_DIR [--reachable-level N] [--packages DIR]
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}

M.REACHABLE_LEVEL=12
M.SLOT={Q='q',W='w',E='e',R='r',extra1='extra'}

local function read(file) return path.is_file(file) and path.read(file) or nil end
local function count(text,needle)
  local n,at=0,1
  while text do local i=text:find(needle,at,true); if not i then break end; n=n+1; at=i+#needle end
  return n
end

-- Package data files run with only pairs/ipairs, like hero_author's loader.
local function load_data(file)
  local text=read(file); if not text then return nil end
  local chunk=assert(load(text,'@'..file,'t',{pairs=pairs,ipairs=ipairs}))
  return chunk()({})
end

-- hero id -> { package, abilities } for every package with a character.lua.
function M.kits(packages_root)
  local kits={}
  local lfs=require('lfs')
  if not path.is_directory(packages_root) then return kits end
  for name in lfs.dir(packages_root) do
    local dir=packages_root..'/'..name
    if name:sub(1,1)~='.' and path.is_directory(dir) then
      local character=load_data(dir..'/character.lua')
      local abilities=load_data(dir..'/abilities.lua')
      if character and abilities and character.hero then
        kits[character.hero.id]={package=name,abilities=abilities.abilities or {}}
      end
    end
  end
  return kits
end

-- Accepted autopilot cast intents per slot name, from the coverage summary
-- when present, otherwise from the trace's authority acks.
function M.accepted_casts(player_dir,player)
  local out={}
  local function add(intent,n)
    local slot=intent:match('^cast_([a-z]+)_')
    if slot and n>0 then out[slot]=(out[slot] or 0)+n end
  end
  local coverage=path.join(player_dir,('autopilot-p%d.coverage.json'):format(player))
  if path.is_file(coverage) then
    for intent,row in pairs(json.read(coverage).intents or {}) do add(intent,row.ack_accepted or 0) end
    return out,'coverage'
  end
  local trace=read(path.join(player_dir,('autopilot-p%d.jsonl'):format(player)))
  for line in (trace or ''):gmatch('[^\r\n]+') do
    if line:find('"event":"authority"',1,true) and line:find('"ack_result":"accepted"',1,true) then
      local intent=line:match('"autopilot_intent":"([^"]+)"')
      if intent then add(intent,1) end
    end
  end
  return out,trace and 'trace' or 'missing'
end

-- Hero level at the end of the match, from the autopilot trace (nil if unknown).
function M.final_hero_level(player_dir,player)
  local trace=read(path.join(player_dir,('autopilot-p%d.jsonl'):format(player)))
  local level
  for line in (trace or ''):gmatch('[^\r\n]+') do
    if line:find('"event":"match_finished"',1,true) then level=tonumber(line:match('"hero_level":(%d+)')) or level end
  end
  return level
end

local function first_level(ab)
  local levels=ab.required_hero_level
  return type(levels)=='table' and levels[1] or 1
end

function M.check(root,options)
  options=options or {}
  local reachable=tonumber(options.reachable_level) or M.REACHABLE_LEVEL
  local packages=options.packages or path.join(b.root,'docs','character_pipeline','packages')
  local kits=M.kits((packages:gsub('\\','/')))
  local rows={}
  local function row(player,hero,id,status,detail) rows[#rows+1]={player=player,hero=hero,ability=id,status=status,detail=detail} end
  local plan_file=path.join(root,'server','match-plan.json')
  local plan=path.is_file(plan_file) and json.read(plan_file) or nil
  local server=read(path.join(root,'server','server.stdout.log')) or ''
  if not plan then row('-','-','match-plan','FAIL','missing '..plan_file) end
  -- A hero shared with another player makes server lines ambiguous.
  local hero_count={}
  for _,p in ipairs(plan and plan.players or {}) do hero_count[p.hero]=(hero_count[p.hero] or 0)+1 end
  for _,p in ipairs(plan and plan.players or {}) do
    if p.bot==false then
      local kit=kits[p.hero]
      if not kit then
        row(p.player_id,p.hero,'-','SKIP','no authored package for this hero')
      else
        local casts,where=M.accepted_casts(path.join(root,'p'..p.player_id),p.player_id)
        local final_level=M.final_hero_level(path.join(root,'p'..p.player_id),p.player_id)
        local shared=hero_count[p.hero]>1 and ' (hero shared with another player: server lines are not attributed)' or ''
        for _,ab in ipairs(kit.abilities) do
          local slot=M.SLOT[ab.action]
          local cast_ok=count(server,("cast ok ability='%s'"):format(ab.id))
          local learned=count(server,("learn ability='%s'"):format(ab.id))
          local errors=count(server,("ability '%s' execute returned error"):format(ab.id))
          local accepted=slot and casts[slot] or 0
          local detail=('cast_ok=%d learn=%d handler_errors=%d autopilot_accepted=%d (%s)%s'):format(cast_ok,learned,errors,accepted,where,shared)
          local gated=ab.auto_learn and first_level(ab)>reachable
          if gated and learned==0 then
            row(p.player_id,p.hero,ab.id,'NOT_REACHABLE_IN_10MIN',('auto_learn at hero level %d > %d; %s'):format(first_level(ab),reachable,detail))
          elseif learned==0 and final_level and first_level(ab)>final_level then
            row(p.player_id,p.hero,ab.id,'INCONCLUSIVE_RUN_SHORT',('rank 1 needs hero level %d, match ended at %d; %s'):format(first_level(ab),final_level,detail))
          elseif ab.ability_type=='passive' then
            row(p.player_id,p.hero,ab.id,learned>0 and 'PASS' or 'FAIL','passive: needs a learn line; '..detail)
          else
            row(p.player_id,p.hero,ab.id,(cast_ok>0 and accepted>0) and 'PASS' or 'FAIL','cast: needs cast ok and an accepted autopilot cast; '..detail)
          end
        end
      end
    end
  end
  local mismatch,terminated=count(server,'observer mismatch'),count(server,'secure match terminated')
  row('-','-','run:no-observer-mismatch',mismatch==0 and 'PASS' or 'FAIL',('count=%d'):format(mismatch))
  row('-','-','run:no-termination',terminated==0 and 'PASS' or 'FAIL',('count=%d'):format(terminated))
  local ok,checked,inconclusive=true,0,false
  for _,r in ipairs(rows) do
    if r.status=='FAIL' then ok=false end
    if r.status=='INCONCLUSIVE_RUN_SHORT' then inconclusive=true end
    if r.status=='PASS' and r.ability:sub(1,4)~='run:' then checked=checked+1 end
  end
  ok=ok and checked>0
  return ok,rows,ok and inconclusive
end

if arg and arg[0] and arg[0]:match('check_hero_live_acceptance%.lua$') then
  local options=b.lib('args').parse(arg)
  local root=assert(options.run,'--run SPLIT_RUN_DIR required')
  local ok,rows,inconclusive=M.check(path.absolute(root,b.root),{reachable_level=options['reachable-level'],packages=options.packages})
  for _,r in ipairs(rows) do print(('%-22s p%-3s %-16s %-30s %s'):format(r.status,tostring(r.player),r.hero,r.ability,r.detail or '')) end
  if inconclusive then print('hero live acceptance verdict: INCONCLUSIVE (match ended before some ranks were reachable; rerun)');os.exit(2) end
  print(ok and 'hero live acceptance verdict: PASS' or 'hero live acceptance verdict: FAIL')
  os.exit(ok and 0 or 1)
end
return M
