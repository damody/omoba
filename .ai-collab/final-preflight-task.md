# Bounded Grok implementation proposal

Return a minimal unified diff ONLY; no tool calls or file writes. Codex integrates and tests independently. No commits, pushes, resets, clean, restore, branches, credentials, installation, engine edits, simulation, or external operations. Preserve all current changes. Do not broaden architecture or claim tests ran.

Outcome: fix the current final verification entry's reuse-built path so binary readiness is checked before the live launch command. Existing staged-only build verifies only DLL SHA, not UnrealEditor module BuildIds. A concurrent OTHER project changed engine BuildId; the live launcher properly refused. Never rewrite IDs, change thresholds, force success, or automatically rebuild reused artifacts.

Target scripts/verify_moba_local_baseline.lua. Current exact relevant code:
```lua
local ok,err=xpcall(function()
  local editor,ue_root=require('moba_role_launch').editor({})
  command('frontend-build',platform.lua_executable,{path.join(b.root,'scripts/build_ue_moba.lua'),
    arg[2]=='--reuse-built' and '--verify-staged-only' or '--build-only','--ue-root',ue_root})
  if arg[2]~='--reuse-built' then
  for _,build in ipairs({
```

Existing shared pure readiness API:
```lua
require('ue_binary_preflight').require_ready(ue_root,path.join(b.root,'omfue','om.uproject'))
```
It checks project/plugin module manifests + binary presence against engine ID without writing anything; existing gameplay and selection use it. Insert it AFTER frontend build command (full build must be allowed to repair an OLD project binary), BEFORE Rust builds and actual game. Keep final launcher's own preflight, which protects against later concurrent changes. Explain exact placement and scope, not ABI/full-game proof. No new helper, no stage changes, no new runtime dependencies, no fallback. Diff plus 3 concise review assertions sufficient.
