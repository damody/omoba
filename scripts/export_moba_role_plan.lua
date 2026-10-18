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
local function canonical_player_id(key)
  local text
  if math.type(key)=='integer' then text=string.format('%d',key)
  elseif type(key)=='string' then text=key
  else error('summoner player ID must be a canonical unsigned integer') end
  assert(text:match('^[0-9]+$'),'summoner player ID must be a canonical unsigned integer')
  local id=tonumber(text)
  assert(id and id>=1 and id<=4294967295 and id==math.floor(id) and tostring(id)==text,
    'summoner player ID must be a canonical unsigned integer')
  return id
end
local function spell_id(value)
  if value==nil or value==json.null then return nil end
  assert(type(value)=='string' and value:match('^[%w_]+$'),'summoner spell ID must be a catalog string')
  return value
end
local function row_slots(row)
  assert(type(row)=='table','summoner row must be a two-slot array')
  local seen=0
  for key in pairs(row) do
    assert(math.type(key)=='integer' and key>=1 and key<=2,'summoner row index must be slot 0 or slot 1')
    seen=seen+1
  end
  assert(seen==2 and row[1]~=nil and row[2]~=nil,'summoner row requires exactly two slots')
  return spell_id(row[1]),spell_id(row[2])
end
local function authenticated_summoner_loadouts(value)
  local loadouts=value.summoner_loadouts
  if loadouts==nil then return 'AUTHENTICATED_SUMMONER_LOADOUTS={}' end
  assert(loadouts~=json.null and type(loadouts)=='table','summoner_loadouts must be a map')
  local rows={}
  for key,row in pairs(loadouts) do
    local id=canonical_player_id(key)
    local slot0,slot1=row_slots(row)
    local fields={}
    if slot0 then fields[#fields+1]='slot0='..json.encode(slot0) end
    if slot1 then fields[#fields+1]='slot1='..json.encode(slot1) end
    rows[#rows+1]={id,('%s={%s}'):format(json.encode(tostring(id)),table.concat(fields,','))}
  end
  table.sort(rows,function(a,b) return a[1]<b[1] end)
  local encoded={}
  for index,row in ipairs(rows) do encoded[index]=row[2] end
  return 'AUTHENTICATED_SUMMONER_LOADOUTS={'..table.concat(encoded,',')..'}'
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
  -- MATCH_ROLE_PLAN_JSON keeps the authored arrays, including JSON null slots.
  -- AUTHENTICATED_SUMMONER_LOADOUTS is TOML and omits those nulls.
  return table.concat({
    'MATCH_GAMEPLAY_MODE="three_lane"',
    'MATCH_LOCKSTEP_MODE="secure_v2_required"',
    'MATCH_MANA_ENABLED='..tostring(value.mana_enabled==true),
    'MATCH_MAP_ID='..json.encode(assert(value.map_id,'role recipe requires map_id')),
    'MATCH_ROLE_PLAN_JSON='..json.encode(json.encode(value)),
    'AUTHENTICATED_TEAM_BINDINGS={'..table.concat(bindings,',')..'}',
    'AUTHENTICATED_HERO_BINDINGS={}',
    authenticated_summoner_loadouts(value),
  },'\n')..'\n'
end
if ... == 'export_moba_role_plan' then return M end
assert(#arg==2 or (#arg==3 and arg[3]=='--server-fragment'),
  'usage: export_moba_role_plan.lua INPUT.lua OUTPUT [--server-fragment]')
print(#arg==3 and M.server_fragment(arg[1],arg[2]) or M.export(arg[1],arg[2]))
