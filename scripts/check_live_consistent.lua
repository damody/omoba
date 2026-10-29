-- Is live consistent right now (E143)? Anyone can run it at any time:
--
--   tools\lua\lua.exe scripts\check_live_consistent.lua [--ubt] [--ue-root PATH]
--
-- om_codegen and the Lua content loaders (lua_content_check.lua) over live lua_data, and cargo check of base_content, the authority and the bridge, on a mirror
-- of live (target/land/tree), so live's own outputs (OmGenerated, target/) are never written.
-- --ubt also compiles an Unreal shadow synced from live (target/land/shadow). The same checks
-- guard land_to_live.lua; this is that landing with no changes. It also reports tracked text files
-- whose working copy has other line endings than a git checkout writes (E171), in
-- target/land/eol-report.txt; a report, not a failure (land_to_live repairs the files it lands).
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

function M.main(argv,deps)
  local checks,ue_root='codegen,cargo,server,bridge',nil
  local i=1
  while i<=#argv do
    local v=argv[i]
    if v=='--ubt' then checks='codegen,cargo,server,bridge,ubt'
    elseif v=='--ue-root' then i=i+1;ue_root=assert(argv[i],'--ue-root needs a path')
    elseif v=='--help' or v=='-h' then print('Usage: tools/lua/lua.exe scripts/check_live_consistent.lua [--ubt] [--ue-root PATH]');return
    else error('unknown argument: '..v) end
    i=i+1
  end
  local report=require('land_to_live').run({changes={},checks=checks,ue_root=ue_root,dry_run=true},deps)
  print(report.passed and ('live is consistent ('..table.concat(report.checks,', ')..')')
    or ('live is NOT consistent: '..tostring(report.error)))
  local land=require('land_to_live')
  local b=require('_bootstrap')
  local d={root=(deps and deps.root) or b.root,process=(deps and deps.process) or b.lib('process'),path=(deps and deps.path) or b.lib('path')}
  local rows=land.eol_report(d)
  local lines={}
  for _,row in ipairs(rows) do lines[#lines+1]=('%s\tworktree %s, checkout %s'):format(row.path,row.worktree,row.expected) end
  local file=d.path.join(d.root,'target','land','eol-report.txt')
  d.path.mkdir_p(d.path.parent(file))
  d.path.write(file,table.concat(lines,'\n')..'\n',true)
  print(('line endings: %d tracked text file(s) differ from a checkout (list: %s)'):format(#rows,file))
  for index=1,math.min(#lines,20) do print('  '..lines[index]) end
  report.eol_mismatches=#rows
  if not report.passed then os.exit(1) end
  return report
end

if not pcall(debug.getlocal,4,1) then M.main(arg) end
return M
