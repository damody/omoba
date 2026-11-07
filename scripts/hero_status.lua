-- Status of every hero, in M6 batch order, from the code and the data that exist now.
-- docs/plans/2026-10-28-history-nobu-full-port/all-heroes-status.md is its output; do not edit it.
--
--   tools\lua\lua.exe scripts\hero_status.lua [--out FILE] [--run NAME]   write the table
--   tools\lua\lua.exe scripts\hero_status.lua sweep [--run NAME] [--matches 2] [--parallel 8] [--batch N] [--no-build]
--       headless matches for every batch through verify_m6_batch (target/hero-status/runs/NAME/); a group
--       that already has its report is kept, so a stopped sweep continues with the same --run.
--
-- What is not measured says 未驗證. No evidence is never read as a pass.
local source=debug.getinfo(1,'S').source:sub(2)
local scripts_dir=assert(source:match('^(.*)[/\\]'))
package.path=scripts_dir..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,json=b.lib('path'),b.lib('process'),b.lib('json')
local lfs=require('lfs')
local M={}

M.OUT='docs/plans/2026-10-28-history-nobu-full-port/all-heroes-status.md'
M.RUNS='target/hero-status/runs'
M.GROUP=10

-- The skill-set versions (D22): package <base>_<suffix>, its own hero wearing the base's body.
M.VARIANTS={{suffix='old',title='舊版技能組'},{suffix='v20',title='2.0 版技能組'}}
-- package -> its base hero's package and the version suffix; nil for a base hero
function M.base_of(package,is_package)
  for _,variant in ipairs(M.VARIANTS) do
    local base=package:match('^(.+)_'..variant.suffix..'$')
    if base and is_package[base] then return base,variant.suffix end
  end
end

-- The batches as m6-waves.md lists them: that order is the plan everything else refers to. v3_waves.lua
-- would now give a somewhat different order, because the documents changed since. Heroes the plan does
-- not list come first as batch 0 (the ten it started from) or last.
function M.batches(root)
  local tools=path.join(root,'docs','character_pipeline','tools')
  V3_WAVES_AS_MODULE=true
  local built=dofile(path.join(tools,'v3_waves.lua')).build(root)
  V3_WAVES_AS_MODULE=nil
  local sources=dofile(path.join(tools,'v3_sources.lua')).heroes(root)
  local package_of,source_of={},{}
  for _,hero in ipairs(sources) do package_of[hero.source_id]=hero.package;source_of[hero.package]=hero.source_id end
  local batches={{index=0,heroes={}}}
  local placed={}
  local current
  for line in path.read(path.join(root,'docs','plans','2026-10-28-history-nobu-full-port','m6-waves.md')):gmatch('[^\r\n]+') do
    local number=line:match('^## 第 (%d+) 批')
    if number then current={index=math.tointeger(tonumber(number)),heroes={}};batches[#batches+1]=current
    elseif line:match('^## ') then current=nil
    elseif current then
      local id=line:match('^| (%u%d+) ')
      if id then
        local package=assert(package_of[id],'m6-waves.md names '..id..', which has no package')
        current.heroes[#current.heroes+1]=package;placed[package]=true
      end
    end
  end
  for _,hero in ipairs(sources) do
    if built.shipped[hero.source_id] then
      table.insert(batches[1].heroes,hero.package);placed[hero.package]=true
    end
  end
  table.sort(batches[1].heroes)
  -- D22: the skill-set variants (<hero>_old, <hero>_v20) follow, one section per version, in their
  -- base heroes' order.
  local is_package,order,n={},{},0
  for _,hero in ipairs(sources) do is_package[hero.package]=true end
  for _,batch in ipairs(batches) do for _,package in ipairs(batch.heroes) do n=n+1;order[package]=n end end
  for _,variant in ipairs(M.VARIANTS) do
    local list={}
    for _,hero in ipairs(sources) do
      local base,kind=M.base_of(hero.package,is_package)
      if kind==variant.suffix and not placed[hero.package] then list[#list+1]=hero.package;placed[hero.package]=true end
    end
    table.sort(list,function(a,b)
      local x,y=order[(M.base_of(a,is_package))] or math.huge,order[(M.base_of(b,is_package))] or math.huge
      if x~=y then return x<y end
      return a<b
    end)
    if #list>0 then batches[#batches+1]={index=#batches,heroes=list,variant=variant} end
  end
  local rest={}
  for _,hero in ipairs(sources) do if not placed[hero.package] then rest[#rest+1]=hero.package end end
  table.sort(rest)
  if #rest>0 then batches[#batches+1]={index=#batches,heroes=rest,unplanned=true} end
  return batches,source_of
end

local function latest_run(root,given)
  if given then return given end
  local dir=path.join(root,M.RUNS)
  if not path.is_directory(dir) then return nil end
  local best
  for entry in lfs.dir(dir) do
    if entry:sub(1,1)~='.' and path.is_directory(path.join(dir,entry)) and (not best or entry>best) then best=entry end
  end
  return best
end

-- One verify_m6_batch run, its output in OUTPUT.stdout.log / .stderr.log as it goes: a batch that waits
-- (memory admission before its build, 20261106 b10g1 waited 6 h on 12 GB of 16) shows why while it runs.
-- A spawned process gives no exit code here: the run passed when it wrote OUTPUT/report.json.
local function run_batch(lua,root,args,output)
  local pid=process.spawn(lua,args,{cwd=root,stdout=output..'.stdout.log',stderr=output..'.stderr.log'})
  while not process.wait(pid,60000) do end
  local stderr=path.is_file(output..'.stderr.log') and path.read(output..'.stderr.log') or ''
  return path.is_file(path.join(output,'report.json')) and 0 or 1,stderr
end

function M.sweep(root,options)
  local run=options.run or os.date('%Y%m%d-%H%M%S')
  local base=path.join(root,M.RUNS,run)
  path.mkdir_p(base)
  print('sweep run '..run)
  local built=options.no_build
  local lua=b.lib('platform').lua_executable
  for _,batch in ipairs((M.batches(root))) do
    if math.type(batch.index)=='integer' and (not options.batch or options.batch==batch.index) then
      for group=1,(#batch.heroes+M.GROUP-1)//M.GROUP do
        local name=('b%dg%d'):format(batch.index,group)
        local output=path.join(base,name)
        if path.is_file(path.join(output,'report.json')) then
          print(name..' already has its report')
        else
          -- verify_m6_batch wants a new directory: a stopped group starts over beside the old one.
          local attempt=1
          while path.exists(output) do attempt=attempt+1;output=path.join(base,name..'-'..attempt) end
          local args={path.join(root,'scripts','verify_m6_batch.lua'),'--recipes','quick,quick_swap',
            '--matches',tostring(options.matches),'--parallel',tostring(options.parallel),'--output',output}
          -- By name: `--batch` would seat v3_waves' current order, not the plan's.
          local heroes={}
          for i=(group-1)*M.GROUP+1,math.min(group*M.GROUP,#batch.heroes) do heroes[#heroes+1]=batch.heroes[i] end
          args[#args+1]='--heroes';args[#args+1]=table.concat(heroes,',')
          if built then args[#args+1]='--no-build' end
          local started=os.time()
          print(name..': '..output..'.stdout.log')
          local code,stderr=run_batch(lua,root,args,output)
          print(('%s exit %s %ds'):format(name,tostring(code),os.time()-started))
          if code~=0 then print(stderr:sub(-1500)) else built=true end
        end
      end
    end
  end
  -- The training maps have no trees: heroes with a tree-target ability also play on nobu.
  local trees=M.tree_heroes(root)
  local output=path.join(base,'nobu')
  if #trees>0 and not options.batch and not path.is_file(path.join(output,'report.json')) then
    local attempt=1
    while path.exists(output) do attempt=attempt+1;output=path.join(base,'nobu-'..attempt) end
    local args={path.join(root,'scripts','verify_m6_batch.lua'),'--recipes','nobu','--matches',tostring(options.matches),
      '--parallel',tostring(options.parallel),'--output',output,'--heroes',table.concat(trees,',')}
    if built then args[#args+1]='--no-build' end
    local started=os.time()
    print('nobu: '..output..'.stdout.log')
    local code,stderr=run_batch(lua,root,args,output)
    print(('nobu exit %s %ds'):format(tostring(code),os.time()-started))
    if code~=0 then print(stderr:sub(-1500)) end
  end
  return 0
end

-- Heroes whose package has a tree-target ability (`target = { type = "tree"`).
function M.tree_heroes(root)
  local heroes={}
  local dir=path.join(root,'docs','character_pipeline','packages')
  for hero in lfs.dir(dir) do
    local file=path.join(dir,hero,'v3.lua')
    if hero:sub(1,1)~='.' and path.is_file(file) then
      local f=assert(io.open(file,'rb'));local text=f:read('a');f:close()
      if text:find('target%s*=%s*{%s*type%s*=%s*"tree"') then heroes[#heroes+1]=hero end
    end
  end
  table.sort(heroes)
  return heroes
end

-- abilities[hero][ability]={casts,inputs,damage}, attack[hero]=damage outside any ability, from the match
-- reports of the run. Every seat counts, so a hero seen as a filler in another group adds to its row.
function M.match_evidence(root,run)
  local out={abilities={},attack={},games={},matches=0,failed={},groups=0}
  if not run then return out end
  local base=path.join(root,M.RUNS,run)
  if not path.is_directory(base) then return out end
  for group in lfs.dir(base) do
    local dir=path.join(base,group)
    if group:sub(1,1)~='.' and path.is_file(path.join(dir,'report.json')) then
      out.groups=out.groups+1
      for file in lfs.dir(dir) do
        if file:match('^quick.*%-%d+%.json$') or file:match('^nobu%-%d+%.json$') then
          local ok,report=pcall(json.read,path.join(dir,file))
          if ok and report.success then
            out.matches=out.matches+1
            for _,hero in ipairs(report.hero_stats or {}) do
              local rows=out.abilities[hero.hero] or {}
              out.abilities[hero.hero]=rows
              out.games[hero.hero]=(out.games[hero.hero] or 0)+1
              for id,cast in pairs(hero.casts or {}) do
                if id~='' then
                  local row=rows[id] or {casts=0,inputs=0,damage=0}
                  rows[id]=row
                  local traced=type(cast.results)=='table' and cast.results.accepted or 0
                  row.casts=row.casts+math.max(cast.accepted or 0,traced)
                  row.inputs=row.inputs+(cast.inputs or 0)
                  for result,count in pairs(type(cast.results)=='table' and cast.results or {}) do
                    if result:match('^rejected:') then
                      row.rejected=row.rejected or {}
                      row.rejected[result:sub(10)]=(row.rejected[result:sub(10)] or 0)+count
                    end
                  end
                end
              end
              for id,dealt in pairs(hero.ability_damage or {}) do
                local row=rows[id] or {casts=0,inputs=0,damage=0}
                rows[id]=row
                row.damage=row.damage+(dealt.damage or 0)
              end
              -- Basic attacks carry no ability tag (moba_headless DamageTrace).
              local untagged=hero.ability_damage and hero.ability_damage['(untagged)']
              out.attack[hero.hero]=(out.attack[hero.hero] or 0)+(untagged and untagged.damage or 0)
            end
          else
            out.failed[#out.failed+1]=group..'/'..file
          end
        end
      end
    end
  end
  table.sort(out.failed)
  return out
end

-- reports[package]=the generator's report for a hero, without writing anything.
function M.generated(root)
  V3_RUST_AS_MODULE=true
  local rust=dofile(path.join(root,'docs','character_pipeline','tools','v3_rust.lua'))
  V3_RUST_AS_MODULE=nil
  local reports,problems=rust.run('.','scripts',rust.heroes('.','all'),{write=false})
  local by={}
  for _,report in ipairs(reports) do by[report.package or report.hero]=report end
  return by,problems
end

local function exists(file) return lfs.attributes(file,'mode')~=nil end
local function day(file)
  local at=lfs.attributes(file,'modification')
  return at and os.date('%m-%d',at) or '?'
end

function M.tripo(root,hero,imported)
  local chars=path.join(root,'omfue','Content','Omoba','Characters')
  local clips=0
  local dir=path.join(chars,'TripoRetargetV1',hero)
  if path.is_directory(dir) then
    for entry in lfs.dir(dir) do if entry:match('%.uasset$') then clips=clips+1 end end
  end
  return {
    source=exists(path.join(root,'omfue','SourceArt','Characters','TripoV1',hero,hero..'.fbx')),
    mesh=exists(path.join(chars,'TripoV1',hero,hero..'.uasset')),
    skeleton=exists(path.join(chars,'TripoV1',hero,hero..'_Skeleton.uasset')),
    clips=clips,imported=imported[hero]==true,
  }
end

function M.video(root,hero)
  local dir=path.join(root,'target','spotlight',hero)
  local row={delivered=0}
  local mp4=path.join(dir,hero..'_spotlight.mp4')
  if exists(mp4) then row.mp4=day(mp4) end
  local report=path.join(dir,'spotlight.json')
  if exists(report) then
    local ok,data=pcall(json.read,report)
    if ok then row.casts,row.skipped,row.report=data.casts,data.skipped,day(report);row.issues=data.issues end
  end
  local delivered=path.join(root,'target','spotlight-delivered')
  if path.is_directory(delivered) then
    for entry in lfs.dir(delivered) do
      if entry:sub(1,#hero+5)==hero..'_run-' then row.delivered=row.delivered+1 end
    end
  end
  return row
end

function M.collect(root,options)
  local content=require('content_builder').new(path.join(root,'scripts','lua_data'))
  local heroes,abilities={},{}
  for _,row in ipairs(content.include('templates/heroes.lua')) do heroes[row.id]=row end
  for _,row in ipairs(content.include('templates/abilities.lua')) do abilities[row.id]=row end
  local imported=content.include('templates/moba_tripo_hero_visuals.lua').imported
  local dlls={}
  for _,row in ipairs(dofile(path.join(root,'scripts','hero_dlls.lua'))) do dlls[row.hero]=row end
  local recipe=path.read(path.join(root,'omfue','Plugins','OmRuntime','Source','OmGenerated','om_asset_recipe.json'))
  local generated,problems=M.generated(root)
  local run=latest_run(root,options.run)
  local matches=M.match_evidence(root,run)
  local spotlight=require('record_hero_spotlight')
  local batches,source_of=M.batches(root)
  local is_package={}
  for package in pairs(source_of) do is_package[package]=true end
  local out={batches={},run=run,matches=matches,problems=problems}
  for _,batch in ipairs(batches) do
    local rows={}
    for _,package in ipairs(batch.heroes) do
      local hero=heroes[package]
      local row={package=package,source=source_of[package],hero=hero,slots={},others={},abilities={}}
      -- A variant wears its base hero's Tripo body (templates/heroes.lua variant_of).
      row.base=hero and hero.variant_of or M.base_of(package,is_package)
      row.listed=hero~=nil and not hero.test_only and not hero.tombstone and hero.source_ref~=nil
      row.dll=dlls[package]~=nil
      row.ue=recipe:find(package,1,true)~=nil
      row.tripo=M.tripo(root,row.base or package,imported)
      row.video=M.video(root,package)
      row.games=matches.games[package] or 0
      row.attack=matches.attack[package]
      local blockers={}
      local report=generated[package]
      if report then for _,entry in ipairs(report.entries) do blockers[entry.id]=entry.blockers end end
      local cast=matches.abilities[package] or {}
      row.total,row.done=0,0
      -- A hero not in the content yet is counted from its document's entries (the generator's report).
      row.report=report~=nil
      local ids,kinds={},{}
      if hero then ids=hero.abilities or {}
      elseif report then
        for _,entry in ipairs(report.entries) do
          if entry.kind=='ability' or entry.kind==nil then ids[#ids+1]=entry.id end
        end
      end
      for index,id in ipairs(ids) do
        if id~='' then
          local ability=abilities[id]
          local cell={id=id,index=index,kind=ability and ability.ability_type or '?',blockers=blockers[id],evidence=cast[id]}
          row.total=row.total+1
          if cell.blockers and #cell.blockers==0 then row.done=row.done+1 end
          row.abilities[#row.abilities+1]=cell
          if index<=4 then row.slots[index]=cell else row.others[#row.others+1]=cell end
        end
      end
      local ok,script=pcall(spotlight.load_package,root,package)
      if ok then
        -- A summon's passive has no key to press: it runs while the summon is out.
        local unshown={}
        for _,u in ipairs(script.unshown or {}) do
          if u.reason~='summon passive' then unshown[#unshown+1]=u end
        end
        -- The script itself (record_hero_spotlight.script): the attacks and the death are its last segments.
        local body={}
        local okscript,full=pcall(spotlight.script,script)
        for _,slot in ipairs(okscript and full.slots or {}) do if slot.kind=='attack' or slot.kind=='death' then body[slot.kind]=true end end
        row.script={shown=#script.slots,unshown=unshown,attack=body.attack==true,death=body.death==true}
      else row.script={error=tostring(script):match('[^\n]*')} end
      rows[#rows+1]=row
    end
    out.batches[#out.batches+1]={index=batch.index,unplanned=batch.unplanned,variant=batch.variant,rows=rows}
  end
  return out
end

local function is_castable(kind) return kind=='active' or kind=='ultimate' or kind=='toggle' end

-- One ability's cell: a cast count, 被動, or 未驗證 when the run has no match with this hero.
function M.ability_cell(row,cell)
  if not cell then return '—' end
  if cell.blockers==nil then return '未驗證' end
  if #cell.blockers>0 then return '未產生' end
  if not is_castable(cell.kind) then return '被動' end
  if row.games==0 then return '未驗證' end
  local casts=cell.evidence and cell.evidence.casts or 0
  return casts>0 and tostring(casts) or '**0**'
end

function M.gaps(row)
  local gaps={}
  if not row.listed then gaps[#gaps+1]='未上架' end
  if not row.report then gaps[#gaps+1]='v3 文件無法產生 Rust（見產生器問題）' end
  if not row.dll then gaps[#gaps+1]='hero_dlls 未列' end
  if not row.ue then gaps[#gaps+1]='UE 目錄無此英雄' end
  local unbuilt,never={},{}
  for _,cell in ipairs(row.abilities) do
    if cell.blockers==nil then unbuilt[#unbuilt+1]=cell.id..'（未驗證）'
    elseif #cell.blockers>0 then unbuilt[#unbuilt+1]=cell.id
    elseif is_castable(cell.kind) and row.games>0 and not (cell.evidence and cell.evidence.casts>0) then
      -- Inputs the authority refused say why; none means the bot never tried.
      local reasons={}
      for reason,count in pairs(cell.evidence and cell.evidence.rejected or {}) do reasons[#reasons+1]=('%s %d'):format(reason,count) end
      table.sort(reasons)
      never[#never+1]=cell.id..(#reasons>0 and '（被拒 '..table.concat(reasons,'、')..'）' or '（沒有嘗試）')
    end
  end
  if #unbuilt>0 then gaps[#gaps+1]='未產生：'..table.concat(unbuilt,'、') end
  if #never>0 then gaps[#gaps+1]='bot 未施放：'..table.concat(never,'、') end
  if row.games==0 then gaps[#gaps+1]='施放未驗證'
  elseif not row.attack or row.attack<=0 then gaps[#gaps+1]='普攻傷害 0' end
  if not row.tripo.mesh then gaps[#gaps+1]='無 Tripo 模型'
  elseif not (row.tripo.skeleton and row.tripo.clips>0 and row.tripo.imported) then gaps[#gaps+1]='Tripo 綁定不全' end
  if row.script.error then gaps[#gaps+1]='演示腳本錯誤：'..row.script.error
  elseif #row.script.unshown>0 then
    local ids={}
    for _,u in ipairs(row.script.unshown) do ids[#ids+1]=u.id..'（'..u.reason..'）' end
    gaps[#gaps+1]='演示跳招：'..table.concat(ids,'、')
  end
  if not row.script.error and not (row.script.attack and row.script.death) then
    local missing={}
    if not row.script.attack then missing[#missing+1]='普攻' end; if not row.script.death then missing[#missing+1]='死亡' end
    gaps[#gaps+1]='演示缺'..table.concat(missing,'、')..'段'
  end
  if (row.video.skipped or 0)>0 then gaps[#gaps+1]=('上次錄影略過 %d 招'):format(row.video.skipped) end
  -- The report's issues: skips, a hero downed early, a channel cut short, a recording that ended early.
  local issues={}
  for _,issue in ipairs(type(row.video.issues)=='table' and row.video.issues or {}) do
    if issue.kind~='skipped' then issues[#issues+1]=tostring(issue.ability or issue.kind)..'（'..tostring(issue.reason)..'）' end
  end
  if #issues>0 then gaps[#gaps+1]='上次錄影問題：'..table.concat(issues,'、') end
  if not row.video.mp4 then gaps[#gaps+1]='無演示影片' end
  return gaps
end

function M.render(data)
  local out={}
  local function add(line) out[#out+1]=line end
  add('# 全部英雄狀態表');add('')
  add('由 `tools\\lua\\lua.exe scripts\\hero_status.lua` 從程式與實際資料產生，不要手改；重跑就更新。');add('')
  add('**怎麼讀**');add('')
  add('- **批次**：`m6-waves.md` 的順序。第 0 批是規劃開始前就有的十位。舊版與 2.0 版技能組（`<英雄>_old`、`<英雄>_v20`）排在最後，順序跟本體一樣；Tripo 一欄看的是本體的模型。')
  add('- **上架**：內容目錄的英雄列不是 `test_only`、不是墓碑，且有 `source_ref`（等於 `omoba_template_ids::hero_playable`）。')
  add('- **已產生**：這位英雄的招式裡，`v3_rust.lua` 沒有 blocker 的數量。')
  add('- **Q／W／E／R／其他**：headless 對局裡這招的實際施放次數（每個座位都算）。`被動` 無法施放，沒有驗證它的效果；`**0**` 是 bot 這幾局沒有施放，不等於放不出來；`未驗證` 是這次沒有這位英雄的對局。')
  add('- **普攻**：這幾局裡沒有招式標記的傷害（`(untagged)`），有就是 ✓。這是間接證據，沒有單獨量普攻次數。')
  add('- **Tripo**：模型是 `Content/Omoba/Characters/TripoV1/<英雄>/<英雄>.uasset`；綁定要同時有 skeleton、重定向動作、列在 `moba_tripo_hero_visuals.imported`。沒有的英雄用共用人形。`.uasset` 只查存在，沒有開啟驗證。')
  add('- **演示腳本**：`record_hero_spotlight.load_package` 排得出的段數與跳過的招。**影片**：`target/spotlight/<英雄>/` 目前有的 mp4 與日期；舊影片可能已被清理，也可能早於最新內容。')
  add('- 影片只看目前的狀態，沒有判斷內容對不對。conformance 的逐招結果目前不在表內（`target/v3-conformance` 沒有結果）。')
  add('')
  -- Counted apart: the base heroes, the skill-set variants (D22), and everyone.
  local KEYS={'total','listed','all_built','seen','never','tripo','script_full','videos','done'}
  local counts={base={},variant={},all={}}
  for _,c in pairs(counts) do for _,k in ipairs(KEYS) do c[k]=0 end end
  local gaps_of={}
  for _,batch in ipairs(data.batches) do
    for _,row in ipairs(batch.rows) do
      local hit={}
      hit.total=true
      hit.listed=row.listed
      hit.all_built=row.total>0 and row.done==row.total
      hit.videos=row.video.mp4~=nil
      hit.tripo=row.tripo.mesh and row.tripo.skeleton and row.tripo.clips>0 and row.tripo.imported
      hit.script_full=not row.script.error and #row.script.unshown==0 and row.script.attack and row.script.death
      hit.seen=row.games>0
      gaps_of[row.package]=M.gaps(row)
      hit.done=#gaps_of[row.package]==0
      local never=0
      for _,cell in ipairs(row.abilities) do
        if row.games>0 and cell.blockers and #cell.blockers==0 and is_castable(cell.kind) and not (cell.evidence and cell.evidence.casts>0) then never=never+1 end
      end
      for _,c in ipairs({counts[batch.variant and 'variant' or 'base'],counts.all}) do
        for _,k in ipairs(KEYS) do if k~='never' and hit[k] then c[k]=c[k]+1 end end
        c.never=c.never+never
      end
    end
  end
  add('## 總計');add('')
  add('「變體」是舊版與 2.0 版技能組（D22），各自是一位可選英雄。');add('')
  add('| 項目 | 本體 | 變體 | 合計 |');add('|---|---:|---:|---:|')
  local LABELS={{'total','英雄'},{'listed','上架'},{'all_built','全部招式都已產生'},{'seen','有對局施放資料'},
    {'never','這幾局 bot 沒施放的招式'},{'tripo','Tripo 模型與綁定齊全'},{'script_full','演示腳本不跳招且有普攻、死亡'},
    {'videos','目前有演示影片'},{'done','「缺什麼」是空的（完成）'}}
  for _,l in ipairs(LABELS) do add(('| %s | %d | %d | %d |'):format(l[2],counts.base[l[1]],counts.variant[l[1]],counts.all[l[1]])) end
  add('')
  local m=data.matches
  add(('對局資料：run `%s`，%d 組，%d 局完成，%d 局未完成。'):format(tostring(data.run),m.groups,m.matches,#m.failed))
  if #m.failed>0 then add('未完成：'..table.concat(m.failed,'、')) end
  if #data.problems>0 then add('');add('產生器問題：'..table.concat(data.problems,'；')) end
  add('')
  for _,batch in ipairs(data.batches) do
    add(batch.variant and ('## '..batch.variant.title) or batch.unplanned and '## 未列入批次' or ('## 第 %d 批'):format(batch.index));add('')
    add('| 編號 | 英雄 | 上架 | 已產生 | Q | W | E | R | 其他 | 普攻 | Tripo | 綁定 | 演示腳本 | 影片 | 缺什麼 |')
    add('|---|---|---|---:|---:|---:|---:|---:|---|---|---|---|---|---|---|')
    for _,row in ipairs(batch.rows) do
      local others={}
      for _,cell in ipairs(row.others) do others[#others+1]=M.ability_cell(row,cell) end
      local script=row.script.error and '錯誤' or (('%d 段'):format(row.script.shown)..(#row.script.unshown>0 and ('，跳 %d'):format(#row.script.unshown) or ''))
      local attack=row.games==0 and '未驗證' or ((row.attack or 0)>0 and '✓' or '**0**')
      local rigged=row.tripo.skeleton and row.tripo.clips>0 and row.tripo.imported
      local rig=row.tripo.mesh and (rigged and ('有（%d 動作）'):format(row.tripo.clips) or '不全') or '無'
      local gaps=gaps_of[row.package]
      add(('| %s | %s `%s` | %s | %d/%d | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s |'):format(
        tostring(row.source),row.hero and row.hero.display_name or row.package,row.package,row.listed and '是' or '**否**',row.done,row.total,
        M.ability_cell(row,row.slots[1]),M.ability_cell(row,row.slots[2]),M.ability_cell(row,row.slots[3]),M.ability_cell(row,row.slots[4]),
        #others>0 and table.concat(others,' ') or '—',attack,row.tripo.mesh and '有' or '無',rig,script,
        row.video.mp4 or (row.video.report and ('無（%s 錄過）'):format(row.video.report) or '無'),
        #gaps>0 and table.concat(gaps,'；') or '—'))
    end
    add('')
  end
  return table.concat(out,'\n')
end

function M.write(root,options)
  local data=M.collect(root,options)
  local file=path.join(root,options.out or M.OUT)
  path.write(file,M.render(data)..'\n',true)
  print('wrote '..file)
  return 0
end

function M.main(argv)
  local verb
  local options={matches=2,parallel=8}
  local i=1
  if argv[1] and argv[1]:sub(1,2)~='--' then verb=argv[1];i=2 end
  while i<=#argv do
    local key=argv[i]
    if key=='--no-build' then options.no_build=true
    elseif key=='--run' or key=='--out' then options[key:sub(3)]=assert(argv[i+1],key..' needs a value');i=i+1
    elseif key=='--matches' or key=='--parallel' or key=='--batch' then
      options[key:sub(3)]=assert(math.tointeger(tonumber(argv[i+1] or '')),key..' needs a number');i=i+1
    else error('unknown argument: '..key) end
    i=i+1
  end
  if verb=='sweep' then return M.sweep(b.root,options) end
  assert(verb==nil,'unknown command: '..tostring(verb))
  return M.write(b.root,options)
end

if arg and arg[0] and arg[0]:match('hero_status%.lua$') and (...)~='hero_status' then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
