local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
require('_bootstrap')
local check=require('ue_saved_process_identity').check
local records={{role='ue-p1',pid=123,executable='D:/UE5.8/Engine/Binaries/Win64/UnrealEditor.exe'}}
assert(check(nil,123,'ue-p1',records).stopped)
local reused=check({path='C:/Windows/System32/conhost.exe'},123,'ue-p1',records)
assert(reused.stopped and reused.pid_reused)
assert(not pcall(check,{path='d:\\ue5.8\\engine\\binaries\\win64\\unrealeditor.exe'},123,'ue-p1',records))
assert(not pcall(check,{path=''},123,'ue-p1',records))
assert(not pcall(check,{path='C:/Windows/System32/conhost.exe'},123,'ue-p1',{}))
assert(not pcall(check,nil,124,'ue-p1',records))
assert(not pcall(check,nil,123,'ue-p1',{records[1],records[1]}))
assert(check(nil,123,'ue-p1',nil).stopped)
print('saved process identity: 8 scenarios passed')
