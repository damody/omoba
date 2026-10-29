-- Per-run content artifacts (E124). Every launch copies the current base_content build into
-- its own run directory and points the server, the client runtimes and the Unreal run
-- snapshot at that copy. A shared scripts/base_content.dll went stale whenever another
-- session rebuilt base_content between its stage and a launch, and the launch preflight
-- refused the run (E002, E106, E124); a copy that only this run reads cannot go stale, and
-- the build output is never mapped by a running server, so later builds never fail to link.
--
-- Layout: <run>/artifacts/scripts/base_content.dll (the only DLL there: the loaders read
-- every script DLL of OMB_SCRIPTS_DIR) and <run>/artifacts/base_content.json (profile,
-- source, size, SHA-256).
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.PROFILES={'debug','release'}
M.ATTEMPTS=3

local function deps_of(deps)
  deps=deps or {}
  local b=require('_bootstrap')
  return {
    path=deps.path or b.lib('path'),
    lfs=deps.lfs or require('lfs'),
    sha256=deps.sha256 or b.lib('hash').sha256,
    json=deps.json or b.lib('json'),
    getenv=deps.getenv or os.getenv,
  }
end

-- Where a launch takes its Rust binaries from: OM_RUST_RUN_BINARIES names one cargo target
-- directory (a session's own CARGO_TARGET_DIR) for every crate; otherwise each crate's own
-- target directory (omb/target, omoba-client-runtime/target, scripts/target). Every launcher
-- and preflight resolves through here, as OM_UE_RUN_BINARIES does for Unreal (E125).
function M.rust_target(root,crate,deps)
  local d=deps_of(deps)
  local env=os.getenv('OM_RUST_RUN_BINARIES')
  if env and env~='' then return d.path.absolute(env,root) end
  return d.path.join(root,crate,'target')
end
function M.rust_binary(root,crate,profile,name,deps)
  return deps_of(deps).path.join(M.rust_target(root,crate,deps),profile,name)
end

local function stat(d,file)
  local a=d.lfs.attributes(file)
  return a and a.mode=='file' and {size=a.size,modification=a.modification} or nil
end

-- The newest base_content build of either profile: the frontend build makes debug, a
-- release server build makes release, and either is a build of the same sources. PREFERRED
-- wins a tie (the profile this workflow just built).
function M.built_base_content(root,deps,preferred)
  local d=deps_of(deps)
  local best
  for _,profile in ipairs(M.PROFILES) do
    local file=M.rust_binary(root,'scripts',profile,'base_content.dll',d)
    local a=stat(d,file)
    if a and (not best or a.modification>best.modification
      or (a.modification==best.modification and profile==preferred)) then
      best={file=file,profile=profile,size=a.size,modification=a.modification}
    end
  end
  return best
end

local function copy(source_file,destination)
  local input=assert(io.open(source_file,'rb'))
  local output=assert(io.open(destination,'wb'))
  while true do
    local chunk=input:read(4*1024*1024)
    if not chunk then break end
    output:write(chunk)
  end
  input:close();output:close()
end

-- Build freshness (E190). The content marker only proves the catalog data matches; a launch
-- with --no-build from a session target (OM_RUST_RUN_BINARIES) ran a server linked before
-- a gameplay fix landed, and its acceptance run failed on a divergence live had already
-- fixed. cargo writes <target>/<profile>/<name>.d beside every binary it links, listing each
-- source file rustc read (crate sources, include_str! files, generated OUT_DIR sources); a
-- source newer than the binary means cargo would relink it. Returns those sources (newest
-- first), or nil when the binary has no dep-info (not a cargo build, cannot be checked).
-- OMOBA_SOURCES_AS_OF (unix seconds, set by a launcher that starts several role launchers
-- for one match) tolerates sources changed after that moment: the authority was checked
-- when the match started, and a client started minutes later must not be refused because
-- another session landed a change meanwhile (E202). A deleted source is always stale.
function M.dep_info_sources(text)
  local body=text:match('^.-: (.-)\r?\n') or text:match('^.-: (.*)$') or ''
  local sources,current={},{}
  local i=1
  while i<=#body do
    local c=body:sub(i,i)
    if c=='\\' and body:sub(i+1,i+1)==' ' then current[#current+1]=' ';i=i+1
    elseif c==' ' then
      if #current>0 then sources[#sources+1]=table.concat(current);current={} end
    else current[#current+1]=c end
    i=i+1
  end
  if #current>0 then sources[#sources+1]=table.concat(current) end
  for index,source_file in ipairs(sources) do sources[index]=source_file:gsub('^\\\\%?\\','') end
  return sources
end
function M.stale_sources(binary,deps)
  local d=deps_of(deps)
  local built=stat(d,binary)
  local dep_info=binary:gsub('%.[^./\\]+$','')..'.d'
  if not built or not d.path.is_file(dep_info) then return nil end
  local stale={}
  local as_of=math.tointeger(tonumber(d.getenv('OMOBA_SOURCES_AS_OF') or ''))
  for _,source_file in ipairs(M.dep_info_sources(d.path.read(dep_info,true))) do
    local a=stat(d,source_file)
    -- A listed directory (a tracked content root) changes time whenever any file in it is
    -- written; the files rustc actually read are listed one by one.
    -- Relative entries (deps/*.d of a test binary) have no known base and are not checked.
    if d.lfs.attributes(source_file,'mode')=='directory' or not source_file:match('^%a:[/\\]') and not source_file:match('^[/\\]') then
    elseif not a then stale[#stale+1]={file=source_file,missing=true,modification=math.huge}
    elseif a.modification>built.modification and not (as_of and a.modification>as_of) then
      stale[#stale+1]={file=source_file,modification=a.modification}
    end
  end
  table.sort(stale,function(a,b) return a.modification>b.modification end)
  return stale
end
-- Refuses a binary built before one of its sources changed. ROLE names it in the error.
function M.verify_current_build(role,binary,deps)
  local stale=M.stale_sources(binary,deps)
  if not stale or #stale==0 then return end
  local shown={}
  for index=1,math.min(#stale,5) do
    shown[#shown+1]='  '..stale[index].file..(stale[index].missing and ' (deleted)' or '')
  end
  error(('%s was built before %d of its sources changed: %s\n%s\n'..
    'Rebuild it (drop --no-build, or cargo build with the same CARGO_TARGET_DIR); a launch '..
    'would test code that is not the current source.'):format(role,#stale,binary,table.concat(shown,'\n')),0)
end

-- Copies the current build into RUN_DIR and returns {scripts_dir, dll, manifest, profile,
-- source, sha256}. A build that relinks the DLL while it is copied is detected (size/time
-- changed) and the copy retried; the digest is of the copy, so it is what this run loads.
-- FROM names another run's staged copy instead: processes of one match on one machine (a
-- split authority and its clients) must load identical content, so a later invocation
-- copies the first one's DLL rather than whatever is newest by then.
function M.stage_base_content(root,run_dir,deps,preferred,from)
  local d=deps_of(deps)
  local scripts_dir=d.path.join(run_dir,'artifacts','scripts')
  local dll=d.path.join(scripts_dir,'base_content.dll')
  assert(not d.path.exists(dll),'run already has a staged base_content: '..dll)
  d.path.mkdir_p(scripts_dir)
  for _=1,M.ATTEMPTS do
    local built
    if from then
      local a=stat(d,from)
      assert(a,'missing base_content to share: '..from)
      built={file=from,profile='shared',size=a.size,modification=a.modification}
    else
      built=M.built_base_content(root,d,preferred)
      assert(built,'no base_content build in scripts/target/{debug,release}; build it before launch')
      M.verify_current_build('base_content',built.file,d)
    end
    copy(built.file,dll)
    local after=stat(d,built.file)
    if after and after.size==built.size and after.modification==built.modification then
      local report={schema_version=1,scope='run-base-content',profile=built.profile,source=built.file,
        size=built.size,modification=built.modification,sha256=d.sha256(dll):lower(),dll=dll}
      local manifest=d.path.join(run_dir,'artifacts','base_content.json')
      d.path.write(manifest,d.json.encode(report),true)
      report.scripts_dir=scripts_dir;report.manifest=manifest
      return report
    end
    os.remove(dll)
  end
  error('base_content kept changing while it was staged for this run; is a build running?',0)
end

-- Executables (E125). A running omobab.exe / omoba-client-runtime.exe locks its file, so
-- another session's cargo build failed (os error 5) or rebuilt only some binaries, and a
-- later launch mixed generations (the bridge rejected the runtime's content hash and the
-- renderer never received a frame). Launches run copies from target/run-bin/<key>/ instead;
-- <key> is derived from the sources' names, sizes and times, so a rebuild yields a new copy
-- and runs already started keep theirs. Old copies are renamed aside before deletion:
-- Windows refuses to rename a directory whose executable is running, so one in use is kept.
M.KEEP_BINARIES=6

local function fnv(text,h)
  h=h or 0xcbf29ce484222325
  for byte in text:gmatch('.') do h=((h ~ byte:byte())*0x100000001b3)&0xffffffffffffffff end
  return h
end

-- FILES: {{name=..., source=...}, ...}. Returns the key or nil when a source is missing.
function M.binaries_key(files,deps)
  local d=deps_of(deps)
  local h
  for _,file in ipairs(files) do
    local a=stat(d,file.source)
    if not a then return nil end
    h=fnv(file.name..'|'..a.size..'|'..a.modification..'\n',h)
  end
  return ('%016x'):format(h or 0)
end

local function remove_tree(d,dir)
  for name in d.lfs.dir(dir) do
    if name~='.' and name~='..' then
      local entry=d.path.join(dir,name)
      if d.lfs.attributes(entry,'mode')=='directory' then remove_tree(d,entry) else assert(os.remove(entry)) end
    end
  end
  assert(d.lfs.rmdir(dir))
end

local function prune_binaries(d,base,current)
  local ready={}
  for name in d.lfs.dir(base) do
    local dir=d.path.join(base,name)
    if name~='.' and name~='..' and name~=current and d.path.is_directory(dir) then
      local marker=d.path.join(dir,'ready.txt')
      if name:match('^%x+$') and d.path.is_file(marker) then
        ready[#ready+1]={dir=dir,time=d.lfs.attributes(marker,'modification')}
      elseif not name:match('^%x+$') and os.time()-(d.lfs.attributes(dir,'modification') or 0)>3600 then
        ready[#ready+1]={dir=dir,time=0,stale=true}
      end
    end
  end
  table.sort(ready,function(a,b) return a.time>b.time end)
  for index,item in ipairs(ready) do
    if item.stale or index>=M.KEEP_BINARIES then
      local target=item.dir:match('%.trash%-') and item.dir or item.dir..'.trash-'..os.time()
      if target==item.dir or os.rename(item.dir,target) then pcall(remove_tree,d,target) end
    end
  end
end

-- Returns {dir, key, files={name=copy}}; the copies keep their names (logs and process
-- listings stay readable) and their times.
function M.stage_binaries(root,files,deps)
  local d=deps_of(deps)
  local base=d.path.join(root,'target','run-bin')
  for _,file in ipairs(files) do
    if stat(d,file.source) then M.verify_current_build(file.name,file.source,d) end
  end
  d.path.mkdir_p(base)
  for attempt=1,M.ATTEMPTS do
    local key=M.binaries_key(files,d)
    if not key then
      for _,file in ipairs(files) do assert(stat(d,file.source),'missing executable: '..file.source) end
    end
    local dir=d.path.join(base,key)
    local result={dir=dir,key=key,files={}}
    for _,file in ipairs(files) do result.files[file.name]=d.path.join(dir,file.name) end
    local marker=d.path.join(dir,'ready.txt')
    if d.path.is_file(marker) then
      local now=os.time();d.lfs.touch(marker,now,now) -- most recently used is pruned last
      prune_binaries(d,base,key)
      return result
    end
    local staging=dir..'.partial-'..os.time()..'-'..math.random(0,999999)
    d.path.mkdir_p(staging)
    for _,file in ipairs(files) do
      local copy_path=d.path.join(staging,file.name)
      copy(file.source,copy_path)
      local modified=d.lfs.attributes(file.source,'modification')
      d.lfs.touch(copy_path,modified,modified)
    end
    if M.binaries_key(files,d)==key then
      d.path.write(d.path.join(staging,'ready.txt'),key..'\n',true)
      if os.rename(staging,dir) or d.path.is_file(marker) then
        if d.path.is_directory(staging) then remove_tree(d,staging) end
        prune_binaries(d,base,key)
        return result
      end
    end
    remove_tree(d,staging)
  end
  error('executables kept changing while they were staged for this run; is a build running?',0)
end

-- The content catalog hash a built server, runtime or bridge carries (CONTENT_CATALOG_MARKER,
-- E125), or nil for a binary built before the marker existed.
M.MARKER='OM_CONTENT_CATALOG_DATA_HASH='
function M.content_hash(file)
  local input=assert(io.open(file,'rb'))
  local tail=''
  while true do
    local chunk=input:read(4*1024*1024)
    if not chunk then break end
    local text=tail..chunk
    local hash=text:match(M.MARKER..'(%x+);')
    if hash then input:close();return hash:lower() end
    tail=text:sub(-(#M.MARKER+80))
  end
  input:close()
  return nil
end

-- Refuses a launch whose binaries carry different content: the renderer's bridge rejects a
-- runtime whose content hash differs from its own and then shows no frame at all. ROLES is
-- {{role=..., file=...}, ...}; EXPECTED is the authority's catalog content hash. A binary
-- without a marker (built before E125) cannot be checked and is reported, not refused.
function M.verify_content(roles,expected,log)
  log=log or print
  local seen={}
  for _,item in ipairs(roles) do
    local hash=M.content_hash(item.file)
    if hash then seen[#seen+1]=item.role..'='..hash..' ('..item.file..')'
    else log('[run-artifacts] '..item.role..' has no content marker (built before E125); rebuild it to have it checked: '..item.file) end
    item.hash=hash
  end
  for _,item in ipairs(roles) do
    if item.hash and item.hash~=expected:lower() then
      error(('launch binaries carry different content: authority content=%s, %s\n'..
        'Rebuild the stale one (the renderer would reject the runtime and show no frame).'):format(
        expected,table.concat(seen,', ')),0)
    end
  end
  return roles
end

return M
