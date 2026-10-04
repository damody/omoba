-- Pure, fail-closed planner for retiring isolated duplicate event components.
local M = {}
local function index(snapshot)
  assert(snapshot.ok and not snapshot.truncated and snapshot.node_count == snapshot.total_node_count,
    'complete graph snapshot required')
  assert(#(snapshot.nodes or {}) == snapshot.node_count, 'incomplete node array')
  local nodes, edges = {}, {}
  for _, node in ipairs(snapshot.nodes or {}) do
    assert(node.id and not nodes[node.id] and node.content_hash, 'duplicate/unhashed node')
    nodes[node.id], edges[node.id] = node, {}
  end
  for _, node in ipairs(snapshot.nodes) do
    for _, pin in ipairs(node.pins or {}) do
      for _, link in ipairs(pin.connected_to or {}) do
        assert(nodes[link.node_id], 'snapshot has unresolved peer')
        edges[node.id][link.node_id], edges[link.node_id][node.id] = true, true
      end
    end
  end
  return nodes, edges
end
local function component(start, edges)
  local seen, queue = {[start]=true}, {start}
  for _, id in ipairs(queue) do
    for peer in pairs(edges[id]) do if not seen[peer] then seen[peer]=true; queue[#queue+1]=peer end end
  end
  return seen
end
local function event(nodes, name)
  local found
  for id, node in pairs(nodes) do
    if node.op == 'event' and node.detail.name == name then
      assert(not found, 'duplicate event: '..name); found=id
    end
  end
  return found
end
function M.plan(snapshot, rule)
  local nodes, edges = index(snapshot)
  local replacement = assert(event(nodes, rule.replacement_event), 'replacement event missing')
  assert(#(nodes[replacement].exec_to or {}) > 0, 'replacement event is disconnected')
  local replacement_nodes = component(replacement, edges)
  local found_call, fields = false, {}
  for id in pairs(replacement_nodes) do
    local node = nodes[id]
    if node.op == 'call' and node.detail['function'] == rule.sink_function then found_call=true end
    if node.op == 'break_struct' and node.detail.struct == rule.replacement_payload then
      for _, pin in ipairs(node.pins or {}) do
        if pin.direction == 'output' and #(pin.connected_to or {}) > 0 then fields[pin.name]=true end
      end
    end
  end
  assert(found_call, 'replacement sink missing')
  for _, field in ipairs(rule.required_fields) do assert(fields[field], 'replacement field missing: '..field) end
  -- Optional exact field-to-sink proof; only declared pure conversions may intervene.
  for field,binding in pairs(rule.field_bindings or {}) do
    local reached,visited=false,{}
    local function follow(pin)
      for _,link in ipairs(pin.connected_to or {}) do
        local peer=nodes[link.node_id]
        if peer.op=='call' and peer.detail['function']==rule.sink_function and link.pin==binding.sink_pin then
          reached=true
        elseif peer.is_pure and peer.op=='call' and peer.detail['function']==binding.via and not visited[peer.id] then
          visited[peer.id]=true
          for _,output in ipairs(peer.pins or {}) do if output.direction=='output' then follow(output) end end
        end
      end
    end
    for id in pairs(replacement_nodes) do
      local node=nodes[id]
      if node.op=='break_struct' and node.detail.struct==rule.replacement_payload then
        for _,pin in ipairs(node.pins or {}) do if pin.name==field and pin.direction=='output' then follow(pin) end end
      end
    end
    assert(reached,'replacement sink binding missing: '..field)
  end
  local root = event(nodes, rule.legacy_event)
  if not root then
    for _,node in pairs(nodes) do
      assert(not (node.op=='break_struct' and rule.legacy_payload and node.detail.struct==rule.legacy_payload), 'orphaned legacy payload remains')
      assert(not (node.op=='call' and node.detail['function']==rule.legacy_event),'legacy call remains')
    end
    return {patch={},removed={},preserved=nodes,already_migrated=true}
  end
  local retired = component(root, edges)
  local allowed_calls, allowed_ops = {}, {}
  for _, value in ipairs(rule.allowed_calls) do allowed_calls[value]=true end
  for _, value in ipairs(rule.allowed_ops) do allowed_ops[value]=true end
  local ids, preserved = {}, {}
  for id,node in pairs(nodes) do
    if retired[id] then
      assert(not replacement_nodes[id], 'legacy shares replacement nodes')
      assert(allowed_ops[node.op], 'unexpected legacy operation: '..tostring(node.op))
      assert(node.op ~= 'event' or id == root, 'legacy component shares another event')
      assert(node.op ~= 'call' or allowed_calls[node.detail['function']], 'unexpected legacy call')
      ids[#ids+1]=id
    else preserved[id]=node end
  end
  table.sort(ids)
  assert(#ids <= rule.max_removed_nodes, 'legacy component exceeds deletion bound')
  local patch = {}
  for _,id in ipairs(ids) do
    if id ~= root then patch[#patch+1]={op='delete_node',node_id=id,reconnect=false,expected_hash=nodes[id].content_hash} end
  end
  return {patch=patch,root=root,removed=ids,preserved=preserved,already_migrated=false}
end
function M.patch_result(value, count, preview)
  assert(value.ok == true and #(value.results or {}) == count, 'incomplete patch result')
  for _,item in ipairs(value.results) do
    assert(item.status ~= 'rejected', 'patch rejected: '..tostring(item.reason))
    if preview then assert(item.would_apply == true, 'preview did not approve item') end
  end
  if not preview then assert(value.applied == true, 'patch not applied') end
end
function M.verify(before_plan, after, rule, allow_root)
  local after_nodes = index(after)
  local root = event(after_nodes,rule.legacy_event)
  if allow_root then
    assert(root == before_plan.root, 'legacy root identity changed')
    for _,pin in ipairs(after_nodes[root].pins or {}) do assert(#(pin.connected_to or {})==0,'legacy root still connected') end
  else assert(not root, 'legacy event remains') end
  for id,node in pairs(before_plan.preserved) do
    assert(after_nodes[id] and after_nodes[id].content_hash == node.content_hash, 'preserved node changed: '..id)
    local json=require('tools.lua.lib.json')
    assert(json.encode(after_nodes[id])==json.encode(node),'preserved node wiring changed: '..id)
  end
  for _,id in ipairs(before_plan.removed) do
    if not (allow_root and id==root) then assert(not after_nodes[id], 'retired node remains') end
  end
  for id in pairs(after_nodes) do assert(before_plan.preserved[id] or (allow_root and id==root),'unexpected new node: '..id) end
  if not allow_root then assert(M.plan(after,rule).already_migrated, 'migration is not idempotent') end
end
return M
