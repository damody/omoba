-- What the v3 documents do not carry yet, read from the pinned original (history_nobu):
--   * icons: nobu's own textures (AbilityTextureName -> resource/flash3/images/...png;
--     hero portraits: override_hero -> content/.../panorama/images/heroes/<name>.png);
--   * shop categories (scripts/shops/nobu_shops.txt) for items without shop.category.
-- Icons are returned as paths relative to the source root; build_site copies them.
-- Every lookup returns nil when the original is missing and the page shows a placeholder.
local M = {}

M.DEFAULT_ROOT = "C:/nobu/history_nobu"
local GAME = "game/dota_addons/nobu"
local CONTENT = "content/dota_addons/nobu"

-- Valve KV tokenizer: quoted or bare strings, braces; // comments
local function kv_tokens(text)
  local toks, i, n = {}, 1, #text
  while i <= n do
    local c = text:sub(i, i)
    if c:match("%s") then i = i + 1
    elseif text:sub(i, i + 1) == "//" then
      local e = text:find("\n", i, true); i = e and e + 1 or n + 1
    elseif c == "{" or c == "}" then toks[#toks + 1] = c; i = i + 1
    elseif c == '"' then
      local e = text:find('"', i + 1, true) or n
      toks[#toks + 1] = { s = text:sub(i + 1, e - 1) }; i = e + 1
    else
      local s, e = text:find("^[^%s{}\"]+", i)
      toks[#toks + 1] = { s = text:sub(s, e) }; i = e + 1
    end
  end
  return toks
end

-- calls fn(path, key, value) for every key/value pair; path = keys of the enclosing blocks
local function kv_walk(text, fn)
  local toks = kv_tokens(text)
  local stack, i = {}, 1
  while i <= #toks do
    local t = toks[i]
    if t == "}" then stack[#stack] = nil; i = i + 1
    elseif t == "{" then i = i + 1
    else
      local nxt = toks[i + 1]
      if nxt == "{" then stack[#stack + 1] = t.s; i = i + 2
      elseif type(nxt) == "table" then fn(stack, t.s, nxt.s); i = i + 2
      else i = i + 1 end
    end
  end
end

local function read(p)
  local f = io.open(p, "rb")
  if not f then return nil end
  local s = f:read("a"); f:close()
  return s
end

local function exists(p) local f = io.open(p, "rb"); if f then f:close(); return true end; return false end

function M.open(source_root)
  local lfs = require("lfs")
  source_root = (source_root or M.DEFAULT_ROOT):gsub("\\", "/")
  local S = { root = source_root, available = lfs.attributes(source_root .. "/" .. GAME, "mode") == "directory" }
  local textures, overrides, shop = {}, {}, {}
  local shop_order = {}
  if S.available then
    local npc = source_root .. "/" .. GAME .. "/scripts/npc"
    local function scan(dir)
      for name in lfs.dir(dir) do
        local p = dir .. "/" .. name
        if name ~= "." and name ~= ".." then
          if lfs.attributes(p, "mode") == "directory" then scan(p)
          elseif name:match("%.txt$") then
            local text = read(p)
            if text then
              local hero = name:match("^(%u%d%d)_2%.txt$")
              kv_walk(text, function(path, k, v)
                if k == "AbilityTextureName" and #path == 2 and textures[path[2]] == nil then textures[path[2]] = v end
                if k == "override_hero" and hero then overrides[hero] = v end
              end)
            end
          end
        end
      end
    end
    scan(npc)
    local shops = read(source_root .. "/" .. GAME .. "/scripts/shops/nobu_shops.txt")
    if shops then
      kv_walk(shops, function(path, k, v)
        if k == "item" and #path == 2 then
          local cat = path[2]
          shop[v] = shop[v] or {}
          local seen = false
          for _, c in ipairs(shop[v]) do if c == cat then seen = true end end
          if not seen then shop[v][#shop[v] + 1] = cat end
          if not shop_order[cat] then shop_order[#shop_order + 1] = cat; shop_order[cat] = #shop_order end
        end
      end)
    end
  end
  S.shop_order = shop_order

  local function texture_png(tex)
    if not tex then return nil end
    local rel
    if tex:match("^item_") then rel = GAME .. "/resource/flash3/images/items/" .. tex:gsub("^item_", "") .. ".png"
    else rel = GAME .. "/resource/flash3/images/spellicons/" .. tex .. ".png" end
    if exists(source_root .. "/" .. rel) then return rel end
    return nil
  end

  -- the KV AbilityTextureName as written, and the hero's override_hero (nil when absent)
  function S.texture(source_id) return textures[source_id] end
  function S.texture_png(tex) return texture_png(tex) end
  function S.override_hero(hero_id) return overrides[hero_id] end
  function S.ability_icon(source_id) return texture_png(textures[source_id]) end
  function S.item_icon(source_id) return texture_png(textures[source_id]) end
  function S.hero_icon(hero_id)
    local o = overrides[hero_id]
    if not o then return nil end
    for _, dir in ipairs({ "/panorama/images/heroes/", "/panorama/images/heroes/selection/" }) do
      for _, ext in ipairs({ ".png", ".jpg" }) do
        local rel = CONTENT .. dir .. o .. ext
        if exists(source_root .. "/" .. rel) then return rel end
      end
    end
    return nil
  end
  -- every shop the item is sold in, in file order; the first one that is not "pregame" is its category
  function S.shop_categories(source_id) return shop[source_id] end
  function S.shop_category(source_id)
    for _, c in ipairs(shop[source_id] or {}) do if c ~= "pregame" then return c end end
    return nil
  end
  return S
end

return M
