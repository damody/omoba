-- v3_rust.lua: an entry marked `hook = "<name>"` (D7, m3-rust-codegen-design.md §2.4) becomes a
-- trait its hand-written src/hooks/<id>.rs implements; an entry the generator cannot lower yet
-- emits no code at all (no stub) and stays in the report.
-- tools\lua\lua.exe scripts\tests\v3_rust_hook_test.lua
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

local function tamako()
  return assert(v3.load_file(assert(sources.hero_path(b.root, 'C15'))))
end
local function entry(doc, id)
  for _, a in ipairs(doc.abilities) do if a.id == id then return a end end
  error(id)
end

test('a hook entry is a trait its hand-written file implements', function()
  local doc = tamako()
  local a = entry(doc, 'c15_burst_arrow')
  a.hook, a.coverage, a.handler = 'c15_burst_arrow_hook', 'needs_hook', nil
  doc.hooks = { c15_burst_arrow_hook = { why = '試驗：整招手寫，確認產生的 trait 與實作檔的接法', test = 'c15_burst_arrow_hook_conformance',
    kind = 'ability', reads = { '$c15_burst_arrow.arrow_radius', '$c15_burst_arrow.fire_width' } } }
  local errs = v3.validate_doc(doc)
  assert(#errs == 0, tostring(errs[1]))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'tamako' })
  local rs = assert(files['src/generated/c15_burst_arrow.rs'], 'the hook entry is emitted')
  assert(has(rs, 'pub trait Hook {') and has(rs, "fn on_cast(cx: &mut Cx<'_, '_>) -> Res;"), 'the trait with the cast method')
  assert(has(rs, '/scripts/heroes/tamako/src/hooks/c15_burst_arrow.rs"]') and has(rs, 'mod hand;'), 'the hand-written file')
  assert(has(rs, '<hand::Impl as Hook>::on_cast(&mut cx)'), 'the cast calls the impl')
  for _, e in ipairs(report.entries) do
    if e.id == 'c15_burst_arrow' then assert(#e.blockers == 0, 'a hook entry has no blockers') end
  end
  assert(has(files['src/generated/registry.rs'], 'c15_burst_arrow::ffi()'), 'registered')
end)

test('an undeclared or wrong-kind hook is a schema error', function()
  local doc = tamako()
  local a = entry(doc, 'c15_burst_arrow')
  a.hook, a.coverage, a.handler = 'nope', 'needs_hook', nil
  local errs = v3.validate_doc(doc)
  local joined = table.concat((function() local t = {} for i, e in ipairs(errs) do t[i] = tostring(e) end return t end)(), '\n')
  assert(has(joined, '沒有宣告在 hooks'), joined)
  doc.hooks = { nope = { why = '試驗：kind 不是 ability 的 hook 不能掛在招式上', test = 'x', kind = 'action' } }
  a.hook = 'nope'
  errs = v3.validate_doc(doc)
  joined = table.concat((function() local t = {} for i, e in ipairs(errs) do t[i] = tostring(e) end return t end)(), '\n')
  assert(has(joined, 'kind = "ability"'), joined)
end)

test('an entry the generator cannot lower yet emits no code', function()
  local doc = tamako()
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'tamako' })
  local blocked
  for _, e in ipairs(report.entries) do if #e.blockers > 0 then blocked = e.id end end
  assert(blocked, 'C15 still has an entry the generator does not lower')
  assert(files['src/generated/' .. blocked .. '.rs'] == nil, 'no stub for ' .. blocked)
  assert(not has(files['src/generated/registry.rs'], blocked .. '::ffi()'), 'not registered')
end)

test('A13E chain sickle is the hand-written hook of hattori_hanzo', function()
  local doc = assert(v3.load_file(assert(sources.hero_path(b.root, 'A13'))))
  local errs = v3.validate_doc(doc)
  assert(#errs == 0, tostring(errs[1]))
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'hattori_hanzo' })
  local rs = assert(files['src/generated/a13_chain_sickle.rs'], 'emitted')
  assert(has(rs, '/scripts/heroes/hattori_hanzo/src/hooks/a13_chain_sickle.rs"]'), 'its file')
  assert(has(rs, "fn on_cast(cx: &mut Cx<'_, '_>) -> Res;") and has(rs, 'fn on_scheduled('), 'cast and its schedules')
  local f = assert(io.open(path.join(b.root, 'scripts', 'heroes', 'hattori_hanzo', 'src', 'hooks', 'a13_chain_sickle.rs'), 'rb'))
  local hand = f:read('a'); f:close()
  assert(has(hand, 'impl Hook for Impl'), 'implemented')
  -- no game numbers in the hand-written file: only step ids (`const FLY: u16 = 1`) may be literals
  for line in hand:gmatch('[^\n]+') do
    if not line:match('^%s*//') and not line:match('^const %u+: u16 = %d+;') then
      assert(not line:find('from_i32%(') and not line:find('from_raw%(') and not line:find('[^%w_]%d+%.%d+'), 'a number in: ' .. line)
    end
  end
  for _, h in pairs(doc.hooks) do assert(h.cases and #h.cases > 0, 'cases for conformance') end
end)

print(('%d tests passed'):format(count))
