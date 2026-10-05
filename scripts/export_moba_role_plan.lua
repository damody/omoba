-- Export a Lua recipe into the strict Rust-owned match-plan schema.
local source = debug.getinfo(1,'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path,json = bootstrap.lib('path'),bootstrap.lib('json')
local M = {}
local function recipe(input,output)
  input = path.absolute(input,bootstrap.root)
  output = path.absolute(output,bootstrap.root)
  assert(input:lower() ~= output:lower(),'refusing to overwrite the Lua recipe')
  local value = assert(loadfile(input))()
  assert(type(value)=='table','role recipe must return a table')
  return value,output
end
function M.export(input,output)
  local value
  value,output=recipe(input,output)
  -- Authoring executes Lua; strict schema/catalog/control checks remain Rust's
  -- responsibility before any World is constructed.
  path.write(output,json.encode(value),true)
  return output
end
function M.server_fragment(input,output)
  local value
  value,output=recipe(input,output)
  path.write(output,M.server_fields(value),true)
  return output
end
function M.server_fields(value)
  local humans={}
  for _,player in ipairs(assert(value.players,'role recipe requires players')) do
    if player.bot==false then humans[#humans+1]=player end
  end
  table.sort(humans,function(a,b) return a.player_id<b.player_id end)
  local bindings={}
  for _,player in ipairs(humans) do
    assert(math.type(player.player_id)=='integer' and math.type(player.team_id)=='integer','human IDs must be integers')
    bindings[#bindings+1]=('%s=%d'):format(json.encode(tostring(player.player_id)),player.team_id)
  end
  -- A replacement [server] field fragment, not blindly appended to game.toml.
  -- JSON quoting is also valid TOML basic-string quoting for this encoder.
  return table.concat({
    'MATCH_GAMEPLAY_MODE="three_lane"',
    'MATCH_LOCKSTEP_MODE="secure_v2_required"',
    'MATCH_MAP_ID='..json.encode(assert(value.map_id,'role recipe requires map_id')),
    'MATCH_ROLE_PLAN_JSON='..json.encode(json.encode(value)),
    'AUTHENTICATED_TEAM_BINDINGS={'..table.concat(bindings,',')..'}',
    'AUTHENTICATED_HERO_BINDINGS={}',
  },'\n')..'\n'
end
if ... == 'export_moba_role_plan' then return M end
assert(#arg==2 or (#arg==3 and arg[3]=='--server-fragment'),
  'usage: export_moba_role_plan.lua INPUT.lua OUTPUT [--server-fragment]')
print(#arg==3 and M.server_fragment(arg[1],arg[2]) or M.export(arg[1],arg[2]))
