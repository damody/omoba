package.path='scripts/?.lua;'..package.path
local pkg=require('package_ue_moba')
local lfs=require('lfs')

assert(pkg.options({}).config=='Development')
assert(pkg.options({'--config','Shipping'}).config=='Shipping')
assert(not pcall(pkg.options,{'--config','Debug'}))
assert(not pcall(pkg.options,{'--bogus'}))
local args=table.concat(pkg.uat_args({ue_root='U',config='Development'},'P.uproject','OUT'),' ')
for _,needle in ipairs({'RunUAT.bat','BuildCookRun','-project=P.uproject','-platform=Win64','-cook','-stage','-pak','-archivedirectory=OUT','-clientconfig=Development'}) do
  assert(args:find(needle,1,true),'missing '..needle)
end

assert(table.concat(pkg.uat_args({ue_root='U',config='Shipping'},'P.uproject','OUT'),' '):find('-nodebuginfo',1,true),'Shipping archive has no symbols')
assert(not args:find('-nodebuginfo',1,true),'Development keeps symbols for diagnostics')

-- Inspection finds the required roles in a nested archive and reports what is missing.
local root=os.getenv('TEMP')..'/omoba-package-test-'..os.time()
assert(lfs.mkdir(root));assert(lfs.mkdir(root..'/Windows'));assert(lfs.mkdir(root..'/Windows/OmGame'))
local function touch(p) local f=assert(io.open(p,'wb'));f:write('x');f:close() end
touch(root..'/Windows/OmGame.exe');touch(root..'/Windows/om_bridge.dll')
local report=pkg.inspect(root,function() return string.rep('a',64) end)
assert(report.found['game executable']=='Windows/OmGame.exe' and #report.files==2)
assert(table.concat(report.missing,',')=='script dll,pak')
touch(root..'/Windows/base_content.dll');touch(root..'/Windows/OmGame/pakchunk0-Windows.pak')
assert(#pkg.inspect(root,function() return string.rep('a',64) end).missing==0)
for _,p in ipairs({'/Windows/OmGame/pakchunk0-Windows.pak','/Windows/base_content.dll','/Windows/om_bridge.dll','/Windows/OmGame.exe'}) do os.remove(root..p) end
lfs.rmdir(root..'/Windows/OmGame');lfs.rmdir(root..'/Windows');lfs.rmdir(root)
-- A pass during which another writer edited the sources is not a verdict:
-- it retries into a fresh archive and only a stable pass is reported.
do
  local b=require('_bootstrap');local path,json=b.lib('path'),b.lib('json')
  local function run(outcomes,label)
    local out=path.join(b.root,'target','tests','package-retry-'..label..'-'..os.time())
    local pass,fingerprint=0,0
    local ok,err=pcall(pkg.run,{ue_root='U',config='Shipping',output=out},{
      runner=function(args)
        pass=pass+1
        local archive;for _,a in ipairs(args) do archive=archive or a:match('^%-archivedirectory=(.*)$') end;path.mkdir_p(archive)
        local outcome=outcomes[pass];if outcome.edit then fingerprint=fingerprint+1 end
        return {exit_code=outcome.exit,stdout=outcome.stdout or ('pass '..pass),stderr=''}
      end,
      fingerprint=function() return tostring(fingerprint) end,newest_input=function() return 0 end,now=function() return 1000 end,
      sleep_ms=function() end,quiet_seconds=0,inspect=function() return {files={},found={['game executable']='Windows/OmGame.exe'},missing={}} end})
    return ok,err,out,pass
  end
  local ok,_,out,passes=run({{exit=6,edit=true},{exit=0}},'recover')
  assert(ok and passes==2,'mid-edit failure retried once')
  local report=json.read(path.join(out,'package-report.json'))
  assert(report.exit_code==0 and report.archive:match('archive%-2$') and #report.passes==2)
  assert(report.passes[1].sources_changed and report.passes[1].exit_code==6 and not report.passes[2].sources_changed)
  -- A held UnrealBuildTool mutex waits and retries the same pass.
  local mutex_ok,_,mutex_out,mutex_passes=run({{exit=10,stdout='Result: Failed (ConflictingInstance)'},{exit=0}},'mutex')
  assert(mutex_ok and mutex_passes==2 and #json.read(path.join(mutex_out,'package-report.json')).passes==1,'mutex wait is not a pass')
  local busy_ok,busy_err=run({{exit=0,edit=true},{exit=0,edit=true},{exit=0,edit=true},{exit=0,edit=true}},'busy')
  assert(not busy_ok and tostring(busy_err):find('kept changing',1,true),'never reports a stale archive')
end
print('package workflow: options, UAT arguments and archive inspection passed')
