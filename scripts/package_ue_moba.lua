-- Packaged (cooked, staged, archived) Windows build of omfue through the engine's
-- own BuildCookRun. Editor success does not prove a package works (README §8 L4),
-- so the archive is checked for the game executable, the Rust bridge, the script
-- DLL and the HUD manifest, and a file manifest with SHA-256 is written. Never
-- deletes an existing archive: every run gets a new output directory.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,json,hash=b.lib('path'),b.lib('process'),b.lib('json'),b.lib('hash')
local lfs=require('lfs')
local consistent=require('consistent_build')
local M={}

-- A cooked build reads the same sources as the consistent build. A pass during
-- which another writer edited them is not a verdict (UHT output no longer
-- matches the headers, or the archive is stale): wait for quiet and retry.
M.MAX_PASSES=4

M.REQUIRED={
  {role='game executable',pattern='^OmGame%.exe$'},
  {role='rust bridge',pattern='^om_bridge%.dll$'},
  {role='script dll',pattern='^base_content%.dll$'},
  {role='pak',pattern='%.pak$'},
}

function M.options(args)
  local options={ue_root='D:/UE5.8',config='Development'}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--ue-root' or key=='--config' or key=='--output' then
      i=i+1;options[key:sub(3):gsub('%-','_')]=assert(args[i],key..' requires a value')
    else error('unknown argument: '..key) end
    i=i+1
  end
  assert(options.config=='Development' or options.config=='Shipping','--config must be Development or Shipping')
  return options
end

function M.uat_args(options,project,archive)
  local args={'/d','/c',path.join(options.ue_root,'Engine','Build','BatchFiles','RunUAT.bat'),'BuildCookRun',
    '-project='..project,'-noP4','-platform=Win64','-clientconfig='..options.config,'-serverconfig='..options.config,
    '-build','-cook','-stage','-pak','-archive','-archivedirectory='..archive,'-utf8output','-unattended','-nocompileeditor'}
  -- A release archive is what players receive: no ~1.8 GB symbol file in it.
  if options.config=='Shipping' then args[#args+1]='-nodebuginfo' end
  return args
end

-- Every file under the archive with size and SHA-256; required roles must all be present.
function M.inspect(archive,sha256)
  sha256=sha256 or hash.sha256
  local files,found={},{}
  local function walk(dir,rel)
    for name in lfs.dir(dir) do
      if name~='.' and name~='..' then
        local full=dir..'/'..name
        local relative=rel=='' and name or (rel..'/'..name)
        local mode=lfs.attributes(full,'mode')
        if mode=='directory' then walk(full,relative)
        elseif mode=='file' then
          files[#files+1]={path=relative,size=lfs.attributes(full,'size'),sha256=sha256(full)}
          for _,rule in ipairs(M.REQUIRED) do if name:match(rule.pattern) then found[rule.role]=relative end end
        end
      end
    end
  end
  walk(archive,'')
  table.sort(files,function(a,b) return a.path<b.path end)
  local missing={}
  for _,rule in ipairs(M.REQUIRED) do if not found[rule.role] then missing[#missing+1]=rule.role end end
  return {files=files,found=found,missing=missing}
end

function M.run(options,deps)
  local project=path.join(b.root,'omfue','om.uproject')
  local root=options.output and path.absolute(options.output,b.root)
  if not root then
    local parent=path.mkdir_p(path.join(b.root,'target','ue-package'))
    for suffix=1,1000 do
      local candidate=path.join(parent,os.time()..'-'..suffix)
      if lfs.mkdir(candidate) then root=candidate;break end
    end
  else
    assert(not path.exists(root),'output must be a new directory: '..root);path.mkdir_p(root)
  end
  assert(root,'cannot reserve package directory')
  deps=deps or {}
  local runner=deps.runner or function(args) return process.run('cmd.exe',args,{cwd=b.root,check=false}) end
  local fingerprint=deps.fingerprint or function() return consistent.fingerprint(b.root) end
  local newest=deps.newest_input or function() return consistent.newest_input(b.root) end
  local now,sleep=deps.now or os.time,deps.sleep_ms or function(ms) b.lib('time').sleep_ms(ms) end
  local quiet=deps.quiet_seconds or consistent.START_QUIET_SECONDS
  local report={schema_version=2,config=options.config,project=project,passes={}}
  local result,archive,log
  for pass=1,M.MAX_PASSES do
    local waited=0
    while now()-newest()<quiet and waited<consistent.MAX_START_WAIT_SECONDS do
      if waited==0 then print(('package: waiting for %ds of quiet sources before pass %d'):format(quiet,pass)) end
      sleep(consistent.START_POLL_MS);waited=waited+consistent.START_POLL_MS//1000
    end
    archive=path.join(root,pass==1 and 'archive' or ('archive-'..pass))
    log=path.join(root,pass==1 and 'buildcookrun.log' or ('buildcookrun-'..pass..'.log'))
    print('Packaging omfue ('..options.config..') into '..archive..'; log '..log)
    local before=fingerprint()
    result=runner(M.uat_args(options,project,archive))
    local busy=0
    while result.exit_code~=0 and consistent.ubt_busy(result) do
      busy=busy+1
      assert(busy<=consistent.MAX_BUSY_RETRIES,'package: UnrealBuildTool stayed busy for '..busy..' waits')
      print(('package: another UnrealBuildTool holds the build mutex; waiting %ds (retry %d/%d)')
        :format(consistent.BUSY_WAIT_MS//1000,busy,consistent.MAX_BUSY_RETRIES))
      sleep(consistent.BUSY_WAIT_MS)
      before=fingerprint()
      result=runner(M.uat_args(options,project,archive))
    end
    path.write(log,(result.stdout or '')..'\n'..(result.stderr or ''),true)
    local changed=fingerprint()~=before
    report.passes[#report.passes+1]={archive=archive,log=log,exit_code=result.exit_code,sources_changed=changed}
    if not changed then break end
    print(('package: sources changed during pass %d (%s); its result is not a verdict, retrying'):format(pass,
      result.exit_code==0 and 'archive would be stale' or 'failure while another writer was mid-edit'))
    assert(pass<M.MAX_PASSES,'package: sources kept changing for '..M.MAX_PASSES..' passes; another writer is active')
  end
  report.archive=archive;report.exit_code=result.exit_code
  if result.exit_code==0 and path.is_directory(archive) then
    local inspected=(deps.inspect or M.inspect)(archive)
    report.files=inspected.files;report.found=inspected.found;report.missing=inspected.missing
  end
  path.write(path.join(root,'package-report.json'),json.encode(report),true)
  assert(result.exit_code==0,'BuildCookRun failed with exit '..tostring(result.exit_code)..'; see '..log)
  assert(#report.missing==0,'package is missing: '..table.concat(report.missing,', ')..'; see '..path.join(root,'package-report.json'))
  print(('Package ready: %d files; game %s'):format(#report.files,report.found['game executable']))
  return root
end

if arg and arg[0] and arg[0]:match('package_ue_moba%.lua$') then
  if arg[1]=='--help' then
    print('Usage: tools/lua/lua.exe scripts/package_ue_moba.lua [--ue-root D:/UE5.8] [--config Development|Shipping] [--output NEW_DIRECTORY]')
    print('Runs the engine BuildCookRun for Win64, then checks the archive for OmGame.exe, om_bridge.dll, base_content.dll and paks and writes package-report.json with SHA-256.')
    return
  end
  M.run(M.options(arg))
end
return M
