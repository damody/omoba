-- Puts skill-set variant heroes (D22: packages/<hero>_old, <hero>_v20) in the game, in one
-- land_to_live batch:
--
--   tools\lua\lua.exe scripts\land_hero_variants.lua [--hero PACKAGE ...] [--all [--except PACKAGE ...]] [--with SRC=LIVE ...] [--dry-run]
--
-- For each package whose v3 document is a variant (source.variant old / v20) and whose every
-- ability is generated Rust: its crate scripts/heroes/<package> (v3_rust.lua scaffold), its row in
-- scripts/hero_dlls.lua and its playable entry in scripts/v3_test_heroes.lua, then
-- templates/v3_test_heroes.lua regenerated, and its rlib linked into base_content's tests (Cargo.toml
-- dev-dependencies and the test manifest in src/lib.rs, as the base heroes are). A package that is
-- not ready is reported and left out.
-- --all takes every variant package not yet in the game. The shadow files are under
-- target/hero-variants/<time>/; --dry-run stops before land_to_live. --with adds a file of the same
-- batch, e.g. a variant's hand-written hook (scripts/heroes/<package>/src/hooks/<id>.rs).
local source=debug.getinfo(1,'S').source:sub(2)
local scripts_dir=assert(source:match('^(.*)[/\\]'))
package.path=scripts_dir..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process=b.lib('path'),b.lib('process')
local lfs=require('lfs')
local M={}

M.VARIANTS={old=true,v20=true}

local function read(file) local f=assert(io.open(file,'rb'));local t=f:read('a');f:close();return t end
local function write(file,text) path.mkdir_p(file:match('^(.*)[/\\]'));local f=assert(io.open(file,'wb'));f:write(text);f:close() end
local function copy(from,to) write(to,read(from)) end
-- insert `text` (LF lines) before the end of the table that opens at `opening`, keeping CRLF
local function append_to_list(body,opening,text)
  local crlf=body:find('\r\n',1,true)~=nil
  local start=assert(body:find(opening,1,true),'no '..opening)
  local close=assert(body:find('\r?\n}',start),'no end of '..opening)
  if crlf then text=text:gsub('\r?\n','\r\n') end
  local nl=crlf and '\r\n' or '\n'
  return body:sub(1,close-1)..nl..text..body:sub(close)
end

-- insert `text` (LF lines) after the last line matching `pattern`, keeping CRLF
local function append_after_last(body,pattern,text)
  local crlf=body:find('\r\n',1,true)~=nil
  local last
  local pos=1
  while true do
    local s,e=body:find(pattern,pos)
    if not s then break end
    last=e;pos=e+1
  end
  assert(last,'no line matching '..pattern)
  local eol=body:find('\n',last,true) or #body
  if crlf then text=text:gsub('\r?\n','\r\n') end
  local nl=crlf and '\r\n' or '\n'
  return body:sub(1,eol)..text..nl..body:sub(eol+1)
end

local function rust()
  V3_RUST_AS_MODULE=true
  local r=dofile(path.join(b.root,'docs','character_pipeline','tools','v3_rust.lua'))
  V3_RUST_AS_MODULE=nil
  return r
end

-- package -> { package, crate (DLL name), ready, why }
function M.inspect(packages)
  local r=rust()
  local by={}
  for _,h in ipairs(r.heroes(b.root,'all')) do by[h.package]=h end
  local out={}
  for _,package in ipairs(packages) do
    local h=by[package]
    local row={package=package}
    if not h then row.why='no v3 document'
    else
      local chunk=assert(loadfile(h.path,'t',require('hero_author.cli').safe_env()))
      local doc=chunk();if type(doc)=='function' then doc=doc({}) end
      local variant=doc and doc.source and doc.source.variant
      local reports,problems=r.run(b.root,'scripts',{h},{write=false})
      if not M.VARIANTS[variant] then row.why='not a skill-set variant (source.variant '..tostring(variant)..')'
      elseif #problems>0 then row.why=tostring(problems[1])
      else
        local blocked={}
        for _,e in ipairs(reports[1].entries) do if #e.blockers>0 then blocked[#blocked+1]=e.id end end
        if #blocked>0 then row.why='not generated: '..table.concat(blocked,', ')
        else row.ready=true;row.crate='hero_'..doc.id:lower() end
      end
    end
    out[#out+1]=row
  end
  return out
end

-- every variant package that the game does not list yet
function M.pending()
  local listed={}
  for _,row in ipairs(dofile(path.join(b.root,'scripts','hero_dlls.lua'))) do listed[row.hero]=true end
  local out={}
  local dir=path.join(b.root,'docs','character_pipeline','packages')
  for name in lfs.dir(dir) do
    for suffix in pairs(M.VARIANTS) do
      if name:match('_'..suffix..'$') and not listed[name] and path.is_file(path.join(dir,name,'v3.lua')) then out[#out+1]=name end
    end
  end
  table.sort(out)
  return out
end

function M.run(options)
  local packages=options.heroes
  if options.all then
    packages={}
    for _,package in ipairs(M.pending()) do if not options.except[package] then packages[#packages+1]=package end end
  end
  assert(#packages>0,'no variant packages to land (--hero PACKAGE or --all)')
  local rows=M.inspect(packages)
  local ready={}
  for _,row in ipairs(rows) do
    if row.ready then ready[#ready+1]=row;print(('ready   %s (%s)'):format(row.package,row.crate))
    else print(('skipped %s: %s'):format(row.package,row.why)) end
  end
  if #ready==0 then print('nothing to land');return 1 end
  local shadow=path.join(b.root,'target','hero-variants',os.date('%Y%m%d-%H%M%S'))
  local files={
    dlls={live='scripts/hero_dlls.lua'},
    cargo={live='scripts/base_content/Cargo.toml'},
    lib={live='scripts/base_content/src/lib.rs'},
    heroes={live='scripts/v3_test_heroes.lua'},
    out={live='scripts/lua_data/templates/v3_test_heroes.lua'},
  }
  for name,f in pairs(files) do
    f.src=path.join(shadow,name..'.lua');f.base=f.src..'.base'
    copy(path.join(b.root,f.live),f.src);copy(path.join(b.root,f.live),f.base)
  end
  local dlls,heroes=read(files.dlls.src),read(files.heroes.src)
  local cargo,lib=read(files.cargo.src),read(files.lib.src)
  local changes={}
  local r=rust()
  for _,row in ipairs(ready) do
    if not dlls:find("hero='"..row.package.."'",1,true) then
      dlls=append_to_list(dlls,'return {',("  {hero='%s',package='%s',dir='heroes/%s'},"):format(row.package,row.crate,row.package))
    end
    if not heroes:find('package = "'..row.package..'"',1,true) then
      heroes=append_to_list(heroes,'local HEROES = {',('  { package = "%s", playable = true },'):format(row.package))
    end
    -- base_content's tests link every hero rlib (FC tests learn the variant's abilities)
    local impl=row.crate..'_impl'
    if not cargo:find('\n'..impl..' = ',1,true) then
      cargo=append_after_last(cargo,'\nhero_[%w_]+_impl = { path = "../heroes/[%w_]+" }',('%s = { path = "../heroes/%s" }'):format(impl,row.package))
    end
    if not lib:find(impl..'::units()',1,true) then
      lib=append_after_last(lib,'\n *v%.extend%(hero_[%w_]+_impl::units%(%)%);','        v.extend('..impl..'::units());')
      lib=append_after_last(lib,'\n *v%.extend%(hero_[%w_]+_impl::abilities%(%)%);','        v.extend('..impl..'::abilities());')
    end
    if not path.is_directory(path.join(b.root,'scripts','heroes',row.package)) then
      local out=path.join(shadow,'crates')
      process.run(b.lib('platform').lua_executable,{path.join(b.root,'docs','character_pipeline','tools','v3_rust.lua'),
        '--out',out,'--hero',row.package},{cwd=b.root,label='v3_rust '..row.package})
      local dir=path.join(out,'heroes',row.package)
      for _,rel in ipairs({'Cargo.toml','build.rs','src/lib.rs','dll/Cargo.toml','dll/src/lib.rs'}) do
        local file=path.join(dir,rel)
        assert(path.is_file(file),row.package..': v3_rust.lua wrote no '..rel)
        changes[#changes+1]=file..'=scripts/heroes/'..row.package..'/'..rel
      end
    end
  end
  write(files.dlls.src,dlls);write(files.heroes.src,heroes);write(files.cargo.src,cargo);write(files.lib.src,lib)
  -- regenerate templates/v3_test_heroes.lua from the shadow list, against the shadow hero_dlls
  local script=read(files.heroes.src)
  local function rep(a,c) local i,j=script:find(a,1,true);assert(i,'v3_test_heroes.lua: '..a);script=script:sub(1,i-1)..c..script:sub(j+1) end
  local slash=function(p) return (p:gsub('\\','/')) end
  rep('local OUT = "scripts/lua_data/templates/v3_test_heroes.lua"','local OUT = "'..slash(files.out.src)..'"')
  rep('dofile("scripts/hero_dlls.lua")','dofile("'..slash(files.dlls.src)..'")')
  local runner=path.join(shadow,'regenerate.lua')
  write(runner,script)
  process.run(b.lib('platform').lua_executable,{runner},{cwd=b.root,label='v3_test_heroes'})
  for _,f in pairs(files) do changes[#changes+1]=f.src..'='..f.live..'@'..f.base end
  for _,extra in ipairs(options.with or {}) do changes[#changes+1]=extra end
  table.sort(changes)
  local args={path.join(b.root,'scripts','land_to_live.lua')}
  for _,c in ipairs(changes) do args[#args+1]='--change';args[#args+1]=c end
  print(('%d file(s) in %s'):format(#changes,shadow))
  if options.dry_run then print('dry run: not landed');return 0 end
  local result=process.run(b.lib('platform').lua_executable,args,{cwd=b.root,check=false,label='land_to_live'})
  io.write(result.stdout or '');io.stderr:write(result.stderr or '')
  return result.exit_code
end

function M.main(argv)
  local options={heroes={},except={}}
  local i=1
  while i<=#argv do
    local key=argv[i]
    if key=='--hero' then options.heroes[#options.heroes+1]=assert(argv[i+1],'--hero needs a package'):match('^%s*(.-)%s*$');i=i+1
    elseif key=='--all' then options.all=true
    elseif key=='--except' then options.except[assert(argv[i+1],'--except needs a package')]=true;i=i+1
    elseif key=='--with' then
      local extra=assert(argv[i+1],'--with needs SRC=LIVE');assert(extra:find('=',1,true),'--with needs SRC=LIVE')
      options.with=options.with or {};options.with[#options.with+1]=extra;i=i+1
    elseif key=='--dry-run' then options.dry_run=true
    else error('unknown argument: '..key) end
    i=i+1
  end
  return M.run(options)
end

if arg and arg[0] and arg[0]:match('land_hero_variants%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
