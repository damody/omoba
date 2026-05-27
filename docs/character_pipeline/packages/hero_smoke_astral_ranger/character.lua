return function(ctx)
  return {
    schema_version = 1,
    hero = {
      id = "hero_smoke_astral_ranger",
      display_name = "煙測星界遊俠",
      title = "星界煙測者",
      gender = "female",
      personality = { "calm", "precise", "protective" },
      role = "marksman",
      combat_read = "遠程單點輸出，使用星界標記與穿透射擊控制走位。",
      art_direction = {
        style = "stylized moba readable from isometric camera",
        silhouette = "long asymmetrical coat, compact arc rifle, high collar",
        palette = { "deep red", "gunmetal", "ivory", "cyan glow" },
        materials = { "matte cloth", "brushed metal", "faint crystal inlay" },
        avoid = { "text", "logos", "copyrighted character likeness", "excessive tiny accessories" },
      },
    },
    gameplay = {
      faction = "player",
      base_stats_draft = {
        hp = 620,
        mana = 300,
        attack_damage = 54,
        attack_range = 720,
        move_speed = 315,
      },
    },
    abilities = {
      {
        id = "hero_smoke_astral_ranger_q",
        slot = "Q",
        name = "星界穿刺",
        gameplay_intent = "line projectile damage",
        visual_motif = "thin cyan ballistic trail with a red impact spark",
      },
      {
        id = "hero_smoke_astral_ranger_w",
        slot = "W",
        name = "星界標記",
        gameplay_intent = "single target mark that amplifies follow-up damage",
        visual_motif = "small rotating star sigil above the target",
      },
      {
        id = "hero_smoke_astral_ranger_e",
        slot = "E",
        name = "相位步",
        gameplay_intent = "short reposition with brief movement speed bonus",
        visual_motif = "afterimage dash with broken cyan light shards",
      },
      {
        id = "hero_smoke_astral_ranger_r",
        slot = "R",
        name = "星幕終擊",
        gameplay_intent = "charged ultimate shot against a marked target",
        visual_motif = "large focused red-cyan comet beam",
      },
    },
    automation = {
      mode = "auto_decide",
      batch_review = true,
      seed_policy = "fixed",
      assumptions = {
        "No user-provided camera style, defaulted to stylized isometric MOBA readability.",
        "No numeric balance provided, generated draft stats only.",
      },
    },
  }
end

