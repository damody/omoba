return function(ctx)
  return {
    schema_version = 1,
    portrait = {
      provider = "comfyui",
      positive_prompt = "full body stylized MOBA heroine, calm precise protective marksman, long asymmetrical deep red coat, compact arc rifle, gunmetal and ivory armor accents, cyan crystal glow, clean silhouette, readable from isometric game camera, high quality character concept art",
      negative_prompt = "text, logo, watermark, copyrighted character likeness, cluttered accessories, extra limbs, malformed hands, low resolution",
      seed = 230501,
      size = { width = 1024, height = 1536 },
      acceptance_criteria = {
        "full character visible",
        "no text or watermark",
        "silhouette remains readable at small size",
      },
      retry_policy = {
        max_attempts = 3,
        change_seed_on_retry = true,
      },
    },
    turnaround = {
      provider = "comfyui",
      positive_prompt = "orthographic 2D character turnaround sheet, front side back views of the same stylized MOBA marksman heroine, consistent outfit, compact arc rifle, neutral pose, clean white background",
      negative_prompt = "text, labels, watermark, inconsistent outfit, perspective distortion",
      seed = 230502,
      size = { width = 2048, height = 1024 },
      acceptance_criteria = {
        "front side and back views are consistent",
        "weapon scale remains stable",
      },
      retry_policy = {
        max_attempts = 3,
        change_seed_on_retry = true,
      },
    },
    skill_icons = {
      q = {
        provider = "comfyui",
        positive_prompt = "MOBA skill icon, thin cyan piercing bullet trail, red spark impact, dark readable background, no text",
        negative_prompt = "letters, numbers, watermark, busy background",
        seed = 230511,
        size = { width = 512, height = 512 },
      },
      w = {
        provider = "comfyui",
        positive_prompt = "MOBA skill icon, rotating star mark sigil, cyan and red glow, target lock motif, no text",
        negative_prompt = "letters, numbers, watermark, face portrait",
        seed = 230512,
        size = { width = 512, height = 512 },
      },
      e = {
        provider = "comfyui",
        positive_prompt = "MOBA skill icon, agile dash afterimage, broken cyan light shards, clean silhouette, no text",
        negative_prompt = "letters, numbers, watermark, clutter",
        seed = 230513,
        size = { width = 512, height = 512 },
      },
      r = {
        provider = "comfyui",
        positive_prompt = "MOBA ultimate skill icon, red cyan comet beam, high contrast energy shot, dramatic but readable, no text",
        negative_prompt = "letters, numbers, watermark, overexposed blur",
        seed = 230514,
        size = { width = 512, height = 512 },
      },
    },
    model_3d = {
      provider = "hunyuan3d",
      positive_prompt = "game-ready stylized 3D model of the same marksman heroine, clean topology target, long coat, compact arc rifle, readable silhouette",
      negative_prompt = "photorealistic pores, excessive tiny ornaments, loose cloth strips that are hard to rig",
      seed = 230521,
      target_format = "glb",
      acceptance_criteria = {
        "single humanoid mesh with separate weapon acceptable",
        "silhouette matches portrait and turnaround",
      },
    },
    rig = {
      provider = "blender",
      positive_prompt = "humanoid biped rig target, standard game skeleton, weapon bone for rifle, clean deformation",
      target_format = "glb",
      acceptance_criteria = {
        "root, spine, arms, legs, head bones exist",
        "weapon can attach to hand or weapon bone",
      },
    },
    animations = {
      provider = "kimodo_prompt",
      skeleton_target = "humanoid_game_biped",
      personality = { "calm", "precise", "protective" },
      clips = {
        idle = "calm alert rifle idle, controlled breathing",
        run = "light tactical run with compact rifle held close",
        attack = "precise rifle shot with small recoil",
        cast_q = "piercing aimed shot, quick focus then release",
        cast_w = "mark target gesture with star sigil projection",
        cast_e = "short evasive dash with afterimage",
        cast_r = "charged ultimate rifle stance, comet beam release",
        death = "controlled collapse, weapon lowers first",
      },
    },
  }
end

