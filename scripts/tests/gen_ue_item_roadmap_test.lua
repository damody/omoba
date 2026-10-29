-- M7: the 尚未開放 reason shows blocker families players can read, never engine codes.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
ITEM_ROADMAP_AS_MODULE = true
local M = require('gen_ue_item_roadmap')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

local labels = M.blocker_labels({ 'action:repeat', 'passive:aura', 'action:chance', 'passive_trigger:on_created', 'fn:has_status', 'token:state', 'stat:slow_pct' })
ok(table.concat(labels, ',') == '特殊動作,光環,被動觸發,條件判斷,屬性', 'families in first-seen order, each once')
ok(M.blocker_labels({ 'passive:orb_slot' })[1] == '被動效果', 'other passives')
ok(M.blocker_labels({ 'something_new:x' })[1] == '條件判斷', 'unknown codes still get a readable family')
for _, l in ipairs(M.blocker_labels({ 'state:mute', 'event:on_unequip' })) do ok(not l:find('[%w_:]'), 'no code leaks: ' .. l) end

local rows = M.rows({ { id = 'item_a', name = 'A', has_ir = true, blockers = { 'action:repeat' }, step = 2 } },
  { items = {} }, { entries = {} }, {})
ok(rows[1].blockers[1] == '特殊動作' and rows[1].blocker_codes[1] == 'action:repeat' and rows[1].state == 'blocked', 'row keeps the codes beside the labels')

print('gen_ue_item_roadmap_test: ' .. checks .. ' checks passed')
