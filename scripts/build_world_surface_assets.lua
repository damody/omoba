local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process');local lfs=require('lfs')
local options={};for i=1,#arg,2 do assert(({['--run-id']=true,['--ue-root']=true,['--projection']=true})[arg[i]]and not options[arg[i]]);options[arg[i]]=assert(arg[i+1])end
local id,engine=assert(options['--run-id']),assert(options['--ue-root']);assert(id:match('^[%w_-]+$'));r.supervise_workflow(script,arg)
local dir=p.join(b.root,'.ai-collab/world-surfaces',id);assert(not p.exists(dir),'fresh evidence required');p.mkdir_p(dir)
local owned=p.join(b.root,'omfue/Content/Omoba/WorldSurfacesV1');local report={success=false,authority_games=0,render_verified=false}
local function snapshot()
 local values={};if not p.is_directory(owned)then return values end
 for name in lfs.dir(owned)do if name:match('%.uasset$')then local file=p.join(owned,name);values[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}end end
 return values
end
local function unchanged(values)
 local current=snapshot();assert(j.encode(values)==j.encode(current),'world surface packages changed unexpectedly')
end
local ok,err=xpcall(function()
 require('ue_binary_preflight').require_ready(engine,p.join(b.root,'omfue/om.uproject'))
 local projection=options['--projection']or'world_xy_v1';local plan=require('world_surface_assets').plan(projection);report.projection=projection;report.plan=plan;report.baseline=snapshot()
 local function run(label,input,expect_success)
  local input_file,result_file=p.join(dir,label..'-plan.json'),p.join(dir,label..'-result.json');j.write(input_file,input,false)
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{p.join(b.root,'omfue/om.uproject'),'-run=OmWorldSurfaceAssets',
   '-plan='..input_file,'-result='..result_file,'-AllowCommandletRendering','-d3d12','-unattended','-nop4','-nosplash',
   '-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(dir,label..'-engine.log')},{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(dir,label..'.log'),(result.stdout or'')..(result.stderr or''),false,true)
  local value=j.read(result_file);report[label]=value;assert(value.schema=='omoba.world-surfaces-result.v1')
  if expect_success then
   assert(result.exit_code==0 and value.success and value.error=='',value.error)
   assert(#value.surfaces==#plan.surfaces and value.imported+value.verified==2*#plan.surfaces and value.saved==value.imported)
   for i,row in ipairs(value.surfaces)do local wanted=plan.surfaces[i]
    assert(row.id==wanted.id and row.texture==wanted.texture and row.material==wanted.material and row.graph_verified and row.compiled_shader and row.srgb and row.mipmaps)
    assert(row.world_xy_uv==(projection~='world_triplanar_v1')and row.world_triplanar_uv==(projection=='world_triplanar_v1'))
   end
  else assert(result.exit_code~=0 and not value.success and #value.error>0 and value.saved==0 and value.imported==0 and value.verified==0)end
  return value
 end
 local function copy()return j.decode(j.encode(plan))end
 local n=#plan.surfaces;local cases={};local x=copy();x.schema='unsupported';cases[#cases+1]=x
 x=copy();x.surfaces[n].source='D:/foreign.png';cases[#cases+1]=x
 x=copy();x.surfaces[n].sha256=string.rep('0',64);cases[#cases+1]=x
 x=copy();x.surfaces[n].width=1024;cases[#cases+1]=x
 x=copy();x.surfaces[n].material='/Game/Foreign.Material';cases[#cases+1]=x
 x=copy();x.surfaces[n].repeat_cm=0;cases[#cases+1]=x
 x=copy();x.surfaces[2]=x.surfaces[1];cases[#cases+1]=x
 x=copy();x.surfaces={};cases[#cases+1]=x
 x=copy();x.surfaces[n].projection='unsupported';cases[#cases+1]=x
 for index,input in ipairs(cases)do run('reject-'..index,input,false);unchanged(report.baseline)end
 report.first=run('first',plan,true);report.packages=snapshot()
 for file,stamp in pairs(report.baseline)do assert(j.encode(report.packages[file])==j.encode(stamp),'existing source surface changed')end
 report.second=run('second',plan,true);assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==2*n);unchanged(report.packages)
 x=copy();x.surfaces[n].repeat_cm=8000;run('reject-managed-signature',x,false);unchanged(report.packages)
 assert(j.encode(require('world_surface_assets').plan(projection))==j.encode(plan),'source changed during import')
 report.negative_cases=#cases+1;report.success=true
end,debug.traceback)
report.error=ok and''or tostring(err);if not ok then p.write(p.join(dir,'errors.md'),'# World surface failure\n\n```text\n'..tostring(err)..'\n```\n',false,true)end
j.write(p.join(dir,'summary.json'),report,false);assert(ok,err);print('PASS world surfaces, repeated zero-save and negative plans: '..dir)
