local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local desktop = require('ue_desktop')

local function env(value) return function(name) return name == 'OM_OWN_DESKTOP' and value or nil end end
assert(desktop.args(env(nil))[1] == '-RenderOffScreen', 'automation renders off-screen by default')
assert(desktop.args(env('0'))[1] == '-RenderOffScreen', 'only OM_OWN_DESKTOP=1 opts in')
assert(desktop.args(env('1'))[1] == '-om-own-desktop', 'OM_OWN_DESKTOP=1 keeps a visible window')
local list = desktop.append({'-d3d12'}, env(nil))
assert(#list == 2 and list[2] == '-RenderOffScreen', 'append adds the desktop switch')
local isolated = desktop.isolate_controls({'-d3d12'}, 'D:/run/controls')
assert(#isolated == 2 and isolated[2] == '-om-control-profile-dir=D:/run/controls', 'isolate_controls adds the profile directory')
assert(not pcall(desktop.isolate_controls, {}, ''), 'an empty control profile directory is rejected')
print('ue desktop: 6 checks passed')
