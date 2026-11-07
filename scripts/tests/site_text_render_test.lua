-- Website text renderer against the shared vectors (docs/character_pipeline/schema/v3-text-render-vectors.lua).
-- UE's TooltipRenderer is checked against the same file.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local R = require('site.text_render')
local Q = require('site.rational')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

local function show(runs)
  local out = {}
  for _, r in ipairs(runs) do
    out[#out + 1] = string.format('{t=%q%s%s}', r.t, r.b and ',b' or '', r.dt and (',dt=' .. r.dt) or '')
  end
  return table.concat(out, ' ')
end

local vectors = dofile(dir .. '/../../docs/character_pipeline/schema/v3-text-render-vectors.lua')
ok(vectors.version == 2, 'vector version')
local names = {}
for _, case in ipairs(vectors.cases) do
  ok(not names[case.name], 'duplicate vector ' .. case.name)
  names[case.name] = true
  local okr, res = pcall(R.render_template, case.template, case)
  if case.expect.error then
    ok(not okr, case.name .. ': expected an error, got ' .. (okr and show(res) or ''))
    ok(tostring(res):find(case.expect.error, 1, true), case.name .. ': error should mention "' .. case.expect.error .. '": ' .. tostring(res))
  else
    ok(okr, case.name .. ': ' .. tostring(res))
    local want = case.expect.runs
    local same = #res == #want
    for i = 1, math.max(#res, #want) do
      local a, b = res[i], want[i]
      if not a or not b or a.t ~= b.t or (a.b or false) ~= (b.b or false) or a.dt ~= b.dt then same = false end
    end
    ok(same, case.name .. ':\n  got  ' .. show(res) .. '\n  want ' .. show(want))
  end
end

-- rational helpers the renderer relies on
ok(Q.exact(Q.parse(0.1)) == '1/10', 'float 0.1 reads as 1/10')
ok(Q.exact(Q.parse('19/101')) == '19/101', 'fraction')
ok(Q.to_raw(Q.parse('0.075')) == 77, '7.5% raw is 77/1024')
ok(Q.to_raw(Q.parse('-0.5')) == -512, 'negative raw')
ok(Q.decimal(Q.parse('2.005'), 2) == '2.01', 'half away from zero')
ok(Q.exact(Q.parse('1e-05')) == '1/100000', 'exponent')

-- html: bold inside the damage-type span, escaping, newline
local html = R.html({ { t = 'a<b' }, { t = '5', b = true, dt = 'magical' }, { t = '\nx' } })
ok(html == 'a&lt;b<span class="dt dt-magical"><b>5</b></span><br>x', 'html: ' .. html)

-- the JSON copy UE reads must match the Lua vectors
local X = require('site.export_vectors')
local f = io.open(X.JSON, 'rb')
local stored = f and f:read('a'); if f then f:close() end
ok(stored == X.text(), 'v3-text-render-vectors.json is stale: run tools\\lua\\lua.exe scripts\\site\\export_vectors.lua')

print(string.format('site_text_render_test: %d vectors, %d checks passed', #vectors.cases, checks))
