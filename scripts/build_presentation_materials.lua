local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs')
local options={};for i=1,#arg,2 do assert(({['--run-id']=true,['--ue-root']=true})[arg[i]]and not options[arg[i]]);options[arg[i]]=assert(arg[i+1])end
local id,engine=assert(options['--run-id']),assert(options['--ue-root']);assert(id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local dir=p.join(b.root,'.ai-collab/presentation-materials',id);assert(not p.exists(dir),'fresh evidence required');p.mkdir_p(dir)
local report={success=false,authority_games=0,render_verified=false};local baseline={}
local function snapshot(folder)
 if not p.is_directory(folder)then return end
 for name in lfs.dir(folder)do if name~='.'and name~='..'then local file=p.join(folder,name)
  if p.is_directory(file)then snapshot(file)elseif name:match('%.uasset$')then baseline[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end
 end end
end
local function unchanged(stamps)
 for file,stamp in pairs(stamps)do assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'package changed: '..file)end
end
local ok,err=xpcall(function()
 require('ue_binary_preflight').require_ready(engine,p.join(b.root,'omfue/om.uproject'))
 local plan=require('presentation_materials').plan();report.plan=plan
 snapshot(p.join(b.root,'omfue/Content/OmGenerated/Heroes/saika_magoichi/RecipeV1'))
 snapshot(p.join(b.root,'omfue/Content/Omoba/AuthoredAnimationsV2/saika_magoichi'))
 snapshot(p.join(b.root,'omfue/Content/Omoba/PresentationMaterialsV1'));report.baseline=baseline
 local function run(label,input,expect_success)
  local input_file,result_file=p.join(dir,label..'-plan.json'),p.join(dir,label..'-result.json');j.write(input_file,input,false)
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{p.join(b.root,'omfue/om.uproject'),
   '-run=OmPresentationMaterials','-plan='..input_file,'-result='..result_file,'-AllowCommandletRendering','-d3d12','-unattended','-nop4','-nosplash',
   '-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(dir,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(dir,label..'.log'),(result.stdout or'')..(result.stderr or''),false,true)
  unchanged(baseline);local value=j.read(result_file);report[label]=value
  assert(value.schema=='omoba.presentation-materials-result.v1')
  if expect_success then
   assert(result.exit_code==0 and value.success and value.error=='',value.error)
   assert(#value.materials==#plan.materials and value.imported+value.verified==#plan.materials and value.saved==value.imported)
   for i,row in ipairs(value.materials)do assert(row.object_path==plan.materials[i].object_path and row.graph_verified and row.compiled_shader)end
  else assert(result.exit_code~=0 and not value.success and #value.error>0 and value.saved==0 and value.imported==0 and value.verified==0)end
  return value
 end
 local function copy()return j.decode(j.encode(plan))end
 -- Reject malformed plans before any valid row is created.
 local cases={}
 local x=copy();x.schema='unsupported';cases[#cases+1]=x
 x=copy();x.materials[1].object_path='/Game/Foreign.M_SaikaCloth';cases[#cases+1]=x
 x=copy();x.materials[1].linear_rgb[2]=-1;cases[#cases+1]=x
 x=copy();x.materials[1].roughness=2;cases[#cases+1]=x
 x=copy();x.materials[2]=copy().materials[1];cases[#cases+1]=x
 x=copy();x.materials={};cases[#cases+1]=x
 for index,input in ipairs(cases)do run('reject-'..index,input,false)end
 report.first=run('first',plan,true)
 local packages={};for _,row in ipairs(plan.materials)do
  local package=assert(row.object_path:match('^(.-)%.[^%.]+$'));local file=p.join(b.root,'omfue/Content',package:sub(7)..'.uasset')
  packages[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end;report.packages=packages
 report.second=run('second',plan,true);assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#plan.materials);unchanged(packages)
 x=copy();x.materials[1].linear_rgb[2]=0.25;run('reject-managed-signature',x,false);unchanged(packages)
 report.negative_cases=#cases+1;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err)
if not ok then p.write(p.join(dir,'errors.md'),'# Presentation material failure\n\n```text\n'..tostring(err)..'\n```\n',false,true)end
j.write(p.join(dir,'summary.json'),report,false);assert(ok,err);print('PASS source materials, repeated zero-save, negative plans: '..dir)
