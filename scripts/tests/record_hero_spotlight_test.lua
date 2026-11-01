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
-- A v3-only package (no abilities.lua) takes its slots from the content catalog, in slot order.
local keiji=spotlight.load_package(b.root,'maeda_keiji')
assert(keiji.hero_name=='前田慶次' and #keiji.slots==4,'v3 package slots')
for index,slot in ipairs(keiji.slots) do assert(slot.slot==index-1 and slot.name~='' and slot.line~='') end
assert(keiji.slots[1].id=='c05_matsukaze' and keiji.slots[1].team=='self' and keiji.slots[1].kind=='active')
assert(keiji.slots[3].kind=='passive' and keiji.slots[3].team=='self')
assert(keiji.slots[4].kind=='ultimate' and keiji.slots[4].team=='enemy')

-- An empty slot (an F without a D, E379) is skipped and the F keeps slot 5.
local kazumasu=spotlight.load_package(b.root,'takigawa_kazumasu')
assert(kazumasu.slots[#kazumasu.slots].slot==5 and kazumasu.slots[#kazumasu.slots].id=='a32_soul_blade','F keeps the sixth slot')
for _,slot in ipairs(kazumasu.slots) do assert(slot.slot~=4,'the empty D slot shows nothing') end

-- Tree-target abilities (bot kind nearest_tree: B22 W and D) aim at a tree and are shown on the
-- tree stage, the nobu map (D21: every ability is demonstrated); a hero without one keeps the
-- live-check stage.
local sessai=spotlight.load_package(b.root,'taigen_sessai')
assert(#sessai.unshown==0 and #sessai.slots==5,'every B22 slot is shown')
assert(sessai.slots[2].id=='b22_force_of_nature' and sessai.slots[2].team=='tree' and sessai.slots[5].team=='tree','W and D aim at a tree')
assert(spotlight.recipe_for(sessai)==spotlight.TREE_RECIPE,'B22 is shown on the tree stage')
assert(spotlight.recipe_for(spotlight.load_package(b.root,'saika_magoichi'))==spotlight.RECIPE,'other heroes keep the base stage')

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
-- E544: the observer field records what the authority's observer validation saw in the run.
do
  local lfs,path=require('lfs'),b.lib('path')
  local run=path.join(b.root,'target','spotlight-observer-test-'..os.time())
  path.mkdir_p(path.join(run,'server'));path.mkdir_p(path.join(run,'p1'))
  local function write(file,text) path.write(path.join(run,file),text,true) end
  write('server/server.stdout.log','x OM_PERF {"v":1,"component":"server","metric":"observer_step"}\n')
  local clean=spotlight.observer(run)
  assert(clean.checked and clean.mismatch==0 and clean.first==nil,'a validated run without a mismatch')
  write('server/server.stdout.log','x OM_PERF {"metric":"observer_step"}\n[ERROR] - observer mismatch team=2 replica_tick=2640: [entity=1 schema=0x464f471b expected=a observed=absent]\n'..
    '[ERROR] - observer mismatch team=1 replica_tick=2880: [entity=171 schema=0x464f471b expected=absent observed=a]\n')
  local bad=spotlight.observer(run)
  assert(bad.checked and bad.mismatch==2 and bad.first=='0x464f471b','mismatches are counted, the first schema named')
  write('server/server.stdout.log','')
  write('p1/errors.md','# Role launch error\n\nruntime-p1 exited during match\n')
  local failed=spotlight.observer(run)
  assert(not failed.checked and failed.mismatch==0 and failed.first=='runtime-p1 exited during match','a failed run names its reason')
  local out=path.join(b.root,'target','spotlight-observer-test-out-'..os.time())
  write('build.json','{"schema_version":1,"as_of":1700000000,"commit":"abc123","heroes":[]}')
  local report=spotlight.write_observer(out,'saika_magoichi',run)
  assert(report.observer.first=='runtime-p1 exited during match' and path.is_file(path.join(out,'saika_magoichi','spotlight.json')))
  assert(report.build.commit=='abc123' and report.build.as_of==1700000000,'the report names the build it launched (E583)')
  for _,p in ipairs({'server/server.stdout.log','p1/errors.md','build.json'}) do os.remove(path.join(run,p)) end
  lfs.rmdir(path.join(run,'server'));lfs.rmdir(path.join(run,'p1'));lfs.rmdir(run)
  os.remove(path.join(out,'saika_magoichi','spotlight.json'));lfs.rmdir(path.join(out,'saika_magoichi'));lfs.rmdir(out)
end
print('record_hero_spotlight_test ok')
