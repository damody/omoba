local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local args={};for i=1,#arg,2 do assert(({['--plan']=true,['--run-id']=true,['--ue-root']=true})[arg[i]]and not args[arg[i]]);args[arg[i]]=assert(arg[i+1])end
local id=assert(args['--run-id']);assert(id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local dir=p.join(b.root,'.ai-collab/authored-animations-native-contract',id);assert(not p.exists(dir));p.mkdir_p(dir)
local report={success=false,authority_games=0,cases={}};local ok,err=xpcall(function()
 local plan=j.read(assert(args['--plan']));local stamps={};local lfs=require('lfs')
 local function snapshot(folder,target)target=target or stamps;for name in lfs.dir(folder)do if name~='.'and name~='..'then local file=p.join(folder,name)
  if p.is_directory(file)then snapshot(file,target)elseif name:match('%.uasset$')then target[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end
 end end end
 snapshot(p.join(b.root,'omfue/Content/OmGenerated/Heroes',plan.hero,'RecipeV1'))
 snapshot(p.join(b.root,'omfue/Content/Omoba/AuthoredAnimationsV2',plan.hero))
 local cases={
  {id='unknown-schema',mutate=function(v)v.schema='omoba.authored-animation-plan.v999'end},
  {id='foreign-skeleton',mutate=function(v)v.skeleton='/Game/Foreign/Skeleton.Skeleton'end},
  {id='escaped-source',mutate=function(v)v.animations[1].source=p.join(b.root,'scripts/lua_data/templates/heroes/foreign/clip.fbx')end},
  {id='wrong-sha',mutate=function(v)v.animations[1].sha256=string.rep('0',64)end},
  {id='foreign-asset',mutate=function(v)v.animations[1].asset='/Game/Foreign/Clip_Anim'end},
  {id='duplicate-target',mutate=function(v)v.animations[2]=j.decode(j.encode(v.animations[1]))end},
  {id='empty-jobs',mutate=function(v)v.animations={}end},
 }
 cases[#cases+1]={id='managed-signature-mismatch',mutate=function(v)v.animations[1].seconds=v.animations[1].seconds+0.1 end}
 for _,case in ipairs(cases)do
  local bad=j.decode(j.encode(plan));case.mutate(bad);local input=p.join(dir,case.id..'-plan.json');local output=p.join(dir,case.id..'-result.json');j.write(input,bad,false)
  local result=r.run(p.join(assert(args['--ue-root']),'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{p.join(b.root,'omfue/om.uproject'),
   '-run=OmAuthoredAnimationImport','-plan='..input,'-result='..output,'-unattended','-nop4','-NullRHI','-NoSound','-abslog='..p.join(dir,case.id..'-engine.log')},
   {cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(dir,case.id..'.log'),(result.stdout or '')..(result.stderr or ''),false,true);local value=j.read(output)
  assert(result.exit_code~=0 and value.success==false and #value.error>0 and value.saved==0 and value.imported==0 and value.verified==0,'native bad plan did not reject before mutation')
  for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'rejection changed existing package')end
  local current={};snapshot(p.join(b.root,'omfue/Content/OmGenerated/Heroes',plan.hero,'RecipeV1'),current)
  snapshot(p.join(b.root,'omfue/Content/Omoba/AuthoredAnimationsV2',plan.hero),current)
  for file in pairs(current)do assert(stamps[file],'rejection created unexpected package: '..file)end
  report.cases[#report.cases+1]={id=case.id,exit_code=result.exit_code,error=value.error,saved=0}
 end
 report.packages=stamps;report.success=true
end,debug.traceback)
if not ok then report.error=err;p.write(p.join(dir,'errors.md'),'# Authored native contract failure\n\n```text\n'..err..'\n```\n',false)end
j.write(p.join(dir,'summary.json'),report,false);assert(ok,err);print('PASS eight actual native authored preflight rejections: '..dir)
