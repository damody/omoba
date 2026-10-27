package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local json=b.lib('json')
local spotlight=require('record_hero_spotlight')

-- Options: one or more heroes, or all eight; live runs stay under ten minutes.
local options=spotlight.options({'--hero','saika_magoichi'})
assert(options.heroes[1]=='saika_magoichi' and #options.heroes==1)
assert(options.output=='target/spotlight' and options.ffmpeg==spotlight.DEFAULT_FFMPEG and options.fps==30)
-- Its own authority port, never the shared split default, so it cannot join another live match.
assert(options.port==spotlight.DEFAULT_PORT and options.port~=57061)
assert(spotlight.options({'--hero','oichi','--port','58000'}).port==58000)
assert(not pcall(spotlight.options,{'--hero','oichi','--port','57061'}))
-- Each hero of one invocation gets its own port block (no reuse while a socket is released).
local seen={}
local every=spotlight.options({"--all"})
for index=1,#every.heroes do
  local port=spotlight.run_port(every,index)
  assert(not seen[port] and (port<57061 or port>57070),port) seen[port]=true
end
assert(spotlight.run_port(every,1)==spotlight.DEFAULT_PORT and spotlight.run_port(every,2)==spotlight.DEFAULT_PORT+10)
local all=spotlight.options({'--all'})
assert(#all.heroes==8)
for _,bad in ipairs({{},{'--all','--hero','oichi'},{'--hero','bad name'},{'--max-seconds','900','--hero','oichi'},
  {'--encode-only','--hero','oichi'},{'--unknown'}}) do
  assert(not pcall(spotlight.options,bad),'rejected: '..table.concat(bad,' '))
end

-- One caption line: whole clauses of the first sentence until it says enough, rank lists as
-- ranges, type prefixes and parentheticals dropped, capped by characters (not bytes).
assert(spotlight.first_line('轉換為狙擊狀態 12 秒：攻擊力 +0/10/20/30，攻擊距離 +200/350/500/700。第二句。')
  =='轉換為狙擊狀態 12 秒，攻擊力 +0～30',spotlight.first_line('轉換為狙擊狀態 12 秒：攻擊力 +0/10/20/30，攻擊距離 +200/350/500/700。'))
assert(spotlight.first_line('被動：魔法抗性 +9/18/27/36%，總攻擊傷害 +6/9/12/15%。')=='魔法抗性 +9～36%，總攻擊傷害 +6～15%')
assert(spotlight.first_line('朝前方灑出冰霜（直線投射物，速度 1250），命中的每個敵人受 100/325 魔法傷害。')
  =='朝前方灑出冰霜，命中的每個敵人受 100～325 魔法傷害')
assert(spotlight.first_line('受到的物理傷害 −20（固定格擋）。')=='受到的物理傷害 −20')
local long=spotlight.first_line(string.rep('長',80),10)
assert(utf8.len(long)==10 and long:sub(-#'…')=='…')
assert(spotlight.first_line(nil)=='')
for _,hero in ipairs(spotlight.HEROES) do
  for _,slot in ipairs(spotlight.load_package(b.root,hero).slots) do
    assert(utf8.len(slot.line) and utf8.len(slot.line)>=4 and utf8.len(slot.line)<=44,hero..' '..slot.name..': '..slot.line)
  end
end

-- Every shipped hero package yields a script the renderer accepts (unique slots 0..4,
-- names, known kinds and teams), in slot order.
for _,hero in ipairs(spotlight.HEROES) do
  local package=spotlight.load_package(b.root,hero)
  assert(package.hero==hero and package.hero_name~='' and #package.slots>=4,hero)
  local seen={}
  for index,slot in ipairs(package.slots) do
    assert(slot.slot>=0 and slot.slot<=4 and not seen[slot.slot],hero..' slot '..slot.slot)
    seen[slot.slot]=true
    if index>1 then assert(slot.slot>package.slots[index-1].slot,hero..' slot order') end
    assert(slot.name~='' and ({active=1,passive=1,toggle=1,ultimate=1})[slot.kind] and ({self=1,ally=1,enemy=1,both=1})[slot.team],hero)
  end
  local script=json.decode(json.encode(spotlight.script(package)))
  assert(script.schema==1 and script.hero==hero and #script.slots==#package.slots)
end
local saika=spotlight.load_package(b.root,'saika_magoichi')
assert(saika.hero_name=='雜賀孫市' and saika.slots[1].name=='狙擊模式' and saika.slots[2].team=='ally' and saika.slots[3].kind=='passive')
assert(not pcall(spotlight.load_package,b.root,'missing_hero'))

-- The stage recipe: one human, two held bot stand-ins (enemy and ally) placed in the open centre.
local recipe=assert(loadfile(spotlight.RECIPE))()
local humans,held=0,{}
for _,player in ipairs(recipe.players) do
  if player.bot==false then humans=humans+1 assert(player.player_id==1) end
end
for _,id in ipairs(recipe.showcase.hold_players) do held[id]=true end
assert(humans==1 and held[2] and held[6] and not held[1])
assert(recipe.showcase.minion_waves==false and recipe.showcase.start_level>=6)
for _,id in ipairs({'1','2','6'}) do assert(recipe.showcase.placements[id],'placement '..id) end

-- Encoder input: real-time durations from the recorded timestamps; the last frame repeated.
local frames={{frame=0,t=0.0},{frame=1,t=0.0333},{frame=3,t=0.1}}
local list=spotlight.concat_list(frames,'D:\\frames',30)
assert(list:find('ffconcat version 1.0',1,true)==1)
assert(list:find("file 'D:/frames/frame_000001.jpg'\nduration 0.0667",1,true),list)
local _,files=list:gsub('\nfile ','')
assert(files==4,'last frame listed twice: '..files)
assert(not pcall(spotlight.concat_list,{},'x',30))

-- Keyframes: caption and effect (1.2 s after the cast) of every demonstration, nearest frames.
local many={}
for index=0,300 do many[#many+1]={frame=index,t=index/30} end
local shots=spotlight.keyframes({{event='spotlight_slot',slot=0,aim='self',t=2.0},{event='spotlight_cast',slot=0,aim='self',t=3.5},
  {event='spotlight_skipped',slot=1},{event='spotlight_slot',slot=3,aim='enemy',t=8.0}},many)
assert(#shots==3)
assert(shots[1].frame==60 and shots[1].name=='01_slot0_self_caption')
assert(shots[2].frame==141 and shots[2].name=='02_slot0_self_effect')
assert(shots[3].frame==240)

-- A missing encoder fails early and says where to get one; it is never downloaded.
local ok,err=pcall(spotlight.ffmpeg,{ffmpeg='target/no-such-dir/ffmpeg.exe'})
assert(not ok and tostring(err):find('BtbN',1,true) and tostring(err):find('never downloads',1,true))
-- --all runs each hero in its own invocation (own workflow, own port, builds only first).
local every2=spotlight.options({'--all','--max-seconds','240'})
local first=table.concat(spotlight.child_args(every2,'saika_magoichi',1),' ')
local second=table.concat(spotlight.child_args(every2,'date_masamune',2),' ')
assert(first:find('--hero saika_magoichi',1,true) and first:find('--port '..spotlight.DEFAULT_PORT,1,true) and not first:find('--no-build',1,true))
assert(second:find('--port '..(spotlight.DEFAULT_PORT+10),1,true) and second:find('--no-build',1,true) and second:find('--max-seconds 240',1,true))
-- A finished run with no owned process left passes; a live one is reported.
do
  local path=b.lib('path')
  local root=path.mkdir_p(path.join(b.root,'target','tests','spotlight-retired'))
  path.mkdir_p(path.join(root,'p1'))
  path.write(path.join(root,'p1','owned-processes.json'),'[{"label":"unreal-p1","pid":4294967,"creation_token":"1","executable":"D:/x.exe"}]',true)
  spotlight.assert_retired(root)
end
-- A run whose renderer never reached the demonstration is recorded again; a finished one is not.
do
  local path=b.lib('path')
  local root=path.mkdir_p(path.join(b.root,'target','tests','spotlight-never-recorded'))
  path.mkdir_p(path.join(root,'p1'))
  os.remove(path.join(root,'p1','spotlight-frames-p1','frames.jsonl'))
  assert(spotlight.renderer_never_recorded(root),'no frames')
  path.mkdir_p(path.join(root,'p1','spotlight-frames-p1'))
  path.write(path.join(root,'p1','spotlight-frames-p1','frames.jsonl'),'{"frame":0,"t":0}\n',true)
  path.write(path.join(root,'p1','autopilot-p1.jsonl'),'{"event":"start"}\n',true)
  assert(spotlight.renderer_never_recorded(root),'frames but never finished')
  path.write(path.join(root,'p1','autopilot-p1.jsonl'),'{"event":"spotlight_finished","reason":"complete"}\n',true)
  assert(not spotlight.renderer_never_recorded(root))
end
print('record_hero_spotlight_test ok')
