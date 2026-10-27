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

M.HEROES={'saika_magoichi','date_masamune','omatsu','oichi','inahime','asai_nagamasa','murakami_yoshikiyo','tamako'}
M.SLOT_OF_ACTION={Q=0,W=1,E=2,R=3,extra1=4}
M.RECIPE='scripts/lua_data/moba_hero_spotlight.lua'
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

function M.load_package(root,hero)
  local dir=path.join(root,'docs','character_pipeline','packages',hero)
  assert(path.is_directory(dir),'no hero package for '..hero..': '..dir)
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
function M.script(package)
  return {schema=1,hero=package.hero,hero_name=package.hero_name,title=package.title,
    timing=json.object({caption_lead=1.5,hold=4.5,passive=5}),slots=package.slots}
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
        shots[#shots+1]={name=('%02d_slot%d_%s_%s'):format(#shots+1,event.slot,event.aim or 'self',
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

-- Encode one recorded run: mp4 (H.264, yuv420p, constant 30 fps), keyframe PNGs and a report.
function M.encode(options,hero,run_root)
  local ffmpeg=M.ffmpeg(options)
  local seat=path.join(run_root,'p1')
  local frames_dir=path.join(seat,'spotlight-frames-p1')
  local frames=M.read_frames(path.join(frames_dir,'frames.jsonl'))
  local events=M.read_trace(path.join(seat,'autopilot-p1.jsonl'))
  local finished
  for _,event in ipairs(events) do if event.event=='spotlight_finished' then finished=event end end
  assert(finished,'the renderer never finished the spotlight; see '..path.join(seat,'autopilot-p1.jsonl'))
  local out=path.absolute(path.join(options.output,hero),b.root)
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
  local report={schema_version=1,hero=hero,video=video,keyframes=shots,run=run_root,frames=#frames,
    duration_seconds=frames[#frames].t-frames[1].t,reason=finished.reason,casts=finished.casts,skipped=finished.skipped,
    dropped=finished.dropped}
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
  local script=path.join(out,'spotlight-script.json')
  json.write(script,M.script(package),true)
  local run_root=path.join(out,'run-'..os.date('%Y%m%d-%H%M%S'))
  local args={'--recipe',M.RECIPE,'--hero','1='..hero,'--autopilot','spotlight','--spotlight-script',script,
    '--resolution','1920x1080','--max-seconds',tostring(options.max_seconds),'--port',tostring(M.run_port(options,index or 1)),'--output',run_root,'--ue-root',options.ue_root}
  if options.no_build then args[#args+1]='--no-build' end
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
    print('Heroes: '..table.concat(M.HEROES,', '))
    return
  end
  local options=M.options(arg)
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
