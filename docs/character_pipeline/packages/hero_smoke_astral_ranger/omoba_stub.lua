return function(ctx)
  return {
    schema_version = 1,
    hero = {
      id = "hero_smoke_astral_ranger",
      display_name = "煙測星界遊俠",
      title = "星界煙測者",
    },
    abilities = {
      { id = "hero_smoke_astral_ranger_q", slot = "Q", name = "星界穿刺" },
      { id = "hero_smoke_astral_ranger_w", slot = "W", name = "星界標記" },
      { id = "hero_smoke_astral_ranger_e", slot = "E", name = "相位步" },
      { id = "hero_smoke_astral_ranger_r", slot = "R", name = "星幕終擊" },
    },
    assets = {
      portrait = "omfx/data/hero_portraits/hero_smoke_astral_ranger_portrait.png",
      icons = {
        q = "omfx/data/ability_icons/hero_smoke_astral_ranger_q.png",
        w = "omfx/data/ability_icons/hero_smoke_astral_ranger_w.png",
        e = "omfx/data/ability_icons/hero_smoke_astral_ranger_e.png",
        r = "omfx/data/ability_icons/hero_smoke_astral_ranger_r.png",
      },
      model = "omfx/data/heroes/hero_smoke_astral_ranger/hero_smoke_astral_ranger.glb",
    },
    animations = {
      idle = "omfx/data/heroes/hero_smoke_astral_ranger/animations/idle.glb",
      run = "omfx/data/heroes/hero_smoke_astral_ranger/animations/run.glb",
      attack = "omfx/data/heroes/hero_smoke_astral_ranger/animations/attack.glb",
      cast_q = "omfx/data/heroes/hero_smoke_astral_ranger/animations/cast_q.glb",
      cast_w = "omfx/data/heroes/hero_smoke_astral_ranger/animations/cast_w.glb",
      cast_e = "omfx/data/heroes/hero_smoke_astral_ranger/animations/cast_e.glb",
      cast_r = "omfx/data/heroes/hero_smoke_astral_ranger/animations/cast_r.glb",
      death = "omfx/data/heroes/hero_smoke_astral_ranger/animations/death.glb",
    },
    script_hints = {
      hero_smoke_astral_ranger_q = { kind = "projectile", damage = "physical", shape = "line" },
      hero_smoke_astral_ranger_w = { kind = "buff", target = "enemy", tag = "mark" },
      hero_smoke_astral_ranger_e = { kind = "mobility", movement = "short_dash" },
      hero_smoke_astral_ranger_r = { kind = "projectile", damage = "physical", shape = "single_target_ultimate" },
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

