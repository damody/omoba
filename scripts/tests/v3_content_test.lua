-- Full v3 content (scripts/gen_v3_content.lua): every hero and item compiles, every hero is in the
-- game, and every entry with data behavior has generated Rust (there is no runtime IR, M3'). The
-- content-model load is scripts/check_v3_content.lua (it runs the Rust test).
-- tools\lua\lua.exe scripts\tests\v3_content_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
GEN_V3_CONTENT_AS_MODULE = true
local G = dofile(b.root .. '/scripts/gen_v3_content.lua')
GEN_V3_CONTENT_AS_MODULE = nil

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local content, problems = G.build(b.root)

test('every_document_compiles', function()
  assert(#problems == 0, table.concat(problems, '\n'))
  assert(content.kind == 'full' and content.content_hash, 'kind and hash')
end)

test('every_hero_is_listed', function()
  assert(#content.heroes >= 97, 'every history_nobu hero, got ' .. #content.heroes)
  local listed = 0
  for _, h in ipairs(content.heroes) do
    if h.listed then listed = listed + 1; assert(h.game_id, h.source_id .. ': a listed hero has its game id') end
    for _, id in ipairs(h.abilities) do assert(content.abilities[id], h.source_id .. ' names missing ability ' .. id) end
  end
  assert(listed == #content.heroes, 'every hero has its own DLL and passes the full regression, listed ' .. listed)
end)

-- A hero entry with behavior generates; an item entry generates or names what blocks it (roadmap).
test('behavior_entries_have_generated_rust', function()
  PACKAGE_V3_AS_MODULE = true
  local V = dofile(b.root .. '/docs/character_pipeline/tools/package_v3.lua')
  PACKAGE_V3_AS_MODULE = nil
  local S = dofile(b.root .. '/docs/character_pipeline/tools/v3_sources.lua')
  V3_CODEGEN_COVERAGE_AS_MODULE = true
  local cov = dofile(b.root .. '/docs/character_pipeline/tools/v3_codegen_coverage.lua')
  V3_CODEGEN_COVERAGE_AS_MODULE = nil
  local scan = cov.scan(b.root)
  assert(#scan.problems == 0, table.concat(scan.problems, '\n'))
  local generated, blocked = {}, {}
  for _, h in ipairs(scan.heroes) do
    for _, e in ipairs(h.entries) do
      if #e.blockers == 0 then generated[e.id] = true else blocked[#blocked + 1] = e.id end
    end
  end
  assert(#blocked == 0, 'hero entries the generator refuses: ' .. table.concat(blocked, ', '))
  for _, e in ipairs(scan.items) do generated[e.id] = true end
  local missing = {}
  for _, f in ipairs(S.all(b.root)) do
    local d = assert(V.load_file(f))
    for _, list in ipairs({ d.abilities or {}, d.items or {} }) do
      for _, e in ipairs(list) do
        local castable = e.active or e
        local behavior = e.passive or (e.statuses and #e.statuses > 0) or castable.events
        if behavior and e.coverage ~= 'deferred' and not generated[e.id] then missing[#missing + 1] = e.id end
      end
    end
  end
  assert(#missing == 0, 'entries with behavior and no generated Rust: ' .. table.concat(missing, ', '))
end)

print(count .. ' v3 content tests passed')
