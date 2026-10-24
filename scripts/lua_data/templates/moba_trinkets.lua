-- Training trinket catalog. Names, descriptions, and balance live only here.
-- catalog_id is append-only numeric identity. Declaration order does not assign it.
-- An empty or omitted list is rejected by the content model. No legacy default.
-- This row is original training content. It is not an item, ability, or summoner,
-- and it does not place a ward or grant vision by itself.
return function(ctx)
  return {
    {
      catalog_id = 1,
      id = "training_scout_lantern_v1",
      display_name = "偵察燈",
      description = "放置後暫時提供友方視野。電荷有限，用盡後需等待回復。",
      placement_range = 600,
      initial_charges = 1,
      max_charges = 2,
      recharge_seconds = 120,
      lifetime_seconds = 90,
      sight_radius = 900,
      detection_level = 0,
      stealth_level = 1,
      hit_points = 3,
      maximum_owned_placements = 3,
      direct_attack = "disclosed_enemy",
      automatic_acquisition = "disabled",
    },
    {
      catalog_id = 2,
      id = "training_detection_lantern_v1",
      display_name = "偵隱燈",
      description = "放置可被看見的偵隱燈，提供友方視野並揭露範圍內的一般隱形敵人；更強的隱形效果無法被揭露。地形仍會阻擋視線；電荷回復較慢。",
      placement_range = 600,
      initial_charges = 1,
      max_charges = 1,
      recharge_seconds = 180,
      lifetime_seconds = 60,
      sight_radius = 600,
      detection_level = 2,
      stealth_level = 0,
      hit_points = 4,
      maximum_owned_placements = 1,
      direct_attack = "disclosed_enemy",
      automatic_acquisition = "disabled",
    },
  }
end
