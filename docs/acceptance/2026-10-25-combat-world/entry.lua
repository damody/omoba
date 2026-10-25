package.path='scripts/?.lua;'..package.path
local p,j,h,r=require('tools.lua.lib.path'),require('tools.lua.lib.json'),require('tools.lua.lib.hash'),require('tools.lua.lib.process')
r.supervise_workflow('D:/omoba/omfue/Saved/BuildTemp/combat-world-accept-v1.lua',arg)
local dir='omfue/Saved/Automation/CombatWorldV1';assert(not p.exists(dir));p.mkdir_p(dir)
local function run(name,exe,args)
 print('START '..name);io.stdout:flush();local v=r.run_owned_to_files(exe,args,{cwd='D:/omoba',stdout=dir..'/'..name..'.stdout.log',stderr=dir..'/'..name..'.stderr.log',ready_file=dir..'/'..name..'.ready.json'})
 assert(v.exit_code==0 and v.retired,name..' failed; retain all original output');print('FINISH '..name);io.stdout:flush();return v
end
local inputs={};for _,row in ipairs(j.read('omfue/Saved/BuildTemp/combat-world-parent-v1.json').files)do local copy=dir..'/inputs/'..row.file;p.mkdir_p(p.parent(copy));p.write(copy,p.read(row.file,true),false,true);inputs[#inputs+1]={file=row.file,copy=copy,sha256=h.sha256(row.file)}end
j.write(dir..'/inputs.json',{files=inputs},false)
local lua='D:/omoba/tools/lua/lua.exe'
run('source-before',lua,{'omfue/Saved/BuildTemp/summoner-shield-source-guard.lua','capture',dir..'/source-before.json'})
run('ue',lua,{'scripts/build_ue_moba.lua','--ue-root','D:/UE5.8','--build-executor','local','--disable-action-cache','--prefetch-shared-pch','--max-parallel-actions','20','--compiler-version','Latest'})
local stage=require('moba_stage_contract').verify(require('moba_stage_contract').bindings('D:/omoba'))
j.write(dir..'/staged.json',{success=true,stage=stage},false)
local tests={'Om.Runtime.PublicHealthTextLifecycle','Om.Runtime.NativeCombatVfxLifecycle','Om.Runtime.ShieldNameplatePresentation'}
local native=dir..'/native';p.mkdir_p(native)
p.write(native..'/commands.txt',require('ue_local_acceptance_contract').native_command_file(tests,'D:/omoba/'..native..'/report','D:/omoba/'..native..'/engine.log'),false)
run('native','D:/UE5.8/Engine/Binaries/Win64/UnrealEditor-Cmd.exe',{'D:/omoba/omfue/om.uproject','-unattended','-nop4','-NoSound','-NoLiveCoding','-d3d12','-TestExit=Automation Test Queue Empty','-CmdLineFile=D:/omoba/'..native..'/commands.txt','-om-health-text-capture=D:/omoba/'..native..'/capture'})
local result=require('ue_local_acceptance_contract').native(j.decode((p.read(native..'/report/index.json'):gsub('^\239\187\191',''))),tests)
assert(result.count==3 and result.warnings==0);result.success=true;j.write(native..'/summary.json',result,false)
run('source-verified',lua,{'omfue/Saved/BuildTemp/summoner-shield-source-guard.lua','check',dir..'/source-before.json',dir..'/source-verified.json'})
for _,row in ipairs(inputs)do assert(h.sha256(row.file)==row.sha256 and h.sha256(row.copy)==row.sha256)end
j.write(dir..'/summary.json',{success=true,native=result,rust_fresh=false,backend_fresh=false,catalog_changed=false,network_match_created=false},false)
