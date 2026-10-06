-- Build once, execute independent formal 60Hz seeds, retain every replay report.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,json=b.lib('path'),b.lib('process'),b.lib('json')
local batch,compiled=require('moba_headless_batch'),require('moba_compiled_content')
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_headless_batch.lua [--matches 100] [--seed 1] [--recipe ALL_BOT.lua] [--output NEW_DIRECTORY]')
  return
end
local options=batch.options(arg,b.lib('args'))
local output
if options.output then
  output=path.absolute(options.output,b.root)
  assert(not path.exists(output),'output must be a new directory: '..output)
  path.mkdir_p(path.parent(output))
  assert(require('lfs').mkdir(output),'cannot reserve output directory: '..output)
else
  local parent=path.mkdir_p(path.join(b.root,'omb','target','moba-headless-batches'))
  for suffix=1,1000 do
    local candidate=path.join(parent,os.time()..'-'..suffix)
    if require('lfs').mkdir(candidate) then output=candidate;break end
  end
  assert(output,'cannot reserve batch directory')
end
local report_file=path.join(output,'batch-report.json')
local recipe_file=path.join(output,'role-plan.json')
local scripts_dir=path.join(b.root,'scripts','target','release')
local executable=path.join(b.root,'omb','target','release','moba-headless.exe')
local function report_path(ordinal) return path.join(output,('match-%04d.json'):format(ordinal)) end
local env=compiled.environment()
local report=batch.execute(options,{
  prepare=function()
    require('export_moba_role_plan').export(options.recipe,recipe_file)
    for _,args in ipairs({
      {'build','--manifest-path',path.join(b.root,'scripts','Cargo.toml'),'-p','base_content',
        '--features',compiled.feature,'--release'},
      {'build','--manifest-path',path.join(b.root,'omb','Cargo.toml'),'-p','omobab','--bin','moba-headless',
        '--features',compiled.feature,'--release'},
    }) do
      local result=process.run('cargo',args,{cwd=b.root,env=env,check=false})
      io.write(result.stdout or '');io.stderr:write(result.stderr or '')
      assert(result.exit_code==0,'batch build failed: '..tostring(result.exit_code))
    end
  end,
  run=function(seed,ordinal)
    local result=process.run(executable,{'--role-plan',recipe_file,'--profile','60','--seed',tostring(seed),
      '--scripts-dir',scripts_dir,'--report',report_path(ordinal)},{cwd=b.root,env=env,check=false})
    path.write(path.join(output,('match-%04d.log'):format(ordinal)),
      (result.stdout or '')..(result.stderr or ''))
    assert(result.exit_code==0,'match '..ordinal..' failed: '..tostring(result.exit_code))
    print(('completed match %d/%d seed=%d'):format(ordinal,options.matches,seed))
    return json.read(report_path(ordinal))
  end,
  report_path=report_path,
  save=function(value)
    value.role_plan_file=recipe_file
    value.script_dll=path.join(scripts_dir,'base_content.dll')
    value.executable=executable
    json.write(report_file,value,true)
  end,
})
print('Batch report: '..report_file)
if not report.success then
  path.write(path.join(output,'errors.md'),'# Headless batch failure\n\n'..report.error..
    '\n\nDecision: retain every completed report; fail closed, no weaker profile or fixture fallback.\n')
  error(report.error,0)
end
