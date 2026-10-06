-- Endpoint/seat policy only. Authority admission and content agreement remain Rust-owned.
local M={}
function M.ipv4(value)
  assert(type(value)=='string','IPv4 address required')
  local parts={value:match('^(%d+)%.(%d+)%.(%d+)%.(%d+)$')}
  assert(#parts==4,'explicit IPv4 address required: '..value)
  for _,part in ipairs(parts) do
    assert(tostring(tonumber(part))==part and tonumber(part)<=255,'invalid IPv4 address: '..value)
  end
  assert(tonumber(parts[1])>=1 and tonumber(parts[1])<224
    and value~='255.255.255.255','unicast IPv4 address required: '..value)
  return value
end
function M.validate(options)
  options.server_bind=M.ipv4(options.server_bind or '127.0.0.1')
  if options.selection_bind then
    options.selection_bind=M.ipv4(options.selection_bind)
    assert(options.interactive_selection,'selection-bind requires interactive selection')
    assert(not options.connect,'remote gameplay clients must use the separate invitation selection entry')
  end
  if options.server_only then
    assert(not options.connect,'server-only cannot connect to another host')
    assert(next(options.local_players)==nil,'server-only cannot select local players')
    assert(not options.interactive_selection,'server-only cannot run interactive selection')
    assert(not options.finish_timeout_seconds,'server-only cannot capture native renderer results')
  end
  if options.connect then
    options.connect=M.ipv4(options.connect)
    assert(options.server_bind=='127.0.0.1','remote client cannot configure a server bind')
    assert(options.recipe and options.recipe:lower():match('%.json$'),
      'remote client requires the host final JSON recipe')
    assert(not options.interactive_selection and next(options.hero_selections)==nil,
      'remote client cannot change the host selection')
    assert(next(options.local_players)~=nil,'remote client requires --local-player')
  end
end
function M.local_humans(humans,requested)
  local selected,known={},{}
  for _,human in ipairs(humans) do known[human.player_id]=true end
  for id in pairs(requested) do assert(known[id],'local player is not an admitted human: '..id) end
  local all=next(requested)==nil
  for _,human in ipairs(humans) do
    if all or requested[human.player_id] then selected[#selected+1]=human end
  end
  table.sort(selected,function(a,b) return a.player_id<b.player_id end)
  return selected
end
return M
