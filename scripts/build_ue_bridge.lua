-- Fixed clone-local Lua generation/build/staging. No shell or PowerShell path.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,h=b.lib('path'),b.lib('process'),b.lib('hash')
local root=b.root;local ue=p.join(root,'omfue');local plugin=p.join(ue,'Plugins/OmRuntime')
require('ue_build_prerequisites').require_ready(r)
local function run(label,exe,args)
 print('[om-bridge] '..label);io.stdout:flush()
 local result=r.run(exe,args,{cwd=root,check=false});if result.stdout and result.stdout~=''then print(result.stdout)end
 if result.stderr and result.stderr~=''then io.stderr:write(result.stderr);io.stderr:flush()end
 assert(result.exit_code==0,label..' failed with exit '..tostring(result.exit_code))
end
local function stage(src,dst)
 assert(p.is_file(src),'missing current build artifact: '..src);local sha=h.sha256(src)
 if p.is_file(dst) and h.sha256(dst)==sha then print('[om-bridge] unchanged '..dst);return end
 p.mkdir_p(p.parent(dst));p.write(dst,p.read(src,true),true,true)
 assert(h.sha256(dst)==sha,'staging SHA mismatch: '..dst);print('[om-bridge] staged '..dst..' '..sha)
end
local lua=b.lib('platform').lua_executable
run('animation import prerequisite',lua,{p.join(root,'scripts/ensure_bpgu_animation_import.lua')})
run('content codegen','cargo',{'run','--manifest-path',p.join(ue,'codegen/Cargo.toml'),'--','--content-root',p.join(root,'scripts/lua_data'),'--out',p.join(plugin,'Source/OmGenerated')})
run('synchronize authored HUD labels',lua,{p.join(root,'scripts/sync_ue_hud_manifest_labels.lua')})
run('Rust bridge','cargo',{'build','--manifest-path',p.join(ue,'bridge/Cargo.toml')})
local temp=p.join(ue,'Saved/BuildBridge/om_bridge.h');p.mkdir_p(p.parent(temp))
run('C ABI header','cbindgen',{'--config',p.join(ue,'bridge/cbindgen.toml'),'--crate','om_bridge','--output',temp,p.join(ue,'bridge')})
stage(temp,p.join(plugin,'Source/ThirdParty/OmBridge/include/om_bridge.h'))
stage(p.join(ue,'bridge/target/debug/om_bridge.dll.lib'),p.join(plugin,'Source/ThirdParty/OmBridge/lib/Win64/om_bridge.lib'))
stage(p.join(ue,'bridge/target/debug/om_bridge.dll'),p.join(plugin,'Binaries/Win64/om_bridge.dll'))
-- The parent build explicitly produces this current debug DLL before invoking
-- this entry. Never pick an older release or convenience copy as a fallback.
stage(p.join(root,'scripts/target/debug/base_content.dll'),p.join(plugin,'Binaries/Win64/base_content.dll'))
print('[om-bridge] fixed Lua bridge generation/staging complete')
