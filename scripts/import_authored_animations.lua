local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local options={};for i=1,#arg,2 do assert(({['--hero']=true,['--slots']=true,['--run-id']=true,['--ue-root']=true})[arg[i]] and not options[arg[i]],'invalid option');options[arg[i]]=assert(arg[i+1])end
local hero,id,engine=assert(options['--hero']),assert(options['--run-id']),assert(options['--ue-root'])
assert(hero:match('^[a-z][a-z0-9_]*$')and id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local dir=p.join(b.root,'.ai-collab/authored-animations',id);assert(not p.exists(dir),'fresh evidence required');p.mkdir_p(dir)
local report={success=false,authority_games=0,authority_ack=false};local ok,err=xpcall(function()
 local recipe=j.read(p.join(b.root,'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json'))
 local plan=require('ue_asset_recipe_plan').build(recipe);local binding,definition
 for _,row in ipairs(plan.heroes)do if row.id==hero then binding=row end end
 for _,row in ipairs(recipe.heroes)do if row.id==hero then definition=row end end
 assert(binding and binding.skeleton and definition)
 local wanted={};for slot in assert(options['--slots']):gmatch('[^,]+')do assert(slot:match('^[a-z][a-z0-9_]*$')and not wanted[slot]);wanted[slot]=true end
 assert(next(wanted),'explicit slots required')
 local jobs,seen={},{}
 for _,row in ipairs(definition.animation_sources)do if wanted[row.slot]then
  wanted[row.slot]=nil;local source=p.absolute(row.source,b.root);local metadata=require('fbx_animation_metadata').read(assert(p.read(source,true)))
  assert(#metadata.stacks==1,'explicit single authored stack required')
  local stem=assert(source:match('([^/\\]+)%.fbx$'));assert(stem:match('^[%w_]+$'))
  local asset='/Game/Omoba/AuthoredAnimationsV2/'..hero..'/'..stem..'_Anim';if not seen[asset]then
   jobs[#jobs+1]={source=source,sha256=h.sha256(source),asset=asset,seconds=metadata.stacks[1].seconds};seen[asset]=true
  end
 end end
 assert(not next(wanted) and #jobs>=1 and #jobs<=32,'unknown or excessive authored slots')
 local native={schema='omoba.authored-animation-plan.v2',hero=hero,mesh=binding.mesh..'.'..binding.mesh:match('([^/]+)$'),skeleton=binding.skeleton..'.'..binding.skeleton:match('([^/]+)$'),animations=jobs}
 local plan_file=p.join(dir,'plan.json');j.write(plan_file,native,false)
 local baseline={};local lfs=require('lfs')
 local function snapshot(folder)
  for name in lfs.dir(folder)do if name~='.'and name~='..'then local file=p.join(folder,name)
   if p.is_directory(file)then snapshot(file)elseif name:match('%.uasset$')then baseline[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end
  end end
 end
 snapshot(p.join(b.root,'omfue/Content/OmGenerated/Heroes',hero,'RecipeV1'))
 local owned_folder=p.join(b.root,'omfue/Content/Omoba/AuthoredAnimationsV2',hero)
 if p.is_directory(owned_folder)then snapshot(owned_folder)end
 report.baseline=baseline
 require('ue_binary_preflight').require_ready(engine,p.join(b.root,'omfue/om.uproject'))
 for _,pass in ipairs({'first','second'})do
  local result_file=p.join(dir,pass..'.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{p.join(b.root,'omfue/om.uproject'),'-run=OmAuthoredAnimationImport',
   '-plan='..plan_file,'-result='..result_file,'-unattended','-nop4','-NullRHI','-NoSound','-abslog='..p.join(dir,pass..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(dir,pass..'.log'),(result.stdout or '')..(result.stderr or ''),false,true)
  if p.is_file(result_file)then report[pass]=j.read(result_file)end
  for file,stamp in pairs(baseline)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'existing recipe package changed: '..file)end
  assert(result.exit_code==0,'native authored import failed: '..pass..': '..tostring(report[pass]and report[pass].error))
  assert(report[pass].success and report[pass].imported+report[pass].verified==#jobs and report[pass].saved==report[pass].imported)
  if pass=='first'then
   report.packages={};for _,job in ipairs(jobs)do local file=p.join(b.root,'omfue/Content',job.asset:sub(7)..'.uasset');report.packages[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end
  else
   assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#jobs)
   for file,stamp in pairs(report.packages)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'unchanged import rewrote owned animation')end
  end
 end
 report.success=true
end,debug.traceback)
if not ok then report.error=err;p.write(p.join(dir,'errors.md'),'# Authored animation import failure\n\n```text\n'..err..'\n```\n',false)end
j.write(p.join(dir,'summary.json'),report,false);assert(ok,err);print('PASS native authored animation import: '..dir)
