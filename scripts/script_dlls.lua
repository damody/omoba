-- The script DLL set (M3' ①): base_content plus one DLL per hero of hero_dlls.lua. Builds,
-- launches and stagings take the set from here instead of naming base_content alone, so a
-- launch never runs base_content without the heroes it no longer registers.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

-- {{module=..., file=..., hero=...?}, ...}: base_content first, then hero_dlls.lua order (the
-- host loads them in this order; abilities register in it).
function M.modules()
  -- items.dll (M7): the shop items' generated scripts (kind = item); no hero
  local modules={{module='base_content',file='base_content.dll'},{module='items',file='items.dll'}}
  for _,row in ipairs(require('hero_dlls')) do
    modules[#modules+1]={module=row.package,file=row.package..'.dll',hero=row.hero}
  end
  return modules
end

-- cargo arguments selecting every package of the set: '-p', 'base_content', '-p', 'hero_a02', ...
-- appended to ARGS (a new table when nil).
function M.cargo_packages(args)
  args=args or {}
  for _,module in ipairs(M.modules()) do args[#args+1]='-p';args[#args+1]=module.module end
  return args
end

-- Same set hash as omoba-core scripting::loader::modules_hash: SHA-256 of `module=sha256\n`.
function M.set_text(rows)
  local lines={}
  for _,row in ipairs(rows) do lines[#lines+1]=row.module..'='..row.sha256..'\n' end
  return table.concat(lines)
end

-- PREFIX (a cargo argument list) followed by every package of the set, FEATURE (optional)
-- enabled on each as package/feature, then SUFFIX. Returns a new table.
function M.cargo_args(prefix,feature,suffix)
  local args={}
  for _,value in ipairs(prefix or {}) do args[#args+1]=value end
  M.cargo_packages(args)
  if feature then
    local features={}
    for _,module in ipairs(M.modules()) do features[#features+1]=module.module..'/'..feature end
    args[#args+1]='--features';args[#args+1]=table.concat(features,',')
  end
  for _,value in ipairs(suffix or {}) do args[#args+1]=value end
  return args
end

return M
