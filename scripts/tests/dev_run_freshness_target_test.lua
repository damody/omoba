-- E368: the script DLL stage reads the DLLs where build_ue_moba built them (one Rust target answer).
local f = assert(io.open("scripts/dev_run_freshness.lua", "rb")); local text = f:read("a"); f:close()
assert(text:find("['script-dll']=path.join(require('ue_bridge_stage').rust_target(root,'scripts','target'),profile,'base_content.dll')", 1, true),
  "stage-dll and the script-dll check read the shared Rust target")
assert(not text:find("path.join(root,'scripts','target',profile", 1, true), "no fixed scripts/target path")
print("dev_run_freshness_target_test ok")
