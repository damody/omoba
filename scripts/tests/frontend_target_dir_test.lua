-- E333: the UE frontend build stages the bridge from omfue/bridge/target/debug and base_content from
-- scripts/target/debug, so its cargo builds name those directories whatever CARGO_TARGET_DIR says.
local function read(p) local f=assert(io.open(p,'rb')); local s=f:read('a'); f:close(); return s end
local bridge=read('scripts/build_ue_bridge.lua')
assert(bridge:find("'bridge/Cargo.toml'),'--target-dir',p.join(ue,'bridge/target')",1,true),'the bridge build names its target directory')
local moba=read('scripts/build_ue_moba.lua')
assert(moba:find("path.join(root, 'scripts', 'Cargo.toml'), '--target-dir', path.join(root, 'scripts', 'target')",1,true),
  'the script DLL build names its target directory')
assert(read('scripts/moba_stage_contract.lua'):find("omfue/bridge/target/debug/om_bridge.dll",1,true),'the stage reads the bridge from there')
print('frontend_target_dir_test: 3 checks passed')
