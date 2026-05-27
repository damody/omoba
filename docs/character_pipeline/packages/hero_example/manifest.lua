return function(ctx)
  return {
    schema_version = 1,
    package_root = "docs/character_pipeline/packages/hero_example",
    shared_output_root = "omoba_character_pipeline/hero_example",
    assets = {
      portrait = {
        status = "planned",
        path = "outputs/portrait/hero_example_portrait.png",
        format = "png",
        size = { width = 1024, height = 1536 },
      },
      turnaround = {
        status = "planned",
        path = "outputs/turnaround/hero_example_turnaround.png",
        format = "png",
        size = { width = 2048, height = 1024 },
      },
      skill_icons = {
        q = { status = "planned", path = "outputs/icons/hero_example_q.png", format = "png" },
        w = { status = "planned", path = "outputs/icons/hero_example_w.png", format = "png" },
        e = { status = "planned", path = "outputs/icons/hero_example_e.png", format = "png" },
        r = { status = "planned", path = "outputs/icons/hero_example_r.png", format = "png" },
      },
      model = {
        status = "planned",
        path = "outputs/model/hero_example.glb",
        format = "glb",
      },
      rig_report = {
        status = "planned",
        path = "outputs/rig/hero_example_rig_report.json",
        format = "json",
      },
      animations = {
        idle = { status = "planned", path = "outputs/animations/hero_example_idle.glb", format = "glb" },
        run = { status = "planned", path = "outputs/animations/hero_example_run.glb", format = "glb" },
        attack = { status = "planned", path = "outputs/animations/hero_example_attack.glb", format = "glb" },
        cast_q = { status = "planned", path = "outputs/animations/hero_example_cast_q.glb", format = "glb" },
        cast_w = { status = "planned", path = "outputs/animations/hero_example_cast_w.glb", format = "glb" },
        cast_e = { status = "planned", path = "outputs/animations/hero_example_cast_e.glb", format = "glb" },
        cast_r = { status = "planned", path = "outputs/animations/hero_example_cast_r.glb", format = "glb" },
        death = { status = "planned", path = "outputs/animations/hero_example_death.glb", format = "glb" },
      },
    },
  }
end

