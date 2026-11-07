-- Entry routing only. Match preparation, builds and process ownership stay in their maintained workflows.
local M={}

-- Flags that only the legacy TD launcher understands; a launch that starts with one of them stays on TD.
local TD_ONLY={['--editor']=true,['--headless-smoke']=true,['--game-smoke']=true,['--build-only']=true,['--build']=true,
  ['--safe']=true,['--dx12']=true,['--d3d11']=true,['--networked']=true,['--with-backend']=true,['--single-player']=true,
  ['--seconds']=true}

-- Options that already choose how the match is staffed; the implicit demo route leaves them alone.
local STAFFING={['--interactive-selection']=true,['--server-only']=true,['--connect']=true,['--hero']=true,['--prepare-only']=true,['--help']=true}

function M.plan(arguments)
  assert(type(arguments)=='table','arguments must be a table')
  local mode,offset='moba',1
  if arguments[1]=='--moba' then offset=2
  elseif arguments[1]=='--td' or TD_ONLY[arguments[1]] then mode='td';offset=arguments[1]=='--td' and 2 or 1 end
  local forwarded={}
  for index=offset,#arguments do
    local value=arguments[index]
    assert(type(value)=='string','arguments must contain strings')
    assert(value~='--moba' and value~='--td','choose --moba or --td once, as the first argument')
    forwarded[#forwarded+1]=value
  end
  local help=#arguments==1 and arguments[1]=='--help' or (mode=='td' and #forwarded==1 and forwarded[1]=='--help')
  -- A plain launch is the demo: one human picks a hero, nine Bots fill the other seats. Advanced arguments retain role-launch semantics.
  if mode=='moba' and (#forwarded==0 or offset==1) then
    local staffed=false
    for _,value in ipairs(forwarded)do if STAFFING[value] then staffed=true end end
    if not staffed then table.insert(forwarded,1,'--interactive-selection') end
  end
  return {mode=mode,args=forwarded,help=help}
end

function M.help()
  return table.concat({
    'Usage: run_ue.bat [--moba] [role-launch options]',
    '       run_ue.bat --td [TD options]',
    '',
    'Demo: run_ue.bat',
    '  One human and nine Bots: pick a hero in Unreal, then play a full MOBA match.',
    '  Same as run_ue.bat --moba. Uses fresh match output, the compiled authority and the Unreal frontend.',
    '  Skip rebuilding: run_ue.bat --no-build',
    '  Advanced example: run_ue.bat --moba --interactive-selection --no-build --ue-root D:/UE5.8',
    '  All role-launch options: run_ue.bat --moba --help',
    '',
    'TD: run_ue.bat --td [options]. Legacy TD flags as the first argument also select TD:',
    '  --editor --headless-smoke --game-smoke --build-only --build',
    '  --safe --dx12 --d3d11 --networked --with-backend --single-player --seconds N',
    '',
    'Help does not build, open Unreal, allocate a match or start a backend.',
  },'\n')
end

return M
