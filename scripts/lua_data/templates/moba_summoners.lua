-- Training summoner spells. Names, descriptions, and balance live only here.
-- catalog_id is append-only numeric identity. Declaration order does not assign it.
-- An empty or omitted list is rejected by the content model. No legacy default.
return function(ctx)
  return {
    {
      catalog_id = 1,
      id = "training_point_blink_v1",
      display_name = "閃躍",
      description = "指定地面一點並瞬間移動。超出距離時沿射線夾到範圍內。可跨越牆面，落點必須可站。",
      cooldown = 300,
      target = "point",
      operation = { kind = "point_blink", range = 400, out_of_range = "clamp_to_range" },
    },
    {
      catalog_id = 2,
      id = "training_self_heal_v1",
      display_name = "復甦",
      description = "立即回復自身生命，不選取任何單位。",
      cooldown = 240,
      target = "none",
      operation = { kind = "self_heal", amount = 200 },
    },
  }
end
