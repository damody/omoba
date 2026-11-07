-- A toggle's spotlight segment lasts until the toggle can be turned off again (its cooldown and its
-- lock): a17_old 大筒 turned off at 4.5 s stayed on, -200% move speed, and its death was never shown.
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local spotlight=require('record_hero_spotlight')
local function hold(hero,id) return spotlight.hold_seconds(spotlight.v3_document(b.root,hero),id) end
assert(hold('sassa_narimasa_old','a17_old_great_cannon')==9.5,'大筒: its longest cooldown 9 s and lock 8 s, half a second to spare')
assert(hold('saika_magoichi_old','b01_old_sniper_mode')==10.5,'狙擊模式: its 10 s cooldown')
assert(hold('sassa_narimasa','a17_great_cannon')==nil,'a toggle off again within the base hold keeps it')
assert(hold('inahime','a02_venom_arrow')==nil,'a toggle without a cooldown keeps the base hold')
print('spotlight_toggle_hold_test: ok')
