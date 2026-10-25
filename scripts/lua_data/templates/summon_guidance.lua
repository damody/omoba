-- Shared authoring-time explanation; the same policy drives runtime selection.
return function(ctx)
  local guidance = {}
  for _, summon in ipairs(ctx.include("templates/summons.lua")) do
    assert(type(summon.id) == "string" and summon.id ~= "" and guidance[summon.id] == nil,
      "summon guidance requires unique nonempty IDs")
    local policy = summon.target_priority
    if policy == nil or policy == "nearest" then
      guidance[summon.id] = "索敵：自動攻擊附近可見、射程內的合法敵人，以最近目標優先。"
    elseif policy == "owner_focus" then
      guidance[summon.id] = "索敵：優先協助攻擊主人目前指定的敵人；目標仍須可見、在召喚物射程內且可被攻擊。主人停止攻擊或指定目標無效時，改攻擊最近的合法敵人。"
    else
      error("unsupported summon target_priority for " .. summon.id)
    end
  end
  return guidance
end
