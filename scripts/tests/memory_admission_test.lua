-- E441: a worker starts only while the available commit covers MARGIN times its estimate; a
-- refusal is logged once and admission resumes when commit frees up.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local admission = require('memory_admission')

local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local available, logs = 0, {}
local gate = admission.new(1000, { status = function() return { available_commit_bytes = available } end,
  log = function(line) logs[#logs + 1] = line end })
available = 2000
check(gate:admit(0), 'exactly the margin is enough')
available = 1999
check(not gate:admit(3), 'below the margin waits')
check(not gate:admit(3) and #logs == 1, 'the wait is logged once')
available = 5000
check(gate:admit(2) and #logs == 2 and logs[2]:find('starting workers again', 1, true), 'admission resumes and says so')
check(not pcall(admission.new(1, { status = function() return {} end, log = function() end }).admit), 'a status without commit is an error')
local slept, frees = 0, { 10, 10, 3000 }
local build = admission.new(3000, { margin = 1, log = function() end,
  status = function() slept = slept + 1; return { available_commit_bytes = frees[slept] } end })
build:wait(function(ms) check(ms == admission.WAIT_POLL_MS, 'the wait polls at WAIT_POLL_MS') end)
check(slept == 3, 'a build waits until its estimate is free once (margin 1)')
check(admission.BUILD_MARGIN == 1 and math.type(admission.BUILD_COMMIT_BYTES) == 'integer', 'the build defaults are defined once')
check(admission.MARGIN == 2 and math.type(admission.HEADLESS_COMMIT_BYTES) == 'integer', 'the defaults are defined once')
print(('memory_admission tests passed (%d checks)'):format(checks))
