-- Writes docs/character_pipeline/schema/v3-text-render-vectors.json from the Lua vectors, for
-- consumers that cannot run Lua (UE's Om.Runtime.TooltipRendererVectors test). The Lua file is
-- the source; scripts/tests/site_text_render_test.lua fails when the JSON is stale.
--
--   tools\lua\lua.exe scripts\site\export_vectors.lua [--check]
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local M = {}

M.LUA = b.root .. '/docs/character_pipeline/schema/v3-text-render-vectors.lua'
M.JSON = b.root .. '/docs/character_pipeline/schema/v3-text-render-vectors.json'

-- a value { "1", "2", unit = "s" } becomes { ranks = { "1", "2" }, unit = "s", ... }
local function value(v)
  local out = { ranks = {} }
  for i, x in ipairs(v) do out.ranks[i] = tostring(x) end
  for k, x in pairs(v) do if type(k) == 'string' then out[k] = x end end
  return out
end

local function values(t)
  if not t then return nil end
  local out = json.object({})
  for name, v in pairs(t) do out[name] = value(v) end
  return out
end

function M.text()
  local vectors = dofile(M.LUA)
  local cases = {}
  for i, c in ipairs(vectors.cases) do
    local owners
    if c.owners then
      owners = json.object({})
      for o, t in pairs(c.owners) do owners[o] = values(t) end
    end
    local expect = json.object({})
    if c.expect.error then expect.error = c.expect.error
    else
      expect.runs = {}
      for j, r in ipairs(c.expect.runs) do
        expect.runs[j] = json.object({ t = r.t, b = r.b or nil, dt = r.dt })
      end
    end
    cases[i] = json.object({
      name = c.name, lang = c.lang, template = c.template, rank = c.rank, hero_level = c.hero_level,
      values = values(c.values) or json.object({}), owners = owners, rules = c.rules, names = c.names,
      expect = expect,
    })
  end
  return json.encode(json.object({ version = vectors.version, source = 'v3-text-render-vectors.lua', cases = cases })) .. '\n'
end

if arg and arg[0] and arg[0]:match('export_vectors%.lua$') then
  local text = M.text()
  local f = io.open(M.JSON, 'rb')
  local old = f and f:read('a'); if f then f:close() end
  if arg[1] == '--check' then
    if old ~= text then io.stderr:write('stale: ' .. M.JSON .. '\n'); os.exit(1) end
    print('up to date: ' .. M.JSON)
  else
    local out = assert(io.open(M.JSON, 'wb')); out:write(text); out:close()
    print('wrote ' .. M.JSON)
  end
end
return M
