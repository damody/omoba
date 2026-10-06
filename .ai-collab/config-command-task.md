Implement bounded new Lua module scripts/moba_config_command.lua and new scripts/tests/moba_config_command_test.lua ONLY in D:/code/omoba. Primary integrates existing launcher/docs. Preserve all dirty changes, no commits/push/reset/cleanup/install/credential/external changes. Use apply_patch. Fixed Lua tests only, no games/sims/Unreal/Cargo builds. No need repository exploration: complete requirements and source context below.

Module bootstrap pattern:
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local M={}
return M

Implement M.resolve(options,dependencies): options.profile defaults release; strict debug/release only; path.join(b.root,'omb','target',profile,'moba-config.exe'); dependencies defaults {is_file=path.is_file}; assert file exists with actionable message exact missing path and cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only plus --release if selected. Never fallback profile or call cargo or resolve Unreal. Return exe string.
Implement M.run(options,args,process,dependencies): resolve above, shallow copy args to prevent mutation by process; process.run(exe,args_copy,{cwd=b.root,env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},check=false,label='compiled role configuration'}); fail closed if result.exit_code not exactly0, include code/stdout/stderr diagnostic (no false success or JSON decode before failure); return result.
Tests selfcontained bootstrap package.path scripts parent and require module. Inject is_file/process.run; table debug/release/default/profileinvalid/missing with no fallback for no_build/prepare_only/server_only/connect too; exact executable and compiled-only env; caller args unchanged when fake mutates args; nonzero code, nil code failure, stdout/stderr diagnostics. No actual executable spawn needed. Test command D:/code/omoba/tools/lua/lua.exe scripts/tests/moba_config_command_test.lua. Avoid reading nonexistent files or inspecting unrelated repo/global MCP/auth. Return changed files and actual validation. If tools stall, return complete unified patch in final text instead of waiting indefinitely; don't claim applied unless actually applied.
