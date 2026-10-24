package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
r.supervise_workflow('D:/omoba/omfue/Saved/BuildTemp/buff-transfer-rust-v2.lua',arg)
-- Derive every predecessor from this cohort's own filename. A failed receipt may be retried only after both processes retire.
local t=b.lib('time')
local prefix,version=arg[0]:gsub('\\','/'):match('^(.-)rust%-v(%d+)%.lua$')
assert(prefix and tonumber(version))
for prior=1,tonumber(version)-1 do
 local file=prefix..'rust-v'..prior..'.receipt.json'
 while not p.exists(file)do t.sleep_ms(1000)end
 local receipt=j.read(file)
 assert(receipt.retired and not r.owned_alive(receipt.owner) and not r.owned_alive(receipt.monitor),'prior cohort still owns its target')
end
local directory='D:/omoba/omfue/Saved/Automation/BuffTransferRustV2'
assert(not p.exists(directory));p.mkdir_p(directory)
local cases={
 {'template',{'test','--manifest-path','omoba-template-ids/Cargo.toml','--features','runtime-lua-content','--lib'}},
 {'core',{'test','--manifest-path','omoba-core/Cargo.toml','--lib'}},
 {'codegen',{'test','--manifest-path','omfue/codegen/Cargo.toml','--lib'}},
 {'generate',{'run','--manifest-path','omfue/codegen/Cargo.toml','--bin','om_codegen','--','--content-root','scripts/lua_data','--out','omfue/Plugins/OmRuntime/Source/OmGenerated'}},
 {'base',{'test','--manifest-path','scripts/Cargo.toml','-p','base_content','--release','--lib','--','--nocapture'}},
 {'backend',{'test','--manifest-path','omb/Cargo.toml','-p','omobab','--lib'}},
 {'bridge',{'test','--manifest-path','omfue/bridge/Cargo.toml','--lib'}},
 {'client',{'test','--manifest-path','omoba-client-runtime/Cargo.toml','--lib'}},
 {'abi',{'test','--manifest-path','scripts/Cargo.toml','-p','omb-script-abi','--lib'}},
}
local result={success=true,cases={}}
for _,case in ipairs(cases)do
 print('START '..case[1]);io.stdout:flush()
 local value=r.run_owned_to_files('C:/Users/User/.cargo/bin/cargo.exe',case[2],{cwd='D:/omoba',
  stdout=directory..'/'..case[1]..'.stdout.log',stderr=directory..'/'..case[1]..'.stderr.log',ready_file=directory..'/'..case[1]..'.ready.json'})
 result.cases[#result.cases+1]={name=case[1],exit_code=value.exit_code,retired=value.retired}
 result.success=result.success and value.exit_code==0;j.write(directory..'/progress.json',result,true)
 print('FINISH '..case[1]..' exit='..value.exit_code);io.stdout:flush()
end
j.write(directory..'/summary.json',result,false);assert(result.success,'Rust acceptance failed; preserve logs')
