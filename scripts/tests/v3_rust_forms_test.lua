-- v3_rust.lua forms on the host APIs (status_event, scheduled_effect, unit_var, damage_pre,
-- hero_command): across every hero document, each form is generated somewhere, in the shape the
-- host expects. tools\lua\lua.exe scripts\tests\v3_rust_forms_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local tools = path.join(b.root, 'docs', 'character_pipeline', 'tools')
local sources = dofile(path.join(tools, 'v3_sources.lua'))
PACKAGE_V3_AS_MODULE = true
local v3 = dofile(path.join(tools, 'package_v3.lua'))
PACKAGE_V3_AS_MODULE = nil
V3_RUST_AS_MODULE = true
local R = dofile(path.join(tools, 'v3_rust.lua'))
V3_RUST_AS_MODULE = nil

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end
local function has(text, needle) return text:find(needle, 1, true) ~= nil end

-- every generated entry file of every hero document
local texts = {}
for _, h in ipairs(sources.heroes(b.root)) do
  local doc = assert(v3.load_file(h.path))
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = h.package or 'x' })
  for rel, text in pairs(files) do
    local id = rel:match('^src/generated/([%w_]+)%.rs$')
    if id and id ~= 'registry' and id ~= 'values' and id ~= 'ids' and id ~= 'routines' and id ~= 'mod' then texts[#texts + 1] = { id = id, text = text } end
  end
end
local function some(pred)
  for _, t in ipairs(texts) do if pred(t.text) then return t end end
end

test('a delay schedules with its captures and comes back through on_scheduled', function()
  local t = assert(some(function(s) return has(s, 'cx.schedule(owner, ID, ') and has(s, 'fn on_scheduled(') end), 'a delay')
  assert(has(t.text, 'args.as_slice()'), t.id)
end)

test('a status trigger marks the status and answers on_status_event', function()
  assert(some(function(s) return has(s, 'fn on_status_event(&self, holder: EntityHandle, status: RStr') end), 'a status event hook')
end)

test('state variables read with their init and write with their scope', function()
  assert(some(function(s) return has(s, 'cx.state_num(') or has(s, 'cx.state_bool(') end), 'a read')
  assert(some(function(s) return has(s, 'cx.set_state_num(') or has(s, 'cx.set_state_bool(') or has(s, 'cx.set_state_unit(') end), 'a write')
end)

test('a damage-pre trigger returns its edit from on_damage_pre', function()
  local t = assert(some(function(s) return has(s, 'fn on_damage_pre(') end), 'a damage-pre hook')
  assert(has(t.text, 'cx.damage_edit'), t.id)
end)

test('chance rolls the host RNG', function()
  assert(some(function(s) return has(s, 'if cx.chance(p) {') end), 'a chance')
end)

test('a passive listening to its holder carries a marker with its events', function()
  local t = some(function(s) return s:find('cx%.passive_marker%(ID, .-, [1-9]%d*%)') ~= nil end)
  if t then assert(has(t.text, 'fn on_status_event(') or has(t.text, 'fn on_damage_pre('), t.id .. ': the marker events come back')
  else print('  (no generated passive listens to its holder yet)') end
end)

test('requires.has_status refuses the cast first', function()
  local t = some(function(s) return has(s, 'v3: requires has_status') end)
  if t then
    local cast = t.text:match("fn on_cast%(cx: &mut Cx<'_, '_>%) %-> Res {\n(.-)\n")
    assert(cast and has(cast, 'cx.has_status(cx.caster'), t.id .. ': the check is the first line of the cast')
  else print('  (no generated cast with requires.has_status yet)') end
end)

test('every damage carries its ability and v3 effect path (line B damage trace)', function()
  local n = 0
  for _, t in ipairs(texts) do
    assert(not has(t.text, 'cx.damage(to,'), t.id .. ': a damage without its path')
    for _, epath in t.text:gmatch('cx%.damage_at%("([%w_]+)", "([^"]*)"') do
      n = n + 1
      assert(epath ~= '?' and epath:sub(-1) == ']', t.id .. ': path ' .. epath)
    end
  end
  assert(n > 0, 'some generated damage')
end)

test('ability_level reads the host rank (ext)', function()
  assert(some(function(s) return has(s, 'cx.ability_level(u, ') end), 'an ability_level read')
end)

test('a taunt status taunts through cx.taunt and its removal ends it', function()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'akiyama_nobutomo' then h = x end end
  local doc = assert(v3.load_file(h.path))
  local ab
  for _, a in ipairs(doc.abilities) do if a.id == 'b24_taunt' then ab = a end end
  -- only the taunt: drop the parts the generator still blocks (include_illusions, the stop order)
  local each = ab.events.on_cast[1]
  local flags = {}
  for _, fl in ipairs(each.select.flags or {}) do if fl ~= 'include_illusions' then flags[#flags + 1] = fl end end
  each.select.flags = flags
  local branch = each.actions[2].actions[1].actions[1]
  branch.otherwise = nil
  each.actions[3] = nil -- the later stop order
  doc.abilities = { ab }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/b24_taunt.rs'], 'b24_taunt generates')
  assert(has(text, 'cx.taunt(to, cx.caster, duration)?;'), 'the apply taunts')
  assert(has(text, 'given by the taunt above'), 'the attack order is the taunt')
  assert(has(text, 'cx.end_taunt(from)?;'), 'removing the status ends the taunt')
end)

print(('%d tests passed'):format(count))
