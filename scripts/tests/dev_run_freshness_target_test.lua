-- E368: the script DLL stage reads the DLLs where build_ue_moba built them (one Rust target answer).
local f = assert(io.open("scripts/dev_run_freshness.lua", "rb")); local text = f:read("a"); f:close()
assert(text:find("['script-dll']=path.join(require('ue_bridge_stage').rust_target(root,'scripts','target'),profile,'base_content.dll')", 1, true),
  "stage-dll and the script-dll check read the shared Rust target")
assert(not text:find("path.join(root,'scripts','target',profile", 1, true), "no fixed scripts/target path")
-- E558: a destination another process holds open is skipped (launches use their own copy, E124);
-- any other copy failure still fails the stage.
assert(text:find("pcall(copy,source,dest)", 1, true) and text:find("find('Permission denied',1,true)", 1, true), "a locked destination is skipped")
assert(text:find("error(err,0)end;print('staged: '", 1, true), "other failures are rethrown")
print("dev_run_freshness_target_test ok")
