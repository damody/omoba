-- Raw source pack for one hero: what an author (or a blind-test agent) gets
-- before any analysis. Only catalog facts: provenance, unit KV, tooltip text
-- and ability KV, in the hero's own slot order. No classification, no
-- capability verdicts, no local mapping.
local M = {}

local function fence(text)
  local body = (text or ""):gsub("\r\n", "\n")
  local ticks = "```"
  while body:find(ticks, 1, true) do ticks = ticks .. "`" end
  return ticks .. "text\n" .. body .. "\n" .. ticks
end

local function by_id(catalog)
  local out = {}
  for _, a in ipairs(catalog.abilities) do out[a.id] = out[a.id] or a end
  return out
end

-- Abilities of the hero in unit slot order, then any other block of the
-- hero's file (old variants, helpers) in file order.
local function ordered(catalog, hero)
  local index, seen, rows = by_id(catalog), {}, {}
  for _, slot in ipairs(hero.ability_list or {}) do
    local a = index[slot.ability]
    if a and not seen[a.id] then seen[a.id] = true; rows[#rows + 1] = { ability = a, slot = slot.slot_index } end
  end
  local rest = {}
  for _, a in ipairs(catalog.abilities) do
    if (a.source_hero_id == hero.source_hero_id or a.owner_file_hero_id == hero.source_hero_id) and not seen[a.id] then
      seen[a.id] = true; rest[#rest + 1] = a
    end
  end
  table.sort(rest, function(x, y) if x.path ~= y.path then return x.path < y.path end return x.line < y.line end)
  for _, a in ipairs(rest) do rows[#rows + 1] = { ability = a } end
  return rows
end

function M.render(catalog, hero_id)
  local hero
  for _, h in ipairs(catalog.heroes) do if h.source_hero_id == hero_id then hero = h end end
  assert(hero, "unknown source hero " .. tostring(hero_id))
  local src = catalog.source or {}
  local out = {}
  local function w(s) out[#out + 1] = s end
  w("# 來源資料：" .. hero_id .. " " .. (hero.name_zhtw or ""))
  w("")
  w("由 `scripts/hero_source_index.lua export` 從 source-catalog 產生；內容只有來源事實，沒有分析。")
  w("")
  w("- 倉庫：" .. tostring(src.repository or "https://github.com/damody/nobugg_official"))
  w("- commit：`" .. tostring(src.commit or "ca6ee8602473f6015a397cc121dac6baeb70b2e5") .. "`")
  w("- 陣營：" .. tostring(hero.faction) .. "；主檔：`" .. tostring(hero.main_file) .. "`；單位檔：`" .. tostring(hero.unit_file) .. "`")
  w("- 攻擊：" .. tostring(hero.attack_capabilities) .. "，射程 " .. tostring(hero.attack_range) .. "；主屬性 " .. tostring(hero.attribute_primary))
  w("")
  w("## 單位（" .. hero.unit.path .. ":" .. hero.unit.line .. "，raw_sha256 `" .. hero.unit.raw_sha256 .. "`）")
  w("")
  w(fence(hero.unit.raw))
  w("")
  w("## 技能")
  for _, row in ipairs(ordered(catalog, hero)) do
    local a = row.ability
    local tip = a.tooltip or {}
    w("")
    w("### " .. a.id .. (tip.name and tip.name.value and ("：" .. tip.name.value) or "")
      .. (row.slot and ("（單位技能欄 " .. row.slot .. "）") or "（不在單位技能清單）"))
    w("")
    w("- 位置：`" .. a.path .. ":" .. a.line .. "`，raw_sha256 `" .. a.raw_sha256 .. "`")
    w("- 版本：" .. tostring(a.variant))
    if tip.description then
      w("- tooltip（" .. tostring(tip.description.key) .. "，raw_sha256 `" .. tostring(tip.description.raw_sha256) .. "`）：")
      w("")
      w("  > " .. (a.description_plain or tip.description.value or ""):gsub("\r?\n", " "))
    else
      w("- tooltip：無")
    end
    local missing = {}
    for _, ref in ipairs(a.script_refs or {}) do
      if not ref.present_in_source then missing[#missing + 1] = ref.file .. "#" .. tostring(ref["function"]) end
    end
    if #missing > 0 then w("- 引用但來源 repo 沒有的 vscript：`" .. table.concat(missing, "`, `") .. "`") end
    w("")
    w(fence(a.raw))
  end
  w("")
  return table.concat(out, "\n")
end

-- Capability registry only (id, status, G package, evidence, limitation):
-- what an author may rely on, without any hero's requirement analysis.
function M.capabilities(requirements)
  local rows = {}
  for _, c in ipairs(requirements.capabilities) do rows[#rows + 1] = c end
  table.sort(rows, function(a, b) if a.status ~= b.status then return a.status > b.status end return a.id < b.id end)
  local out = { "# 能力登錄（D3 registry 匯出）", "",
    "由 `scripts/hero_source_index.lua capabilities` 從 `candidate-requirements.lua` 的 capabilities 產生；不含任何英雄的需求分析。", "",
    "| 能力 id | 狀態 | G 包 | 依據 | 限制 |", "|---|---|---|---|---|" }
  local function cell(v) return (tostring(v or "")):gsub("|", "／"):gsub("\r?\n", " ") end
  for _, c in ipairs(rows) do
    out[#out + 1] = string.format("| `%s` | %s | %s | %s | %s |", c.id, c.status, cell(c.g_package), cell(c.evidence), cell(c.limitation))
  end
  out[#out + 1] = ""
  return table.concat(out, "\n")
end

return M
