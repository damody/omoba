return function(ctx)
  return {
    schema_version = 1,
    hero = {
      id = "hero_example",
      display_name = "範例英雄",
      title = "星痕遊俠",
    },
    abilities = {
      { id = "hero_example_q", slot = "Q", name = "穿星射擊" },
      { id = "hero_example_w", slot = "W", name = "星痕標記" },
      { id = "hero_example_e", slot = "E", name = "折光步" },
      { id = "hero_example_r", slot = "R", name = "彗星終幕" },
    },
    assets = {
      portrait = "omfx/data/hero_portraits/hero_example_portrait.png",
      icons = {
        q = "omfx/data/ability_icons/hero_example_q.png",
        w = "omfx/data/ability_icons/hero_example_w.png",
        e = "omfx/data/ability_icons/hero_example_e.png",
        r = "omfx/data/ability_icons/hero_example_r.png",
      },
      model = "omfx/data/heroes/hero_example/hero_example.glb",
    },
    animations = {
      idle = "omfx/data/heroes/hero_example/animations/idle.glb",
      run = "omfx/data/heroes/hero_example/animations/run.glb",
      attack = "omfx/data/heroes/hero_example/animations/attack.glb",
      cast_q = "omfx/data/heroes/hero_example/animations/cast_q.glb",
      cast_w = "omfx/data/heroes/hero_example/animations/cast_w.glb",
      cast_e = "omfx/data/heroes/hero_example/animations/cast_e.glb",
      cast_r = "omfx/data/heroes/hero_example/animations/cast_r.glb",
      death = "omfx/data/heroes/hero_example/animations/death.glb",
    },
    script_hints = {
      hero_example_q = { kind = "projectile", damage = "physical", shape = "line" },
      hero_example_w = { kind = "buff", target = "enemy", tag = "mark" },
      hero_example_e = { kind = "mobility", movement = "short_dash" },
      hero_example_r = { kind = "projectile", damage = "physical", shape = "single_target_ultimate" },
    },
    template_draft = {
      status = "draft",
      base_stats = {
        hp = 620,
        mana = 300,
        attack_damage = 54,
        attack_range = 720,
        move_speed = 315,
      },
    },
    apply_automatically = false,
  }
end

