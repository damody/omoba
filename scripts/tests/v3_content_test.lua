-- Full v3 content (scripts/gen_v3_content.lua): every hero and item compiles, unlisted heroes are
-- marked, and every entry with data behavior carries IR. The content-model load is
-- scripts/check_v3_content.lua (it runs the Rust test).
-- tools\lua\lua.exe scripts\tests\v3_content_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
GEN_V3_CONTENT_AS_MODULE = true
local G = dofile(b.root .. '/scripts/gen_v3_content.lua')
GEN_V3_CONTENT_AS_MODULE = nil

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local content, problems = G.build(b.root)

test('every_document_compiles_with_ir', function()
  assert(#problems == 0, table.concat(problems, '\n'))
  assert(content.kind == 'full' and content.content_hash, 'kind and hash')
end)

test('every_hero_is_listed_or_unlisted', function()
  assert(#content.heroes >= 97, 'every history_nobu hero, got ' .. #content.heroes)
  local listed = 0
  for _, h in ipairs(content.heroes) do
    if h.listed then listed = listed + 1; assert(h.game_id, h.source_id .. ': a listed hero has its game id') end
    for _, id in ipairs(h.abilities) do assert(content.abilities[id], h.source_id .. ' names missing ability ' .. id) end
  end
  assert(listed >= 10 and listed < #content.heroes, 'shipped heroes are listed, the rest unlisted')
end)

test('behavior_entries_carry_ir', function()
  PACKAGE_V3_AS_MODULE = true
  local V = dofile(b.root .. '/docs/character_pipeline/tools/package_v3.lua')
  PACKAGE_V3_AS_MODULE = nil
  local S = dofile(b.root .. '/docs/character_pipeline/tools/v3_sources.lua')
  local missing = {}
  for _, f in ipairs(S.all(b.root)) do
    local d = assert(V.load_file(f))
    for _, list in ipairs({ { d.abilities or {}, content.abilities }, { d.items or {}, content.items } }) do
      for _, e in ipairs(list[1]) do
        local castable = e.active or e
        local behavior = e.passive or (e.statuses and #e.statuses > 0) or castable.events
        if behavior and e.coverage ~= 'deferred' and not (list[2][e.id] and list[2][e.id].ir) then missing[#missing + 1] = e.id end
      end
    end
  end
  assert(#missing == 0, 'entries with behavior but no IR: ' .. table.concat(missing, ', '))
end)

print(count .. ' v3 content tests passed')
