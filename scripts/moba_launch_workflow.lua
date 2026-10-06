-- Ordered workflow; callbacks perform actual builds and own process cleanup.
local M={}
function M.execute(options,steps)
  if options.prepare_only then return steps.prepare() end
  if options.server_only then
    if not options.no_build then steps.build_server() end
    steps.verify_server()
    local plan=steps.prepare()
    assert(plan.mode=='dedicated-server' and #plan.clients==0 and plan.server,
      'server-only workflow requires a dedicated authority plan')
    steps.verify_server()
    steps.launch(plan)
    return plan
  end
  steps.resolve_editor()
  if not options.no_build then
    steps.build_frontend()
    steps.build_runtime() -- Includes selection CLI, server and client runtime.
  end
  steps.verify_stage() -- Fail before the user spends time selecting a hero.
  if options.interactive_selection then steps.select() end
  local plan=steps.prepare()
  assert(plan.human_count>0,'interactive launch requires a human; use headless for all-Bot recipes')
  -- Detect a copy changed while the selection renderer was open.
  steps.verify_stage()
  steps.launch(plan)
  return plan
end
return M
