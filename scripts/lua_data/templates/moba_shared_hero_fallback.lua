-- Unreal-only presentation fallback for heroes that author no character art yet: one of the
-- existing training humanoids (Quaternius rigs with idle/move/attack/cast/hit/death clips),
-- picked by a stable hash of the hero id so heroes stay distinguishable. Never gameplay data.
-- Codegen records it as the class's shared fallback mesh; art assigned in the Editor
-- (unreal-art-guide.md section 0) replaces it and counts as authored art.
return function(ctx)
  local profiles=ctx.include('templates/moba_native_visuals.lua')
  local rigs={'training_vanguard','training_luminary','training_ranger','training_support'}
  local function copy(value)
    if type(value)~='table' then return value end
    local out={}
    for key,item in pairs(value)do out[key]=copy(item)end
    return out
  end
  local M={}
  function M.profile(hero_id,previous)
    assert(type(hero_id)=='string' and hero_id~='','shared hero fallback needs a hero id')
    local hash=0
    for index=1,#hero_id do hash=(hash*31+hero_id:byte(index))%2147483647 end
    local profile=copy(assert(profiles[rigs[hash%#rigs+1]],'missing shared fallback rig profile'))
    profile.shared_fallback=true
    -- A hero's own authored size survives; the rig only supplies the body and clips.
    if previous and previous.height_world_units then profile.height_world_units=previous.height_world_units end
    return profile
  end
  return M
end
