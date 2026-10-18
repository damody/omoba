-- Entry routing only. Match preparation, builds and process ownership stay in their maintained workflows.
local M={}

function M.plan(arguments)
  assert(type(arguments)=='table','arguments must be a table')
  local mode,offset='td',1
  if arguments[1]=='--moba' then mode,offset='moba',2
  elseif arguments[1]=='--td' then offset=2 end
  local forwarded={}
  for index=offset,#arguments do
    local value=arguments[index]
    assert(type(value)=='string','arguments must contain strings')
    assert(value~='--moba' and value~='--td','choose --moba or --td once, as the first argument')
    forwarded[#forwarded+1]=value
  end
  local help=mode=='td' and #forwarded==1 and forwarded[1]=='--help'
  -- A plain MOBA launch opens the real hero selection. Advanced arguments retain role-launch semantics.
  if mode=='moba' and #forwarded==0 then forwarded[1]='--interactive-selection' end
  return {mode=mode,args=forwarded,help=help}
end

function M.help()
  return table.concat({
    'Usage: run_ue.bat --moba [role-launch options]',
    '       run_ue.bat [--td] [TD options]',
    '',
    'MOBA: run_ue.bat --moba',
    '  Opens the maintained single-human/Bot recipe and interactive hero selection.',
    '  Uses fresh match output, the compiled authority and the Unreal frontend.',
    '  Advanced example: run_ue.bat --moba --interactive-selection --no-build --ue-root D:/UE5.8',
    '  All role-launch options: run_ue.bat --moba --help',
    '',
    'TD: existing arguments and the no-argument TD launch are preserved.',
    '  --editor --headless-smoke --game-smoke --build-only --build --no-build',
    '  --safe --dx12 --d3d11 --networked --with-backend --single-player --seconds N',
    '',
    'Help does not build, open Unreal, allocate a match or start a backend.',
  },'\n')
end

return M
