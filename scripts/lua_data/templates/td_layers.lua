return function(_ctx)
  local damage = {
    sharp = 1,
    explosive = 2,
    energy = 4,
    fire = 8,
    cold = 16,
    normal = 32,
    crushing = 64,
    true_damage = 128,
  }
  local all = 255
  local property = { camo = 1, regrow = 2, fortified = 4, moab_class = 8 }
  local function layer(id, label, hp, speed, children, cash, leak, properties, accepted)
    return {
      id = id,
      label = label,
      hp = hp,
      move_speed = speed,
      children = children or {},
      cash = cash,
      leak_value = leak,
      properties = properties or 0,
      accepted_damage = accepted or all,
      regrow_eligible = true,
      fortified_eligible = true,
    }
  end

  return {
    layer("red", "Red", 1, 120, {}, 1, 1),
    layer("blue", "Blue", 1, 140, { "red" }, 1, 2),
    layer("green", "Green", 1, 160, { "blue" }, 1, 3),
    layer("yellow", "Yellow", 1, 185, { "green" }, 1, 4),
    layer("pink", "Pink", 1, 220, { "yellow" }, 1, 5),
    layer("black", "Black", 1, 180, { "pink", "pink" }, 1, 11, 0, all - damage.explosive),
    layer("white", "White", 1, 180, { "pink", "pink" }, 1, 11, 0, all - damage.cold),
    layer("purple", "Purple", 1, 180, { "pink", "pink" }, 1, 11, 0, all - damage.energy - damage.fire),
    layer("zebra", "Zebra", 1, 120, { "black", "white" }, 1, 23, 0, all - damage.explosive - damage.cold),
    layer("lead", "Lead", 1, 120, { "black", "black" }, 1, 23, 0, all - damage.sharp),
    layer("rainbow", "Rainbow", 1, 195, { "zebra", "zebra" }, 1, 47),
    layer("ceramic", "Ceramic", 10, 210, { "rainbow", "rainbow" }, 10, 104),
    layer("moab", "MOAB", 200, 80, { "ceramic", "ceramic", "ceramic", "ceramic" }, 200, 616, property.moab_class),
    layer("bfb", "BFB", 700, 60, { "moab", "moab", "moab", "moab" }, 700, 3164, property.moab_class),
    layer("zomg", "ZOMG", 4000, 45, { "bfb", "bfb", "bfb", "bfb" }, 4000, 16656, property.moab_class),
    layer("ddt", "DDT", 152, 260, {}, 152, 152, property.moab_class + property.camo, all - damage.sharp),
    layer("bad", "BAD", 67200, 35, {}, 67200, 67200, property.moab_class),
  }
end
