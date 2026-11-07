-- Dev hot reload of one hero DLL (M3' ②, docs/plans/2026-10-28-history-nobu-full-port/
-- m3-rust-codegen-design.md §4):
--
--   tools\lua\lua.exe scripts\hero_hot_reload.lua --hero <hero id> --run <run dir>
--       [--no-generate] [--no-build] [--timeout SECONDS]
--
--   1. regenerates the hero's Rust (v3_rust.lua --hero, when that generator exists and
--      --no-generate is not given)
--   2. cargo build -p <package> --profile dev-hot (only that DLL and what it depends on)
--   3. copies the DLL to <run>/artifacts/scripts-hot/<package>.<generation>.dll: a new name, so
--      the build output is never mapped and the next build can relink it (as E125)
--   4. writes <run>/artifacts/scripts-hot/request-<generation>.json; the authority (started with
--      OMB_SCRIPT_HOT_DIR at that directory, a dev-only switch) loads it between two ticks and
--      announces the swap in the team frames, so every replica swaps at the same tick
--   5. waits for request-<generation>.result.json and prints the tick it took effect
--
-- Only values and code may change: the catalog identity (hero and ability ids) must stay the
-- host's, and the hero must register the same abilities. Matches started without
-- OMB_SCRIPT_HOT_DIR (every formal match) never hot reload.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,json,time=b.lib('path'),b.lib('process'),b.lib('json'),b.lib('time')
local sha256=b.lib('hash').sha256
local lfs=require('lfs')
local M={}

M.PROFILE='dev-hot'
M.HOT_DIR='scripts-hot'

function M.options(args)
  local options={generate=true,build=true,timeout=60}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--no-generate' then options.generate=false
    elseif key=='--no-build' then options.build=false
    elseif key=='--hero' or key=='--run' or key=='--timeout' or key=='--target-dir' then
      i=i+1;options[key:sub(3):gsub('%-','_')]=assert(args[i],key..' requires a value')
    elseif key=='--help' or key=='-h' then options.help=true
    else error('unknown argument: '..key,0) end
    i=i+1
  end
  if options.help then return options end
  assert(options.hero,'--hero is required')
  assert(options.run,'--run is required (the run directory of the match to reload)')
  options.timeout=assert(tonumber(options.timeout),'--timeout must be a number')
  return options
end

-- The hero_dlls.lua row of HERO.
function M.row(hero)
  for _,module in ipairs(require('script_dlls').modules()) do
    if module.hero==hero then return module end
  end
  error('hero '..hero..' has no DLL of its own (scripts/hero_dlls.lua)',0)
end

-- The next free generation of PACKAGE in HOT_DIR (1, 2, ...).
function M.next_generation(hot_dir,package_name)
  local generation=0
  if path.is_directory(hot_dir) then
    for name in lfs.dir(hot_dir) do
      local n=name:match('^'..package_name:gsub('%p','%%%0')..'%.(%d+)%.dll$')
      if n then generation=math.max(generation,tonumber(n)) end
    end
  end
  return generation+1
end

local function target_dir(options)
  if options.target_dir then return path.absolute(options.target_dir,b.root) end
  return require('run_artifacts').rust_target(b.root,'scripts')
end

function M.run(options,deps)
  deps=deps or {}
  local run_process=deps.run or function(exe,args,opts) return process.run(exe,args,opts) end
  local log=deps.log or print
  local module=M.row(options.hero)
  local run_dir=path.absolute(options.run,b.root)
  assert(path.is_file(path.join(run_dir,'artifacts','scripts','scripts-manifest.json')),
    'not a run directory with staged script DLLs: '..run_dir)
  if options.generate then
    local generator=path.join(b.root,'scripts','v3_rust.lua')
    if path.is_file(generator) then
      local result=run_process(b.lib('platform').lua_executable,{generator,'--hero',options.hero},{cwd=b.root,check=false})
      assert(result.exit_code==0,'v3_rust.lua failed for '..options.hero..'\n'..(result.stderr or ''))
    else
      log('[hero-hot-reload] v3_rust.lua not present yet; building the hero crate as it is')
    end
  end
  local target=target_dir(options)
  if options.build then
    local started=time.monotonic_ms()
    local args={'build','--manifest-path',path.join(b.root,'scripts','Cargo.toml'),'-p',module.module,
      '--profile',M.PROFILE,'--features',module.module..'/compiled-content-only'}
    local result=run_process('cargo',args,{cwd=b.root,check=false,env={CARGO_TARGET_DIR=target}})
    assert(result.exit_code==0,'cargo build of '..module.module..' failed\n'..(result.stderr or ''))
    local finished=time.monotonic_ms()
    log(('[hero-hot-reload] built %s (%s) in %.1fs'):format(module.module,M.PROFILE,(finished-started)/1000))
  end
  local built=path.join(target,M.PROFILE,module.file)
  assert(path.is_file(built),'missing build output: '..built)
  local hot_dir=path.join(run_dir,'artifacts',M.HOT_DIR)
  path.mkdir_p(hot_dir)
  local generation=M.next_generation(hot_dir,module.module)
  local copy=path.join(hot_dir,('%s.%d.dll'):format(module.module,generation))
  path.write(copy,path.read(built,true),false,true)
  local digest=sha256(copy):lower()
  local request=path.join(hot_dir,('request-%d.json'):format(generation))
  local result_file=path.join(hot_dir,('request-%d.result.json'):format(generation))
  -- Written under a temporary name and renamed: the authority never reads half a request.
  path.write(request..'.partial',json.encode({schema_version=1,module=module.module,hero=options.hero,
    file=copy,sha256=digest,generation=generation}),true,true)
  assert(os.rename(request..'.partial',request))
  log(('[hero-hot-reload] requested %s generation %d (sha256 %s)'):format(module.module,generation,digest))
  local deadline=os.time()+options.timeout
  while os.time()<=deadline do
    if path.is_file(result_file) then
      local result=json.decode(path.read(result_file))
      if result.ok then
        log(('[hero-hot-reload] %s generation %d active from tick %s'):format(module.module,generation,tostring(result.tick)))
      else
        error('the authority refused the hot reload: '..tostring(result.error),0)
      end
      return result
    end
    (deps.sleep_ms or time.sleep_ms)(200)
  end
  error(('no answer from the authority within %ds; is it running with OMB_SCRIPT_HOT_DIR=%s?'):format(options.timeout,hot_dir),0)
end

if not pcall(debug.getlocal,4,1) then
  local options=M.options(arg)
  if options.help then
    print('usage: tools\\lua\\lua.exe scripts\\hero_hot_reload.lua --hero <hero id> --run <run dir> [--no-generate] [--no-build] [--timeout SECONDS] [--target-dir DIR]')
    return
  end
  local ok,error_message=pcall(M.run,options)
  if not ok then io.stderr:write(tostring(error_message)..'\n');os.exit(1) end
end
return M
