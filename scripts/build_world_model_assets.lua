local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs')
local opts={};for k=1,#arg,2 do assert(({['--run-id']=true,['--ue-root']=true})[arg[k]]and not opts[arg[k]]);opts[arg[k]]=assert(arg[k+1])end
local id,engine=assert(opts['--run-id']),assert(opts['--ue-root']);assert(id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local dir=p.join(b.root,'.ai-collab/world-models',id);assert(not p.exists(dir));p.mkdir_p(dir)
local owned=p.join(b.root,'omfue/Content/Omoba/WorldModelsV1');local report={success=false,authority_games=0,render_verified=false}
local function snapshot()
 local out={};if p.is_directory(owned)then for name in lfs.dir(owned)do if name:match('%.uasset$')then local file=p.join(owned,name);out[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end end end
 return out
end
local function unchanged(old)assert(j.encode(snapshot())==j.encode(old),'world model packages changed unexpectedly')end
local ok,err=xpcall(function()
 require('ue_binary_preflight').require_ready(engine,p.join(b.root,'omfue/om.uproject'))
 local plan=require('world_model_assets').plan();report.plan=plan;report.baseline=snapshot()
 local function run(label,input,expect)
  local pf,rf=p.join(dir,label..'-plan.json'),p.join(dir,label..'-result.json');j.write(pf,input,false)
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{p.join(b.root,'omfue/om.uproject'),'-run=OmWorldModelAssets',
   '-plan='..pf,'-result='..rf,'-AllowCommandletRendering','-d3d12','-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(dir,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(dir,label..'.log'),(result.stdout or'')..(result.stderr or''),false,true)
  local value=j.decode(p.read(rf,true):gsub('^\239\187\191',''))
  if expect then assert(result.exit_code==0,label..' failed with exit '..tostring(result.exit_code)..': '..tostring(value.error));require('world_model_assets').verify_result(input,value)
  else assert(result.exit_code~=0 and value.success==false and value.saved==0 and value.imported==0)end
  return value
 end
 local function copy()return j.decode(j.encode(plan))end
 local cases={};local x=copy();x.schema='bad';cases[#cases+1]=x
 x=copy();x.models[1].id='../escape';cases[#cases+1]=x
 x=copy();x.models[1].source_sha256=string.rep('0',64);cases[#cases+1]=x
 x=copy();x.models[1].mtl_sha256=string.rep('0',64);cases[#cases+1]=x
 x=copy();x.models[1].texture_sha256=string.rep('0',64);cases[#cases+1]=x
 x=copy();x.models[1].source=x.models[2].source;cases[#cases+1]=x
 x=copy();x.models[2]=x.models[1];cases[#cases+1]=x
 x=copy();x.models={};cases[#cases+1]=x
 for k,input in ipairs(cases)do run('reject-'..k,input,false);unchanged(report.baseline)end
 report.first=run('first',plan,true);report.packages=snapshot()
 report.second=run('second',plan,true);assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#plan.models);unchanged(report.packages)
 assert(j.encode(require('world_model_assets').plan())==j.encode(plan),'source changed during import')
 report.negative_cases=#cases;report.success=true
end,debug.traceback)
report.error=ok and''or err;if not ok then p.write(p.join(dir,'errors.md'),'# World model import failure\n\n```text\n'..err..'\n```\n',false,true)end
j.write(p.join(dir,'summary.json'),report,false);assert(ok,err);print('PASS bounded original world models: '..dir)
