return function(ctx)
  return {
    schema_version = 1,
    package_root = "docs/character_pipeline/packages/hero_smoke_astral_ranger",
    shared_output_root = "omoba_character_pipeline/hero_smoke_astral_ranger",
    assets = {
      portrait = {
        status = "planned",
        path = "outputs/portrait/hero_smoke_astral_ranger_portrait.png",
        format = "png",
        size = { width = 1024, height = 1536 },
      },
      turnaround = {
        status = "planned",
        path = "outputs/turnaround/hero_smoke_astral_ranger_turnaround.png",
        format = "png",
        size = { width = 2048, height = 1024 },
      },
      skill_icons = {
        q = { status = "planned", path = "outputs/icons/hero_smoke_astral_ranger_q.png", format = "png" },
        w = { status = "planned", path = "outputs/icons/hero_smoke_astral_ranger_w.png", format = "png" },
        e = { status = "planned", path = "outputs/icons/hero_smoke_astral_ranger_e.png", format = "png" },
        r = { status = "planned", path = "outputs/icons/hero_smoke_astral_ranger_r.png", format = "png" },
      },
      model = {
        status = "planned",
        path = "outputs/model/hero_smoke_astral_ranger.glb",
        format = "glb",
      },
      rig_report = {
        status = "planned",
        path = "outputs/rig/hero_smoke_astral_ranger_rig_report.json",
        format = "json",
      },
      animations = {
        idle = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_idle.glb", format = "glb" },
        run = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_run.glb", format = "glb" },
        attack = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_attack.glb", format = "glb" },
        cast_q = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_cast_q.glb", format = "glb" },
        cast_w = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_cast_w.glb", format = "glb" },
        cast_e = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_cast_e.glb", format = "glb" },
        cast_r = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_cast_r.glb", format = "glb" },
        death = { status = "planned", path = "outputs/animations/hero_smoke_astral_ranger_death.glb", format = "glb" },
      },
    },
  }
end

