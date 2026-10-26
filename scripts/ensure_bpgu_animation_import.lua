-- BpGeneratorUltimate 子模組未初始化時，將必要的動畫匯入修正以可重跑方式套用。
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')

local located = require('bpgu_plugin').locate(bootstrap.root, path)
local plugin, version = located.dir, located.version
assert(located.engine == '5.8.0', 'BpGeneratorUltimate ' .. version .. ' is not built for UE 5.8')
-- An installed (precompiled) package is never rebuilt by UBT, so a source patch would
-- have no effect: record it and leave the plugin untouched.
if located.installed then
  assert(path.is_file(path.join(plugin, 'Binaries', 'Win64', 'UnrealEditor-UECPTools.dll')),
    'installed BpGeneratorUltimate ' .. version .. ' has no UECPTools binary; cannot verify animation import support')
  print('[moba-ue] recognized precompiled BpGeneratorUltimate ' .. version .. ' for UE 5.8; source patch not applied, animation import requires MCP verification')
  return
end
local target = path.join(plugin, 'Source', 'UECPTools', 'Private', 'Tools', 'ImportTools.cpp')
assert(path.is_file(target), 'BpGeneratorUltimate ' .. version .. ' source plugin has no ImportTools.cpp')
local source = path.read(target)
local marker_a = 'Factory->SetDetectImportTypeOnImport(false);'
local marker_b = 'Animation import produced %s instead of AnimSequence'
local has_a = source:find(marker_a, 1, true) ~= nil
local has_b = source:find(marker_b, 1, true) ~= nil
assert(has_a == has_b, 'BpGeneratorUltimate animation import is only partially patched')
if has_a then
  print('[moba-ue] BpGeneratorUltimate animation import fix is present')
  return
end

local nl = source:find('\r\n', 1, true) and '\r\n' or '\n'
local function replace_once(text, before, after)
  local first = assert(text:find(before, 1, true), 'unknown BpGeneratorUltimate import source layout')
  assert(not text:find(before, first + #before, true), 'ambiguous BpGeneratorUltimate import source layout')
  return text:sub(1, first - 1) .. after .. text:sub(first + #before)
end

local factory_old = '\tUFbxFactory* Factory = NewObject<UFbxFactory>();' .. nl ..
  '\tFactory->ImportUI->bImportMesh = false;'
local factory_new = '\tUFbxFactory* Factory = NewObject<UFbxFactory>();' .. nl ..
  '\tFactory->SetDetectImportTypeOnImport(false);' .. nl ..
  '\tFactory->ImportUI->bAutomatedImportShouldDetectType = false;' .. nl ..
  '\tFactory->ImportUI->bImportMesh = false;'
source = replace_once(source, factory_old, factory_new)

local result_old = '\tif (!Imported) return;' .. nl .. nl ..
  '\tOutJsonString = MakeSuccessJson(Imported, TEXT("AnimSequence"));'
local result_new = '\tif (!Imported) return;' .. nl ..
  '\tif (!Cast<UAnimSequence>(Imported))' .. nl ..
  '\t{' .. nl ..
  '\t\tOutError = FString::Printf(TEXT("Animation import produced %s instead of AnimSequence at \'%s\'"),' .. nl ..
  '\t\t\t*Imported->GetClass()->GetName(), *Imported->GetPathName());' .. nl ..
  '\t\treturn;' .. nl ..
  '\t}' .. nl .. nl ..
  '\tOutJsonString = MakeSuccessJson(Imported, TEXT("AnimSequence"));'
source = replace_once(source, result_old, result_new)
path.write(target, source, true)
print('[moba-ue] patched BpGeneratorUltimate animation import')
