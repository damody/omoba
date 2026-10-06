-- Shared authoring/configuration preflight; Rust owns validation and consent.
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}
function M.inspect(options)
  local value=require('moba_role_launch').load_recipe(options)
  local humans={}
  for _,p in ipairs(assert(value.players,'selection recipe requires players')) do
    if p.bot==false then humans[#humans+1]=p.player_id end
  end
  assert(#humans>=1 and #humans<=10,'selection requires 1..10 human seats')
  table.sort(humans)
  local exe=require('moba_config_command').resolve(options)
  return {value=value,humans=humans,exe=exe}
end
function M.prepare(options,process,output,inspected)
  assert(not path.exists(output),'selection output must be a new directory: '..output)
  path.mkdir_p(output)
  local candidate=path.join(output,'candidate.json')
  path.write(candidate,json.encode(inspected.value))
  -- Validate a copy, never treat the trusted preparation as remote consent.
  local preflight=json.decode(require('moba_config_command').run(options,{'--lock-plan',candidate},process).stdout)
  assert(preflight.scope=='host-prepared-selection','invalid selection preflight')
  return candidate,preflight
end
return M
