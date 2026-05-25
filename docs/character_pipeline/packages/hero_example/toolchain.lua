return function(ctx)
  return {
    schema_version = 1,
    shared_asset_root = "/media/damody/新增磁碟區/AI_Pic",
    linux_venv_root = "~/.cache/omoba-character-pipeline/venvs",
    providers = {
      comfyui = {
        kind = "image",
        priority = 1,
        root = "/media/damody/新增磁碟區/AI_Pic/ComfyUI/ComfyUI",
        api = "http://127.0.0.1:8188",
        workflow = "workflows/omoba_character_portrait.json",
      },
      stable_diffusion_webui = {
        kind = "image",
        priority = 2,
        root = "/media/damody/新增磁碟區/AI_Pic/stable-diffusion-webui",
        api = "http://127.0.0.1:7860",
        mode = "fallback",
      },
      hunyuan3d = {
        kind = "image_to_3d",
        priority = 1,
        root = "~/.cache/omoba-character-pipeline/providers/hunyuan3d",
        mode = "slot",
      },
      blender = {
        kind = "rig_export_check",
        executable = "blender",
      },
      kimodo_prompt = {
        kind = "text_to_motion_prompt",
        mode = "future_target",
      },
    },
    models = {
      preferred_checkpoint = "stable-diffusion-webui/models/Stable-diffusion/perfectWorld_perfectWorldBakedVAE.safetensors",
    },
    runner = {
      fixed_seed = true,
      long_gpu_jobs_required_for_validation = false,
    },
  }
end

