-- One-shot shader/DDC warmup before any multi-renderer Unreal launch.
-- A cold DDC makes every renderer compile the same global shaders at frame 0,
-- and the host budget's -corelimit leaves only a few ShaderCompileWorkers, so two
-- same-machine clients sat at frame 0 for over an hour (errors.md E003). The
-- warmup runs exactly one renderer without -corelimit and without a gameplay
-- runtime until it renders frames after the map load, then retires it. A stamp
-- keyed by engine binary identity, RHI and project records a completed warmup.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local lfs=require('lfs')
local M={}

M.SETTLED_FRAMES=120
M.DEFAULT_TIMEOUT_MS=3*3600*1000

function M.stamp_path(root,editor,project,graphics)
  local attributes=assert(lfs.attributes(editor),'missing Unreal editor: '..tostring(editor))
  local key=('%s-%d-%d-%s'):format(graphics,attributes.size,attributes.modification,
    (project:gsub('[^%w]','_')):sub(-48))
  return path.join(root,'target','ue-shader-warmup',key..'.json')
end

-- Frame counter from the bracketed second field of a UE log line, e.g. "[..][ 42]".
function M.max_frame_after_load(text)
  local loaded=text:find('LogLoad: Took',1,true)
  if not loaded then return nil end
  local best=0
  for frame in text:sub(loaded):gmatch('\n%[[^%]]*%]%[%s*(%d+)%]') do
    best=math.max(best,math.tointeger(tonumber(frame)) or 0)
  end
  return best
end

-- Warmup arguments: same project, map and RHI as the match renderers, but no
-- gameplay identity, no -corelimit and a presentation endpoint nobody serves.
function M.warmup_args(sample_args,output)
  local args={}
  local skip={['-corelimit']=true,['-om-autopilot']=true,['-om-autopilot-trace']=true,
    ['-om-result-ui-smoke']=true,['-om-result-ui-screenshot']=true,['-sessionname']=true,
    ['-UserDir']=true,['-abslog']=true,['-om-presentation']=true}
  for _,arg in ipairs(sample_args) do
    local name=arg:match('^(%-[%w%-]+)')
    if not (name and skip[name]) then args[#args+1]=arg end
  end
  args[#args+1]='-om-presentation=127.0.0.1:1'
  args[#args+1]='-sessionname=omfue-shader-warmup'
  args[#args+1]='-UserDir='..path.join(output,'ue-shader-warmup')
  args[#args+1]='-abslog='..path.join(output,'ue-shader-warmup.log')
  return args
end

function M.ensure(options,process,time)
  local stamp=M.stamp_path(b.root,options.editor,options.project,options.graphics)
  if path.is_file(stamp) then return {status='warm',stamp=stamp} end
  local log=path.join(options.output,'ue-shader-warmup.log')
  local env={}
  for key,value in pairs(options.env or {}) do env[key]=value end
  env.OM_PRESENTATION_ADDRESS='127.0.0.1:1'
  env.OMFX_PRESENTATION_ADDR='127.0.0.1:1'
  print('Warming Unreal shaders with one renderer (cold DDC; one-time per engine/RHI/project).')
  local started=time.monotonic_ms()
  local _,record=process.spawn_owned(options.editor,M.warmup_args(options.sample_args,options.output),
    {cwd=options.cwd,env=env,stdout=path.join(options.output,'ue-shader-warmup.stdout.log'),
     stderr=path.join(options.output,'ue-shader-warmup.stderr.log')})
  record=process.validate_owned(record)
  if options.journal then options.journal(record,'ue-shader-warmup') end
  local deadline=started+(options.timeout_ms or M.DEFAULT_TIMEOUT_MS)
  local ok,err=pcall(function()
    while true do
      assert(process.owned_alive(record),'shader warmup renderer exited before rendering; see '..log)
      local frames=path.is_file(log) and M.max_frame_after_load(path.read(log)) or nil
      if frames and frames>=M.SETTLED_FRAMES then break end
      assert(time.monotonic_ms()<deadline,'shader warmup timed out; see '..log)
      time.sleep_ms(2000)
    end
  end)
  if process.owned_alive(record) then process.stop_owned(record) end
  assert(process.wait_owned(record,15000),'shader warmup renderer remained after stop')
  assert(ok,err)
  local elapsed=time.monotonic_ms()-started
  path.mkdir_p(path.parent(stamp))
  path.write(stamp,json.encode({schema_version=1,editor=options.editor,project=options.project,
    graphics=options.graphics,elapsed_ms=elapsed,log=log}),true)
  return {status='warmed',stamp=stamp,elapsed_ms=elapsed}
end

return M
