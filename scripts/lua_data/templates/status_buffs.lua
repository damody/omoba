-- Buffs the v3 statuses show (E218). Every `buff` or `debuff` status of an authored hero ability
-- (templates/authored_heroes.lua) and of a shelf item (templates/item_texts_v3.lua) becomes a buff
-- entry after the hand-written ones (templates/buffs.lua), named by the status id, with its zh_TW
-- name and the BuffStore key it is held under (`runtime_id`, `{caster}`/`{rank}` included). The
-- host shows the status by that key (omoba_template_ids::status_buff_by_key); `hidden` statuses
-- get no entry. A key a hand-written buff already names (stun, root, magic_immune) keeps that buff.
-- Numeric buff ids follow this order, so they hold within one content hash; Unreal names buffs by id.
-- `public = true`: every status held under the key is `reveal = "public"`, so every team that sees the
-- unit may show the icon (E216); otherwise only the unit's own team does.
return function(ctx)
  local M = {}

  local function sorted_keys(t)
    local keys = {}
    for k in pairs(t) do keys[#keys + 1] = k end
    table.sort(keys)
    return keys
  end

  function M.apply(buffs, abilities, item_texts)
    -- key -> true while every status held under it is public (false once one is not).
    local public_key = {}
    local function note(statuses)
      for _, s in ipairs(statuses or {}) do
        if s.visibility == "buff" or s.visibility == "debuff" then
          local key = s.runtime_id or s.id
          public_key[key] = public_key[key] ~= false and s.reveal == "public"
        end
      end
    end
    for _, ability in ipairs(abilities or {}) do
      if not ability.tombstone and ability.v3 then note(ability.v3.statuses) end
    end
    for _, id in ipairs(sorted_keys(item_texts or {})) do note(item_texts[id].statuses) end
    local taken_id, taken_key = {}, {}
    for _, b in ipairs(buffs) do
      taken_id[b.id] = true
      taken_key[b.runtime_id or b.id] = true
    end
    local out = {}
    for _, b in ipairs(buffs) do
      if public_key[b.runtime_id or b.id] then
        local copy = {}
        for k, v in pairs(b) do copy[k] = v end
        copy.public = true
        b = copy
      end
      out[#out + 1] = b
    end
    local function add(statuses)
      for _, s in ipairs(statuses or {}) do
        local key = s.runtime_id or s.id
        if (s.visibility == "buff" or s.visibility == "debuff") and not taken_id[s.id] and not taken_key[key] then
          taken_id[s.id], taken_key[key] = true, true
          local name = s.text and s.text.zh_TW and s.text.zh_TW.name or s.id
          out[#out + 1] = { id = s.id, display_name = name, runtime_id = key, ue = { native_only = true },
            public = public_key[key] or nil }
        end
      end
    end
    for _, ability in ipairs(abilities or {}) do
      if not ability.tombstone and ability.v3 then add(ability.v3.statuses) end
    end
    for _, id in ipairs(sorted_keys(item_texts or {})) do add(item_texts[id].statuses) end
    return out
  end

  return M
end
