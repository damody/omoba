-- Hero spotlight videos: each hero casts every ability at an enemy (and ally/self spells on its
-- team) with a caption (key, name, one line from the hero package), recorded from the real
-- authority + renderer. The stage is scripts/lua_data/moba_hero_spotlight.lua; the casts are
-- the renderer autopilot's `spotlight` profile pressing the player's own chords; the window is
-- recorded by the renderer (FOmSpotlightRecorder) and encoded here with ffmpeg to H.264 mp4.
--
--   tools/lua/lua.exe scripts/record_hero_spotlight.lua --hero saika_magoichi
--   tools/lua/lua.exe scripts/record_hero_spotlight.lua --all
--
-- Output: target/spotlight/<hero>/<hero>_spotlight.mp4, keyframes/*.png, spotlight.json (report).
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,json,platform=b.lib('path'),b.lib('process'),b.lib('json'),b.lib('platform')
local lfs=require('lfs')
local M={}

M.HEROES={'tokugawa_ieyasu','yamagata_masakage','okuma_tomohide','sasaki_kojiro','otani_yoshitsugu','maeda_toshiie','nohime','kojima_sadaoki','shimazu_yoshihiro','maeda_keiji'}
-- After the abilities: basic attacks on the stand-in, then the hero's real death. DEATH_AT is where the
-- base stage's held enemies stand (scripts/lua_data/moba_hero_spotlight.lua).
M.DEATH_AT={3000,2400}
-- An ability only castable during another is shown right after it, or that one has run out by its turn.
M.SHOWN_AFTER={a12_mystic_arrow='a12_mystic_possession'}
M.BODY_SHOWS={
  {kind='attack',name='普通攻擊',line='對敵方連續普攻',team='enemy'},
  {kind='death',name='死亡',line='被敵方擊倒',team='self',at=M.DEATH_AT},
}
M.SLOT_OF_ACTION={Q=0,W=1,E=2,R=3,extra1=4,extra2=5}
M.RECIPE='scripts/lua_data/moba_hero_spotlight.lua'
-- A hero whose abilities cross a long way: the stand-ins line up along the open top lane.
M.LANE_RECIPE='scripts/lua_data/moba_hero_spotlight_lane.lua'
M.LANE_DEATH_AT={2150,2400}
M.LANE_HEROES={maeda_toshiie=true}
-- A hero with a tree-target ability is shown on the nobu map, which has trees (D21: every ability
-- is demonstrated).
M.TREE_RECIPE='scripts/lua_data/moba_hero_spotlight_trees.lua'
-- A hero with an ability whose own cast condition needs a stage prop (E630: C19 奇襲 an allied
-- non-hero unit at its point, B23 亡靈之契 a body near the caster; item_spotlight.AIM `props`) is
-- shown on the prop stage: the base stage plus an allied creep and a body.
M.PROPS_RECIPE='scripts/lua_data/moba_hero_spotlight_props.lua'
-- A hero struck or slowing a walker in its passive: the base stage plus a patrol out of every other segment's view,
-- and whole again whenever no striker reaches it (refill_players).
M.PATROL_RECIPE='scripts/lua_data/moba_hero_spotlight_patrol.lua'
M.PATROL_HEROES={maeda_keiji=true,okuma_tomohide=true,tokugawa_ieyasu=true}
-- A hero whose passive answers to a spell: the base stage plus seat 6's scheduled casts in one circle.
M.CASTS_RECIPE='scripts/lua_data/moba_hero_spotlight_casts.lua'
M.CASTS_HEROES={yamagata_masakage=true}
function M.recipe_for(package)
  local tree,props=false,package.props==true
  for _,slot in ipairs(package.slots) do
    if slot.team=='tree' then tree=true end
    if slot.team=='ally_anchor' then props=true end
  end
  assert(not (tree and props),(package.hero or 'hero')..' needs both the tree stage and the prop stage (no stage has both yet)')
  if tree then return M.TREE_RECIPE end
  if props then return M.PROPS_RECIPE end
  if M.PATROL_HEROES[package.hero or ''] then return M.PATROL_RECIPE end
  if M.CASTS_HEROES[package.hero or ''] then return M.CASTS_RECIPE end
  return M.LANE_HEROES[package.hero or ''] and M.LANE_RECIPE or M.RECIPE
end
M.DEFAULT_FFMPEG='target/tools/ffmpeg/ffmpeg.exe'
-- A spotlight run owns its authority port: never the shared split default (57061), so a
-- recording can neither collide with nor join another session's live match.
M.DEFAULT_PORT=57461
M.FFMPEG_SOURCE='BtbN FFmpeg-Builds (github.com/BtbN/FFmpeg-Builds/releases, ffmpeg-master-latest-win64-gpl.zip) or gyan.dev'

function M.options(args)
  local options={heroes={},output='target/spotlight',ffmpeg=M.DEFAULT_FFMPEG,ue_root='D:/UE5.8',fps=30,max_seconds=300,port=M.DEFAULT_PORT}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--all' then options.all=true
    elseif key=='--no-build' then options.no_build=true
    elseif key=='--encode-only' then options.encode_only=true
    elseif key=='--observer-only' then options.observer_only=true
    elseif key=='--hero' then
      i=i+1
      local hero=assert(args[i],'--hero requires a hero id')
      assert(hero:match('^[%w_]+$'),'--hero requires a hero id')
      options.heroes[#options.heroes+1]=hero
    elseif key=='--output' or key=='--ffmpeg' or key=='--ue-root' or key=='--run' then
      i=i+1;options[key:sub(3):gsub('%-','_')]=assert(args[i],key..' requires a value')
    elseif key=='--port' then
      i=i+1;options.port=assert(math.tointeger(tonumber(args[i] or '')),'--port must be an integer')
      assert(options.port>=1024 and options.port<=65525 and options.port~=57061,'--port must be 1024..65525 and not the shared 57061')
    elseif key=='--max-seconds' then
      i=i+1;options.max_seconds=assert(math.tointeger(tonumber(args[i] or '')),'--max-seconds must be an integer')
      assert(options.max_seconds>=60 and options.max_seconds<=600,'--max-seconds must be 60..600 (a live run stays under 10 minutes)')
    else error('unknown argument: '..key) end
    i=i+1
  end
  if options.all then
    assert(#options.heroes==0,'--all and --hero are exclusive')
    for _,hero in ipairs(M.HEROES) do options.heroes[#options.heroes+1]=hero end
  end
  assert(#options.heroes>0,'choose --hero HERO (repeatable) or --all')
  assert(not options.encode_only or (#options.heroes==1 and options.run),'--encode-only re-encodes one --hero from an existing --run directory')
  return options
end

local function chars_of(text)
  local chars={}
  for _,code in utf8.codes(text) do chars[#chars+1]=utf8.char(code) end
  return chars
end

-- One caption line from a package description: the type prefix (被動／切換／引導) and
-- parenthetical details dropped, per-rank lists shown as a range (200/300/400 -> 200～400),
-- then whole clauses of the first sentence until the line says enough (MIN characters),
-- capped at MAX characters (UTF-8 code points) with an ellipsis.
function M.first_line(text,max,min)
  max,min=max or 44,min or 14
  text=tostring(text or ''):gsub('%s+',' '):gsub('^ ',''):gsub(' $','')
  for _,prefix in ipairs({'被動光環：','被動：','切換：','引導：'}) do
    if text:sub(1,#prefix)==prefix then text=text:sub(#prefix+1) break end
  end
  -- Per-rank lists are ASCII digits, so byte patterns are safe here.
  text=text:gsub('([%+%-]?%d+%.?%d*)(%%?)/[%d%./]*/([%d%.]+)(%%?)',function(first,unit,last,unit2)
    return first..unit..'～'..last..unit2
  end)
  text=text:gsub('([%+%-]?%d+%.?%d*)(%%?)/([%d%.]+)(%%?)',function(first,unit,last,unit2)
    return first..unit..'～'..last..unit2
  end)
  -- Multi-byte punctuation is handled per code point, never with byte classes.
  local clauses,current,depth={},{},0
  for _,char in ipairs(chars_of(text)) do
    if char=='（' or char=='(' then depth=depth+1
    elseif char=='）' or char==')' then depth=math.max(depth-1,0)
    elseif depth==0 then
      if char=='。' then break end
      if char=='，' or char=='；' or char=='：' then
        clauses[#clauses+1]=table.concat(current);current={}
      else current[#current+1]=char end
    end
  end
  clauses[#clauses+1]=table.concat(current)
  local line=''
  for _,clause in ipairs(clauses) do
    clause=clause:gsub('^ +',''):gsub(' +$','')
    if clause~='' then
      line=line..(line=='' and '' or '，')..clause
      if utf8.len(line)>=min then break end
    end
  end
  local chars=chars_of(line)
  if #chars<=max then return line end
  return table.concat(chars,'',1,max-1)..'…'
end

-- A package that is only a v3 document (heroes promoted from scripts/v3_test_heroes.lua) has no
-- abilities.lua or character.lua: its slots come from the content catalog (the hero row's ability
-- order, each ability's name, description and type; its team from the v3 bot kind, as the item
-- spotlight aims items). An active the bot declares it cannot aim (bot kind "none", e.g. a tree
-- target) is left out and listed in `unshown`; the other slots keep their numbers.
function M.load_v3_package(root,hero)
  local ctx=require('content_builder').new(path.join(root,'scripts','lua_data'))
  local row
  for _,entry in ipairs(ctx.include('templates/heroes.lua')) do if entry.id==hero then row=entry end end
  assert(row,'hero '..hero..' is not in the content catalog')
  local by_id={}
  for _,ability in ipairs(ctx.include('templates/abilities.lua')) do by_id[ability.id]=ability end
  ITEM_SPOTLIGHT_AS_MODULE=true
  local aim=require('item_spotlight').AIM
  local slots,unshown,props={},{},false
  for index,id in ipairs(assert(row.abilities,'hero without abilities')) do
    local ability=by_id[id]
    local bot=ability and ability.v3 and ability.v3.bot
    if bot and bot.kind=='none' and ability.ability_type~='passive' then
      unshown[#unshown+1]={slot=index-1,id=id,reason='bot kind none'}
    -- "" is an empty slot (an F without a D): nothing to show, the next slot keeps its number.
    elseif id~='' then
    assert(ability,'ability '..id..' of '..hero..' is not in the content catalog')
    local kind=ability.ability_type
    if kind~='passive' and kind~='toggle' and kind~='ultimate' then kind='active' end
    local team='self'
    local target=kind=='passive' or (bot and aim[bot.kind])
    if target~=true then team=target and target.team end
    if not team then
      -- A bot kind the spotlight cannot aim yet is left out and listed, not a failed script for the
      -- hero (E616). ally_anchor_point and self_area_corpses are aimed on the prop stage (E630).
      unshown[#unshown+1]={slot=index-1,id=id,reason='no spotlight aim for bot kind '..tostring(bot and bot.kind)}
    else
    if type(target)=='table' and target.props then props=true end
    slots[#slots+1]={slot=index-1,id=id,name=assert(ability.display_name,'ability without a name'),
      line=M.first_line(ability.description),kind=kind,team=team}
    end
    end
  end
  local doc=M.v3_document(root,hero)
  if doc then slots=M.with_dependents(slots,M.v3_dependents(doc,row.abilities),by_id,aim,unshown) end
  assert(#slots<=16,hero..' has more than 16 demonstrations (FOmSpotlightScript::Parse)')
  -- props: a shown slot needs the prop stage (E630, recipe_for); never part of the renderer script.
  return {hero=hero,hero_name=row.display_name or hero,title=row.title or '',slots=slots,unshown=unshown,props=props}
end

-- E595: the hero's v3 document (docs/character_pipeline/packages/<hero>/v3.lua), loaded with only
-- pure helpers like hero_author/cli.lua safe_env (a package is data); nil without one.
function M.v3_document(root,hero)
  local file=path.join(root,'docs','character_pipeline','packages',hero,'v3.lua')
  if not path.is_file(file) then return nil end
  local env={pairs=pairs,ipairs=ipairs,next=next,select=select,type=type,tostring=tostring,
    tonumber=tonumber,assert=assert,error=error,string=string,table=table,math=math,utf8=utf8}
  local doc=assert(loadfile(file,'t',env))()
  if type(doc)=='function' then doc=doc({}) end
  return doc
end

-- Every op table under NODE in document order (array entries first, then named fields sorted),
-- so add_ability ops keep the order the document grants them in.
local function each_op(node,fn,seen)
  seen=seen or {}
  if type(node)~='table' or seen[node] then return end
  seen[node]=true
  if type(node.op)=='string' then fn(node) end
  for _,value in ipairs(node) do each_op(value,fn,seen) end
  local keys={}
  for key in pairs(node) do if type(key)=='string' then keys[#keys+1]=key end end
  table.sort(keys)
  for _,key in ipairs(keys) do each_op(node[key],fn,seen) end
end

-- E595: what can only be shown after another slot of the hero row (ROW = the row's ability ids):
--   subs[id] = enabler: a row ability that is not learnable, set to a nonzero level only inside one
--     other ability and back to 0 (b26 釋放蓄力波: 蓄力波 sets it to 1 on cast and to 0 after 12 s);
--     one whose level only rises with another (a13 迷蹤步 = 影殺 level + 1) is an ordinary slot.
--   swaps = {id, from, after}: an ability a caster's cast swaps into a row slot (a13 換位之術 for
--     影惑之術 Q, c14 火龍吐息術 for 火吐息術 during 龍化).
--   summons = {template, after, abilities}: a unit template with abilities (its own `abilities` and
--     the add_ability ops of the ability that creates it, in order: b16 月月's 相移...), shown through
--     the player's summon controls after the creating ability (a36 地刺陷阱 → 引爆).
function M.v3_dependents(doc,row)
  local in_row={}
  for index,id in ipairs(row) do if id~='' then in_row[id]=index end end
  local out={subs={},swaps={},summons={}}
  local abilities=doc.abilities or {}
  for _,target in ipairs(abilities) do
    if target.learnable==false and in_row[target.id] then
      local zero,enablers,count=false,{},0
      for _,ability in ipairs(abilities) do
        each_op(ability,function(op)
          if op.op=='set_ability_level' and op.ability==target.id then
            if tonumber(op.level)==0 then zero=true
            elseif ability.id~=target.id and (op.of==nil or op.of=='caster') and not enablers[ability.id] then
              enablers[ability.id]=true;count=count+1
            end
          end
        end)
      end
      -- A level a routine raises (b16 相移 from b16_adjust) is not a cast window.
      local routine=false
      each_op(doc.routines,function(op)
        if op.op=='set_ability_level' and op.ability==target.id and tonumber(op.level)~=0 then routine=true end
      end)
      if zero and count==1 and not routine then out.subs[target.id]=next(enablers) end
    end
  end
  local swapped={}
  for _,ability in ipairs(abilities) do
    if in_row[ability.id] then
      each_op(ability,function(op)
        if op.op=='swap_abilities' and op.on=='caster' and in_row[op.from] and not in_row[op.to] and not swapped[op.to] then
          swapped[op.to]=true
          out.swaps[#out.swaps+1]={id=op.to,from=op.from,slot=in_row[op.from]-1,after=ability.id}
        end
      end)
    end
  end
  for _,template in ipairs(doc.unit_templates or {}) do
    local creator,as
    for _,ability in ipairs(abilities) do
      if in_row[ability.id] and not creator then
        each_op(ability,function(op)
          if op.op=='create_unit' and op.template==template.id and not creator then creator,as=ability,op.as end
        end)
      end
    end
    local list,seen={},{}
    for _,id in ipairs(template.abilities or {}) do if not seen[id] then seen[id]=true;list[#list+1]=id end end
    if creator and as then
      each_op(creator,function(op)
        if op.op=='add_ability' and op.to==as and op.ability and not seen[op.ability] then seen[op.ability]=true;list[#list+1]=op.ability end
      end)
    end
    if #list>0 then
      local targets={}
      for _,ability in ipairs(abilities) do targets[ability.id]=ability.target end
      out.summons[#out.summons+1]={template=template.id,after=creator and creator.id,abilities=list,targets=targets}
    end
  end
  return out
end

-- The aim of a summon's ability: its v3 bot kind when the spotlight can aim it, else its target
-- (summon abilities have bot kind none: the hero bot never casts them). nil when it cannot be shown.
local function summon_team(ability,target,aim)
  local bot=ability.v3 and ability.v3.bot
  if bot and bot.kind~='none' and aim[bot.kind] then return aim[bot.kind].team end
  local kind=(target and target.type) or ability.target_type or 'none'
  if kind=='none' then return 'self' end
  if kind=='point' or kind=='unit' then return target and target.team=='ally' and 'ally' or 'enemy' end
  return nil
end

-- E595: SLOTS (the hero row's demonstrations) with the dependents of DEPS: a sub-ability gets
-- `after` = its enabler, a swapped-in ability and each summon ability become slots of their own,
-- and every dependent is moved right after its enabler, so the renderer shows it while the window
-- the enabler opened is still open. A dependent whose enabler is not shown is listed in UNSHOWN.
function M.with_dependents(slots,deps,by_id,aim,unshown)
  local shown={}
  for _,slot in ipairs(slots) do shown[slot.id]=true end
  local after={}
  local function depend(enabler,slot)
    if not shown[enabler] then
      unshown[#unshown+1]={slot=slot.slot,id=slot.id,summon=slot.summon,reason='enabler '..tostring(enabler)..' not shown'}
      return
    end
    slot.after=enabler
    after[enabler]=after[enabler] or {}
    table.insert(after[enabler],slot)
  end
  local base={}
  for _,slot in ipairs(slots) do
    local enabler=deps.subs[slot.id]
    if enabler then depend(enabler,slot) else base[#base+1]=slot end
  end
  local function slot_of(id,index,extra)
    local ability=assert(by_id[id],'ability '..id..' is not in the content catalog')
    local kind=ability.ability_type
    if kind~='passive' and kind~='toggle' and kind~='ultimate' then kind='active' end
    local slot={slot=index,id=id,name=assert(ability.display_name,'ability without a name'),line=M.first_line(ability.description),kind=kind}
    for key,value in pairs(extra) do slot[key]=value end
    return slot,ability
  end
  for _,swap in ipairs(deps.swaps) do
    if by_id[swap.id] then
      local slot,ability=slot_of(swap.id,swap.slot,{})
      local bot=ability.v3 and ability.v3.bot
      slot.team=bot and aim[bot.kind] and aim[bot.kind].team or (slot.kind=='passive' and 'self') or nil
      if slot.team then depend(swap.after,slot)
      else unshown[#unshown+1]={slot=swap.slot,id=swap.id,reason='bot kind none'} end
    else unshown[#unshown+1]={slot=swap.slot,id=swap.id,reason='not in the content catalog'} end
  end
  for _,summon in ipairs(deps.summons) do
    for index,id in ipairs(summon.abilities) do
      local ability=by_id[id]
      local reason=not ability and 'not in the content catalog' or not summon.after and 'no row ability creates '..summon.template
        or index>4 and 'summon ability past SummonAbility4' or ability.ability_type=='passive' and 'summon passive' or nil
      local team=not reason and summon_team(ability,summon.targets[id],aim) or nil
      if not reason and not team then reason='summon target cannot be aimed' end
      if reason then unshown[#unshown+1]={slot=index-1,id=id,summon=summon.template,reason=reason}
      else
        local slot=slot_of(id,index-1,{summon=summon.template,team=team})
        -- A summon's ultimate-typed ability is still cast through the summon ability keys.
        if slot.kind=='ultimate' or slot.kind=='toggle' then slot.kind='active' end
        depend(summon.after,slot)
      end
    end
  end
  local out={}
  local function emit(slot)
    out[#out+1]=slot
    for _,dependent in ipairs(after[slot.id] or {}) do emit(dependent) end
    after[slot.id]=nil
  end
  for _,slot in ipairs(base) do emit(slot) end
  return out
end

function M.load_package(root,hero)
  local dir=path.join(root,'docs','character_pipeline','packages',hero)
  assert(path.is_directory(dir),'no hero package for '..hero..': '..dir)
  if not path.is_file(path.join(dir,'abilities.lua')) then return M.load_v3_package(root,hero) end
  local abilities=assert(loadfile(path.join(dir,'abilities.lua')))()({})
  local character=assert(loadfile(path.join(dir,'character.lua')))()({})
  local info=character.hero or character
  local slots={}
  for _,ability in ipairs(assert(abilities.abilities,'package has no abilities')) do
    local slot=M.SLOT_OF_ACTION[ability.action]
    assert(slot,('ability %s has unknown action %s'):format(tostring(ability.id),tostring(ability.action)))
    local kind=ability.ability_type
    if kind~='passive' and kind~='toggle' and kind~='ultimate' then kind='active' end
    local team=ability.targeting and ability.targeting.team or 'self'
    if team~='ally' and team~='enemy' and team~='both' then team='self' end
    slots[#slots+1]={slot=slot,id=ability.id,name=assert(ability.name,'ability without a name'),
      line=M.first_line(ability.description),kind=kind,team=team}
  end
  table.sort(slots,function(a,c) return a.slot<c.slot end)
  return {hero=hero,hero_name=info.display_name or hero,title=info.title or '',slots=slots}
end

-- The renderer's spotlight script (FOmSpotlightScript::Parse).
-- The radius an ability's effect covers, from its package values (exact key, its highest level).
-- Their radius is not something on the ground to show: the camera stays close.
M.NO_VIEW={c05_matsukaze=true,c05_flash_thunder=true}
-- Passives seen only in numbers and pace: attacks without them, then learned, then the same attacks (the renderer's LearnMidway).
-- Where only seat 7 of the death's strikers is within STRIKE_REACH (224; seat 9 at 335), clear of the
-- demonstration's stand-in: no other segment changes.
M.STRUCK_AT={2700,2130}
-- c05 雷鳴豪槍: its slowed attacks, at ranks 1 and 4. a01D 乘勢: mana and life back under a striker's hits (mana off: life only).
-- b20 不屈: a spell at ranks 1 and 4, in the circle of moba_hero_spotlight_casts.lua, 262 from its caster and no striker near.
M.STRUCK_BOTH={c05_thunder_spear=M.STRUCK_AT,a01_realm_momentum=M.STRUCK_AT,b20_tenacity={2480,2450}}
M.HOLDS_STILL={b20_tenacity=true}
-- A third half at the top rank. b26: the restore is a 17% chance, hence the long one. c05: beside the patrol of
-- moba_hero_spotlight_patrol.lua, watched unhurt first, then hit: the same road walked slowed.
M.THIRD_HALF={b26_kozuke_valor={at=M.STRUCK_AT,seconds=8},c05_thunder_spear={at={2550,2000},seconds=7,wait=4}}
-- Struck for this long the hero lives: seat 7 took 17 hits and 14 s to kill c05 at 164 a hit (out of its reach refill_players makes it whole).
M.HALF_SECONDS={c05_thunder_spear=5.5,b26_kozuke_valor=4}
-- Where a reach is the point (backend units from the stand-in): c22 燕閃 reaches 228 at rank 1 and 378 at rank 4.
M.STAND_OFF={c22_swallow_flash=330}
-- A melee hero stands where its weapon reaches the stand-in (backend units): the stand-in no longer comes to it,
-- and from the usual 188 the blow landed on nothing. Ranged heroes keep the usual place.
M.WEAPON_REACH={sasaki_kojiro=130,tokugawa_ieyasu=130,yamagata_masakage=130,okuma_tomohide=120,kojima_sadaoki=120,shimazu_yoshihiro=120,maeda_toshiie=170,maeda_keiji=170}
-- Passives that answer to the hero's own casts: the segment casts them, it does not attack.
M.CASTS_OWN={a12_scheme=true}
M.LEARN_MIDWAY={b20_tenacity=true,c22_swallow_flash=true,c05_thunder_spear=true,b26_kozuke_valor=true,a14_stacked_spear=true,a26_twin_guns=true,c03_thunder_wrath=true}
function M.view_radius(doc,id)
  if M.NO_VIEW[id] then return nil end
  for _,ability in ipairs(doc and doc.abilities or {}) do
    if ability.id==id and type(ability.values)=='table' then
      local value=ability.values.radius
      if type(value)=='number' then return value end
      if type(value)=='table' then
        local most=0
        for _,level in ipairs(value) do if type(level)=='number' then most=math.max(most,level) end end
        return most>0 and most or nil
      end
    end
  end
end

-- A skill's segment lasts as long as the skill, so what it leaves does not run into the next segment.
M.BASE_HOLD,M.MAX_HOLD=4.5,8
function M.hold_seconds(doc,id)
  for _,ability in ipairs(doc and doc.abilities or {}) do
    if ability.id==id and type(ability.values)=='table' then
      local value,most=ability.values.duration,0
      if type(value)=='number' then most=value end
      if type(value)=='table' then for _,level in ipairs(value) do if type(level)=='number' then most=math.max(most,level) end end end
      if most+0.5>M.BASE_HOLD then return math.min(most+0.5,M.MAX_HOLD) end
    end
  end
end

function M.script(package)
  local slots={}
  for moved,after in pairs(M.SHOWN_AFTER) do
    local from,to
    for index,row in ipairs(package.slots) do
      if row.id==moved then from=index elseif row.id==after then to=index end
    end
    if from and to then
      local row=table.remove(package.slots,from)
      table.insert(package.slots,from<to and to or to+1,row)
    end
  end
  local doc=package.hero and M.v3_document(b.root,package.hero)
  for _,slot in ipairs(package.slots) do
    slot.view=M.view_radius(doc,slot.id);slot.hold=M.hold_seconds(doc,slot.id)
    if M.LEARN_MIDWAY[slot.id] then slot.learn_midway=true end
    slot.stand_off=M.STAND_OFF[slot.id] or M.WEAPON_REACH[package.hero]
    local struck=M.STRUCK_BOTH[slot.id]
    if struck then slot.struck_x,slot.struck_y=struck[1],struck[2] end
    if M.HOLDS_STILL[slot.id] then slot.holds_still=true end
    local third=M.THIRD_HALF[slot.id]
    if third then slot.third_x,slot.third_y,slot.third_seconds,slot.third_wait=third.at[1],third.at[2],third.seconds,third.wait end
    if M.CASTS_OWN[slot.id] then slot.casts_own=true end
    slot.half_seconds=M.HALF_SECONDS[slot.id]
    slots[#slots+1]=slot
  end
  -- What needs an ability still active has its turn while it is.
  for index=1,#slots-1 do
    if slots[index+1].after==slots[index].id and slots[index].hold then slots[index].hold=math.min(slots[index].hold,M.BASE_HOLD) end
  end
  -- The death ends the recording: only the base stage has the enemies that strike it down.
  local stage=M.recipe_for(package)
  if stage==M.RECIPE or stage==M.LANE_RECIPE or stage==M.PATROL_RECIPE or stage==M.CASTS_RECIPE then
    for _,show in ipairs(M.BODY_SHOWS) do
      if show.kind=='attack' and M.WEAPON_REACH[package.hero] then
        local closer={}
        for key,value in pairs(show) do closer[key]=value end
        closer.stand_off=M.WEAPON_REACH[package.hero]
        show=closer
      end
      if show.kind=='death' and stage==M.LANE_RECIPE then
        local moved={}
        for key,value in pairs(show) do moved[key]=value end
        moved.at=M.LANE_DEATH_AT
        show=moved
      end
      slots[#slots+1]=show
    end
  end
  return {schema=1,hero=package.hero,hero_name=package.hero_name,title=package.title,
    timing=json.object({caption_lead=1.5,hold=4.5,passive=9}),slots=slots}
end

-- frames.jsonl from the renderer, sorted by frame index (workers finish out of order).
function M.read_frames(file)
  local frames={}
  for line in path.read(file):gmatch('[^\r\n]+') do
    local row=json.decode(line)
    frames[#frames+1]={frame=assert(math.tointeger(row.frame)),t=assert(tonumber(row.t))}
  end
  table.sort(frames,function(a,c) return a.frame<c.frame end)
  return frames
end

-- ffmpeg concat-demuxer list: every frame lasts until the next one was captured, so the video
-- keeps real time even when the renderer skipped or dropped a frame. The last frame is listed
-- twice (the demuxer ignores the final duration otherwise).
function M.concat_list(frames,dir,fps)
  assert(#frames>0,'no recorded frames')
  local lines={'ffconcat version 1.0'}
  local default=1/(fps or 30)
  for index,row in ipairs(frames) do
    local next_row=frames[index+1]
    local duration=next_row and math.max(next_row.t-row.t,0.001) or default
    lines[#lines+1]=("file '%s'"):format(path.join(dir,('frame_%06d.jpg'):format(row.frame)):gsub('\\','/'))
    lines[#lines+1]=('duration %.4f'):format(duration)
  end
  lines[#lines+1]=("file '%s'"):format(path.join(dir,('frame_%06d.jpg'):format(frames[#frames].frame)):gsub('\\','/'))
  return table.concat(lines,'\n')..'\n'
end

-- Keyframes: per demonstration, the caption (spotlight_slot) and the effect 1.2 s after the
-- cast (spotlight_cast), each mapped to the nearest recorded frame.
function M.keyframes(events,frames)
  local function nearest(t)
    local best
    for _,row in ipairs(frames) do
      if not best or math.abs(row.t-t)<math.abs(best.t-t) then best=row end
    end
    return best
  end
  local shots={}
  for _,event in ipairs(events) do
    if event.event=='spotlight_slot' or event.event=='spotlight_cast' then
      local t=tonumber(event.t) or 0
      if event.event=='spotlight_cast' then t=t+1.2 end
      local row=nearest(t)
      if row then
        -- A summon's ability (E595) is named by its summon: its slot is the summon ability index.
        shots[#shots+1]={name=('%02d_%sslot%s_%s_%s'):format(#shots+1,
          type(event.summon)=='string' and event.summon:match('^[%w_]+$') and event.summon..'_' or '',event.slot,event.aim or 'self',
          event.event=='spotlight_slot' and 'caption' or 'effect'),frame=row.frame}
      end
    end
  end
  return shots
end

function M.read_trace(file)
  local events={}
  if not path.is_file(file) then return events end
  for line in path.read(file):gmatch('[^\r\n]+') do
    local ok,row=pcall(json.decode,line)
    if ok and type(row)=='table' then events[#events+1]=row end
  end
  return events
end

function M.ffmpeg(options)
  local exe=path.absolute(options.ffmpeg,b.root)
  assert(path.is_file(exe),('ffmpeg not found at %s. Get a Windows build from %s, put ffmpeg.exe under '..
    'target/tools/ffmpeg/ (target/ is never committed) or pass --ffmpeg PATH. This tool never downloads it.'):format(exe,M.FFMPEG_SOURCE))
  return exe
end

-- What the authority's observer validation saw in one split run (E544): every client replica is
-- checked against the authority each checkpoint (omobab state::core "observer mismatch" lines).
-- {checked, mismatch = count of mismatch lines, first = the first divergent schema, or the
-- failure reason a role left (errors.md) when the run failed without a mismatch line}.
-- checked is true once the server logged an observer step for the match.
function M.observer(run_root)
  local result={checked=false,mismatch=0}
  for _,name in ipairs({'server.stdout.log','server.stderr.log'}) do
    local file=path.join(run_root,'server',name)
    if path.is_file(file) then
      for line in path.read(file):gmatch('[^\r\n]+') do
        if line:find('"metric":"observer_step"',1,true) then result.checked=true end
        if line:find('observer mismatch',1,true) then
          result.mismatch=result.mismatch+1
          result.first=result.first or line:match('schema=(0x%x+)') or 'observer mismatch'
        end
      end
    end
  end
  if not result.first then
    for _,role in ipairs({'server','p1'}) do
      local file=path.join(run_root,role,'errors.md')
      if path.is_file(file) then
        for line in path.read(file):gmatch('[^\r\n]+') do
          if not line:match('^#') and line:match('%S') then result.first=line;break end
        end
      end
      if result.first then break end
    end
  end
  return result
end

-- Writes the observer field of OUTPUT/<hero>/spotlight.json for RUN_ROOT, keeping the rest of
-- an existing report (a split run without an encoded video still gets one).
function M.write_observer(output,hero,run_root)
  local out=path.absolute(path.join(output,hero),b.root)
  path.mkdir_p(out)
  local file=path.join(out,'spotlight.json')
  local report=path.is_file(file) and json.decode(path.read(file)) or {schema_version=1,hero=hero}
  report.run=run_root
  report.observer=M.observer(run_root)
  -- The build the match launched (E583: a queue builds once; run_moba_split_local --prebuilt).
  local build=path.join(run_root,'build.json')
  report.build=path.is_file(build) and (function(r) return {commit=r.commit,as_of=r.as_of} end)(json.decode(path.read(build))) or nil
  json.write(file,report,true)
  return report
end

-- Encode one recorded run: mp4 (H.264, yuv420p, constant 30 fps), keyframe PNGs and a report.
-- What the script could not show: skipped segments, and the hero downed outside the death segment.
function M.issues(events)
  local names,issues={},{}
  for _,event in ipairs(events) do
    if event.event=='spotlight_slot' then names[event.slot]=(event.ability~='' and event.ability) or event.key end
  end
  for _,event in ipairs(events) do
    if event.event=='spotlight_skipped' then
      local why=event.level==0 and 'not learned or not castable at its turn (level 0)' or event.cooldown>0 and ('on cooldown %.1f s'):format(event.cooldown) or 'refused until its time ran out'
      issues[#issues+1]={kind='skipped',slot=event.slot,ability=names[event.slot] or '?',reason=why,tick=event.tick}
    elseif event.event=='spotlight_hero_down' then
      issues[#issues+1]={kind='hero_down',slot=event.slot,ability=names[event.slot] or '?',reason='hero downed outside the death segment',t=event.t}
    end
  end
  return issues
end

function M.encode(options,hero,run_root)
  local ffmpeg=M.ffmpeg(options)
  local seat=path.join(run_root,'p1')
  local frames_dir=path.join(seat,'spotlight-frames-p1')
  local events=M.read_trace(path.join(seat,'autopilot-p1.jsonl'))
  local finished
  for _,event in ipairs(events) do if event.event=='spotlight_finished' then finished=event end end
  local out=path.absolute(path.join(options.output,hero),b.root)
  if not finished or not path.is_file(path.join(frames_dir,'frames.jsonl')) then
    -- The renderer writes frames.jsonl as it finishes: cut off before that, there is nothing to encode.
    local issues=M.issues(events)
    issues[#issues+1]={kind='cut_off',reason='the renderer did not finish the spotlight before the time box of '..tostring(options.max_seconds)..' s retired it'}
    json.write(path.join(out,'spotlight.json'),{schema_version=1,hero=hero,run=run_root,reason='incomplete',script_reason='cut_off',issues=issues,keyframes={},observer=M.observer(run_root)},true)
    error('spotlight '..hero..': incomplete, no video; see '..path.join(out,'spotlight.json'),0)
  end
  local frames=M.read_frames(path.join(frames_dir,'frames.jsonl'))
  -- Before the first caption the hero is only hitting the stand-in: the video starts at it. video_start says how far trace times lie ahead of video times.
  local video_start=0
  for _,event in ipairs(events) do
    if event.event=='spotlight_slot' then video_start=tonumber(event.t) or 0;break end
  end
  local shown={}
  for _,row in ipairs(frames) do if row.t>=video_start then shown[#shown+1]=row end end
  if #shown>0 then frames=shown else video_start=0 end
  path.mkdir_p(path.join(out,'keyframes'))
  -- Keyframes describe this run only: never leave an earlier run's frames beside them.
  for name in lfs.dir(path.join(out,'keyframes')) do
    if name:match('%.png$') then assert(os.remove(path.join(out,'keyframes',name))) end
  end
  local list=path.join(out,'frames.ffconcat')
  path.write(list,M.concat_list(frames,frames_dir,options.fps),true)
  local video=path.join(out,hero..'_spotlight.mp4')
  process.run(ffmpeg,{'-hide_banner','-loglevel','error','-y','-f','concat','-safe','0','-i',list,
    '-vf','fps='..options.fps..',scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2,format=yuv420p',
    '-c:v','libx264','-preset','medium','-crf','18','-movflags','+faststart',video},{label='ffmpeg encode'})
  local shots={}
  for _,shot in ipairs(M.keyframes(events,frames)) do
    local png=path.join(out,'keyframes',shot.name..'.png')
    process.run(ffmpeg,{'-hide_banner','-loglevel','error','-y','-i',path.join(frames_dir,('frame_%06d.jpg'):format(shot.frame)),png},{label='ffmpeg keyframe'})
    shots[#shots+1]=png
  end
  local issues=M.issues(events)
  local report={schema_version=1,hero=hero,video=video,video_start=video_start,keyframes=shots,run=run_root,frames=#frames,
    duration_seconds=frames[#frames].t-frames[1].t,reason=#issues>0 and 'incomplete' or finished.reason,
    script_reason=finished.reason,issues=issues,casts=finished.casts,skipped=finished.skipped,
    dropped=finished.dropped,observer=M.observer(run_root)}
  json.write(path.join(out,'spotlight.json'),report,true)
  return report
end

-- The authority port of the INDEX-th hero of one invocation (1-based). Consecutive heroes
-- never reuse a port, so a previous run still releasing its socket cannot collide with the
-- next one; presentation ports (port + player id) stay inside each 10-port block.
function M.run_port(options,index)
  local port=options.port+(index-1)*10
  assert(port<=65525 and (port>57070 or port<57061),'spotlight port block overlaps the shared 57061')
  return port
end

function M.record(options,hero,index)
  local split=require('run_moba_split_local')
  local package=M.load_package(b.root,hero)
  local out=path.absolute(path.join(options.output,hero),b.root)
  path.mkdir_p(out)
  -- An earlier run's report and video must not pass for this run's, whatever becomes of it.
  os.remove(path.join(out,'spotlight.json'))
  local video=path.join(out,hero..'_spotlight.mp4')
  if path.is_file(video) then
    os.remove(video..'.previous')
    assert(os.rename(video,video..'.previous'))
  end
  local script=path.join(out,'spotlight-script.json')
  json.write(script,M.script(package),true)
  local run_root=path.join(out,'run-'..os.date('%Y%m%d-%H%M%S'))
  local args={'--recipe',M.recipe_for(package),'--hero','1='..hero,'--autopilot','spotlight','--spotlight-script',script,
    '--resolution','1920x1080','--max-seconds',tostring(options.max_seconds),'--port',tostring(M.run_port(options,index or 1)),'--output',run_root,'--ue-root',options.ue_root}
  if options.no_build then args[#args+1]='--no-build' end
  -- A batch of heroes queues with whatever else holds the machine, not fails.
  args[#args+1]='--wait-live-lock';args[#args+1]='1800'
  split.run(split.options(args))
  return M.encode(options,hero,run_root)
end

-- Arguments of one hero's own invocation (several heroes run one after another, each in its
-- own supervised workflow, so its server, runtime and renderer are retired before the next).
function M.child_args(options,hero,index)
  local args={'--hero',hero,'--output',options.output,'--ffmpeg',options.ffmpeg,'--ue-root',options.ue_root,
    '--max-seconds',tostring(options.max_seconds),'--port',tostring(M.run_port(options,index))}
  -- Only the first run builds; later heroes reuse the same artifacts.
  if options.no_build or index>1 then args[#args+1]='--no-build' end
  return args
end

-- Every process a finished run owned (renderer, runtime, authority) must be gone. A retired
-- Unreal renderer takes seconds to exit, so wait up to TIMEOUT_MS before calling it a leak.
function M.assert_retired(run_root,timeout_ms,sleep_ms)
  local records={}
  for _,file in ipairs({path.join(run_root,'p1','owned-processes.json'),path.join(run_root,'server','owned-processes.json')}) do
    if path.is_file(file) then
      for _,record in ipairs(json.decode(path.read(file))) do records[#records+1]=record end
    end
  end
  local time=b.lib('time')
  local deadline=time.monotonic_ms()+(timeout_ms or 60000)
  while true do
    local alive={}
    -- Identity includes the creation token: a reused PID is not this run's process.
    for _,record in ipairs(records) do
      -- A PID whose identity cannot even be queried (access denied) was reused by another
      -- user's or a protected process: this run's processes are always ours to inspect.
      local queried,live=pcall(process.owned_alive,record)
      if queried and live then alive[#alive+1]=tostring(record.label)..' '..tostring(record.pid) end
    end
    if #alive==0 then return end
    assert(time.monotonic_ms()<deadline,'spotlight run left processes alive: '..table.concat(alive,', '))
    time.sleep_ms(sleep_ms or 1000)
  end
end

-- A renderer that never reached the demonstration (no trace or no frames, e.g. it hung while
-- starting and the time box retired it) left nothing to encode: that hero is recorded again.
function M.renderer_never_recorded(run_root)
  local seat=path.join(run_root,'p1')
  if not path.is_file(path.join(seat,'spotlight-frames-p1','frames.jsonl')) then return true end
  for _,event in ipairs(M.read_trace(path.join(seat,'autopilot-p1.jsonl'))) do
    if event.event=='spotlight_finished' then return false end
  end
  return true
end

if arg and arg[0] and arg[0]:match('record_hero_spotlight%.lua$') then
  if arg[1]=='--help' then
    print('Usage: tools/lua/lua.exe scripts/record_hero_spotlight.lua (--hero HERO [--hero HERO...] | --all) [--output target/spotlight] [--ffmpeg PATH] [--no-build] [--max-seconds 300]')
    print('       tools/lua/lua.exe scripts/record_hero_spotlight.lua --hero HERO --encode-only --run RUN_DIRECTORY')
    print('       tools/lua/lua.exe scripts/record_hero_spotlight.lua --hero HERO --observer-only --run RUN_DIRECTORY [--output target/spotlight]')
    print('Heroes: '..table.concat(M.HEROES,', '))
    return
  end
  local options=M.options(arg)
  if options.observer_only then
    local report=M.write_observer(options.output,options.heroes[1],path.absolute(assert(options.run,'--observer-only requires --run'),b.root))
    print(('spotlight %s observer: checked=%s mismatch=%d first=%s'):format(report.hero,tostring(report.observer.checked),report.observer.mismatch,tostring(report.observer.first)))
    return
  end
  M.ffmpeg(options) -- fail before a ten-minute run when the encoder is missing
  if options.encode_only then
    local report=M.encode(options,options.heroes[1],path.absolute(options.run,b.root))
    print('spotlight '..report.hero..': '..report.video)
    return
  end
  if #options.heroes>1 then
    -- One supervised invocation per hero: returning from it retires everything that hero started.
    local failures={}
    local function latest_run(hero)
      local out=path.absolute(path.join(options.output,hero),b.root)
      local runs={}
      if path.is_directory(out) then
        for name in lfs.dir(out) do if name:match('^run%-') then runs[#runs+1]=name end end
      end
      table.sort(runs)
      return #runs>0 and path.join(out,runs[#runs]) or nil
    end
    for index,hero in ipairs(options.heroes) do
      local result
      for attempt=1,2 do
        result=process.run(platform.lua_executable,{source,table.unpack(M.child_args(options,hero,index))},
          {check=false,label='spotlight '..hero})
        io.write(result.stdout or '')
        local run=latest_run(hero)
        if run then
          local ok,err=pcall(M.assert_retired,run)
          if not ok then failures[#failures+1]=hero..': '..tostring(err) end
        end
        if result.exit_code==0 or not run or not M.renderer_never_recorded(run) or attempt==2 then break end
        print('spotlight '..hero..': the renderer never reached the demonstration; recording it again')
      end
      if result.exit_code~=0 then
        failures[#failures+1]=hero..': exit '..tostring(result.exit_code)..' '..tostring(result.stderr):sub(1,400)
        print('spotlight '..hero..' FAILED (exit '..tostring(result.exit_code)..')')
      end
    end
    assert(#failures==0,table.concat(failures,'\n'))
    return
  end
  process.supervise_workflow(source,arg)
  local hero=options.heroes[1]
  local result=M.record(options,hero,1)
  print('spotlight '..hero..': '..result.video..' ('..#result.keyframes..' keyframes, skipped '..tostring(result.skipped)..')')
end
return M
