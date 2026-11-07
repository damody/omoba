-- Unreal processes started by automation (native test suites, owned-viewport fixtures,
-- autopilot/spotlight renderers) render off-screen by default. -RenderOffScreen makes UE use
-- the null platform application and a swapchain-less viewport, so no window appears on the
-- user's desktop, nothing takes the foreground and the real cursor is never moved (E114).
-- OM_OWN_DESKTOP=1 opts back in for a run that must exercise the real desktop route; the UE
-- side then sees -om-own-desktop and may bring windows to the front and move the cursor.
local M = {}

function M.owns_desktop(getenv)
  return (getenv or os.getenv)('OM_OWN_DESKTOP') == '1'
end

function M.args(getenv)
  if M.owns_desktop(getenv) then return {'-om-own-desktop'} end
  return {'-RenderOffScreen'}
end

function M.append(list, getenv)
  for _, value in ipairs(M.args(getenv)) do list[#list + 1] = value end
  return list
end

-- Automated runs never read or write the player's own control profile (Saved/Controls): a run killed
-- between a rebinding case and its restore left W unbound for the player and for every later run
-- (E141). Each automated launch gets its own profile directory, normally inside its evidence.
function M.isolate_controls(list, directory)
  assert(type(directory) == 'string' and directory ~= '', 'an automated UE launch needs its own control profile directory')
  list[#list + 1] = '-om-control-profile-dir=' .. directory
  return list
end

return M
