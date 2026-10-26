-- D3 requirement tables: validation against the source catalog and rendering.
--
-- A requirements file lists, per candidate hero, the slot mapping and for each
-- source skill the mechanics it needs. Every mechanic names a capability from
-- the file's capability registry; its status comes from that registry unless
-- an explicit override with a reason is given. Skill and hero statuses are
-- derived (worst wins), so the data cannot claim `supported` for a kit whose
-- mechanics are not. Each reviewed skill pins the catalog raw_sha256 it was
-- read against, so a changed source makes the review stale instead of silent.
local M = {}

M.SCHEMA = "omoba.hero_source_requirements/1"
M.STATUSES = { "supported", "ambiguous", "needs_framework", "deferred" }
local RANK = { supported = 1, ambiguous = 2, needs_framework = 3, deferred = 4 }
M.RANK = RANK
M.ROSTERS = { delivery = true, blind = true, backup = true }
M.G_ORDER = { "G-1", "G-2", "G-3", "G-4", "backlog" }
M.G_PACKAGES = { ["G-1"] = true, ["G-2"] = true, ["G-3"] = true, ["G-4"] = true, backlog = true }
-- The MOBA loadout has four slots and passives occupy one of them, so every
-- source skill maps to Q/W/E/R or to an extra slot (which needs the
-- extra_action_slot capability).
local ACTION_SLOTS = { Q = true, W = true, E = true, R = true, extra1 = true, extra2 = true, extra3 = true }

local function worst(a, b)
  if not a then return b end
  if not b then return a end
  return RANK[a] >= RANK[b] and a or b
end
M.worst = worst

-- Returns derived = { heroes = { [id] = { status, abilities = { [id] = status } } } }, problems
function M.validate(req, catalog)
  local problems = {}
  local function bad(msg) problems[#problems + 1] = msg end
  if req.schema ~= M.SCHEMA then bad("schema must be " .. M.SCHEMA) end
  if req.catalog_commit ~= catalog.source.commit then
    bad("requirements were reviewed against " .. tostring(req.catalog_commit) .. ", catalog is " .. catalog.source.commit)
  end
  local caps = {}
  for _, c in ipairs(req.capabilities or {}) do
    if caps[c.id] then bad("duplicate capability " .. c.id) end
    if not RANK[c.status] then bad("capability " .. tostring(c.id) .. " has invalid status " .. tostring(c.status)) end
    if not c.evidence or c.evidence == "" then bad("capability " .. tostring(c.id) .. " needs evidence") end
    if c.status ~= "supported" and not M.G_PACKAGES[c.g_package] then
      bad("capability " .. tostring(c.id) .. " is " .. tostring(c.status) .. " and needs g_package (" .. table.concat(M.G_ORDER, "/") .. ")")
    end
    caps[c.id] = c
  end
  local abilities, heroes = {}, {}
  for _, a in ipairs(catalog.abilities) do abilities[a.id] = a end
  for _, h in ipairs(catalog.heroes) do heroes[h.source_hero_id] = h end

  local derived = { heroes = {} }
  local seen_hero = {}
  for _, h in ipairs(req.heroes or {}) do
    local hid = h.source_hero_id
    local label = "hero " .. tostring(hid)
    if seen_hero[hid] then bad(label .. " listed twice") end
    seen_hero[hid] = true
    local src = heroes[hid]
    if not src then bad(label .. " not in catalog") end
    if not M.ROSTERS[h.roster] then bad(label .. " has invalid roster " .. tostring(h.roster)) end
    local d = { status = nil, abilities = {} }
    derived.heroes[hid] = d
    local reviewed = {}
    for _, ab in ipairs(h.abilities or {}) do
      local alabel = label .. " ability " .. tostring(ab.id)
      local a = abilities[ab.id]
      if not a then
        bad(alabel .. " not in catalog")
      else
        if a.source_hero_id ~= hid then bad(alabel .. " belongs to " .. tostring(a.source_hero_id)) end
        if a.variant ~= h.variant then bad(alabel .. " is variant " .. a.variant .. ", kit is " .. tostring(h.variant)) end
        if ab.reviewed_sha256 ~= a.raw_sha256 then bad(alabel .. " review is stale (raw_sha256 changed)") end
      end
      if ab.read ~= true then bad(alabel .. " must be read before classification") end
      local status
      if #(ab.mechanics or {}) == 0 then bad(alabel .. " has no mechanics") end
      for _, m in ipairs(ab.mechanics or {}) do
        local cap = caps[m.capability]
        if not cap then
          bad(alabel .. " uses unknown capability " .. tostring(m.capability))
        else
          local ms = cap.status
          if m.status_override then
            if not RANK[m.status_override] then bad(alabel .. " override status invalid") end
            if not m.reason or m.reason == "" then bad(alabel .. " override of " .. m.capability .. " needs a reason") end
            ms = m.status_override
          end
          status = worst(status, ms)
        end
      end
      if ab.status ~= nil and ab.status ~= status then
        bad(alabel .. " declares " .. tostring(ab.status) .. " but mechanics derive " .. tostring(status))
      end
      d.abilities[ab.id] = status
      d.status = worst(d.status, status)
      reviewed[ab.id] = true
    end
    -- Slot mapping covers every skill in the source unit ability list.
    local actions = {}
    for _, s in ipairs(h.slots or {}) do
      if not reviewed[s.ability] then bad(label .. " slot " .. tostring(s.ability) .. " has no reviewed ability") end
      if not ACTION_SLOTS[s.action] then bad(label .. " slot " .. tostring(s.ability) .. " action invalid: " .. tostring(s.action)) end
      if actions[s.action] then bad(label .. " action " .. tostring(s.action) .. " used twice") end
      actions[s.action] = true
      if type(s.action) == "string" and s.action:match("^extra") then
        d.needs_extra_slot = true
        local uses = false
        for _, ab in ipairs(h.abilities or {}) do
          if ab.id == s.ability then
            for _, m in ipairs(ab.mechanics or {}) do if m.capability == "extra_action_slot" then uses = true end end
          end
        end
        if not uses then bad(label .. " slot " .. s.ability .. " uses " .. s.action .. " without an extra_action_slot mechanic") end
      end
    end
    if src then
      local mapped = {}
      for _, s in ipairs(h.slots or {}) do mapped[s.ability] = true end
      for _, e in ipairs(src.ability_list or {}) do
        local a = abilities[e.ability]
        if a and (a.kind == "skill" or a.kind == "sub_ability") and not mapped[e.ability] then
          bad(label .. " source slot " .. e.ability .. " is not mapped")
        end
      end
    end
    for _, ab in ipairs(h.abilities or {}) do
      local a = abilities[ab.id]
      local in_slots = false
      for _, s in ipairs(h.slots or {}) do if s.ability == ab.id then in_slots = true end end
      if a and not in_slots and not ab.sub_of then bad(label .. " ability " .. ab.id .. " needs a slot or sub_of") end
    end
    -- Non-blind kits need a bot_usage draft for every castable slot; blind
    -- tests only get requirement tables.
    local by_id = {}
    for _, ab in ipairs(h.abilities or {}) do by_id[ab.id] = ab end
    for _, s in ipairs(h.slots or {}) do
      local ab = by_id[s.ability]
      if h.roster ~= "blind" and ab and not ab.bot_usage and not ab.passive then
        bad(label .. " ability " .. s.ability .. " needs bot_usage")
      end
      if h.roster == "blind" and ab and ab.bot_usage then bad(label .. " is blind; do not author bot_usage") end
    end
    d.g_packages = {}
    local g_seen = {}
    for _, ab in ipairs(h.abilities or {}) do
      for _, m in ipairs(ab.mechanics or {}) do
        local cap = caps[m.capability]
        local st = m.status_override or (cap and cap.status)
        if cap and st ~= "supported" and cap.g_package then
          local key = cap.g_package .. ":" .. cap.id
          if not g_seen[key] then g_seen[key] = true; d.g_packages[#d.g_packages + 1] = { g = cap.g_package, capability = cap.id, status = st } end
        end
      end
    end
    table.sort(d.g_packages, function(a, b) if a.g ~= b.g then return a.g < b.g end return a.capability < b.capability end)
    if h.status ~= nil and h.status ~= d.status then
      bad(label .. " declares " .. tostring(h.status) .. " but abilities derive " .. tostring(d.status))
    end
  end
  return derived, problems
end

local function esc(s) return (tostring(s or ""):gsub("|", "\\|"):gsub("\n", " ")) end

function M.render(req, catalog, derived)
  local abilities = {}
  for _, a in ipairs(catalog.abilities) do abilities[a.id] = a end
  local caps = {}
  for _, c in ipairs(req.capabilities) do caps[c.id] = c end
  local L = {}
  local function w(s) L[#L + 1] = s or "" end
  w("# D3 候選能力需求表")
  w()
  w("由 `tools\\lua\\lua.exe scripts\\hero_source_index.lua requirements` 從 `candidate-requirements.lua` 生成，不要手改。")
  w("技能與英雄狀態由機制推導（最差者為準：supported < ambiguous < needs_framework < deferred）；每招釘住閱讀時的 `raw_sha256`。")
  w()
  w("## 總覽")
  w()
  w("| 來源 | 名稱 | 名單 | 計畫槽 | 定位 | 狀態 | 額外槽 |")
  w("|---|---|---|---|---|---|---|")
  local names = {}
  for _, h in ipairs(catalog.heroes) do names[h.source_hero_id] = h.name_zhtw end
  for _, h in ipairs(req.heroes) do
    local d = derived.heroes[h.source_hero_id]
    w(string.format("| %s | %s | %s | %s | %s | `%s` | %s |", h.source_hero_id, esc(names[h.source_hero_id]), h.roster,
      esc(h.plan_slot or "—"), esc(h.role), tostring(d.status), d.needs_extra_slot and "需要" or "—"))
  end
  w()
  w("## 能力登錄（引擎現況）")
  w()
  w("| capability | 狀態 | G 包 | 證據 | 限制 |")
  w("|---|---|---|---|---|")
  for _, c in ipairs(req.capabilities) do
    w(string.format("| `%s` | `%s` | %s | %s | %s |", c.id, c.status, c.g_package or "—", esc(c.evidence), esc(c.limitation or "")))
  end
  w()
  w("## 各 G 包需要的能力（依名單）")
  w()
  w("| G 包 | capability | 狀態 | 需要的英雄 |")
  w("|---|---|---|---|")
  local need = {}
  for _, h in ipairs(req.heroes) do
    for _, g in ipairs(derived.heroes[h.source_hero_id].g_packages) do
      local key = g.g .. "\0" .. g.capability
      need[key] = need[key] or { g = g.g, capability = g.capability, status = g.status, heroes = {} }
      need[key].heroes[#need[key].heroes + 1] = h.source_hero_id .. "（" .. h.roster .. "）"
    end
  end
  local keys = {}
  for k in pairs(need) do keys[#keys + 1] = k end
  table.sort(keys)
  for _, k in ipairs(keys) do
    local n = need[k]
    w(string.format("| %s | `%s` | `%s` | %s |", n.g, n.capability, n.status, table.concat(n.heroes, "、")))
  end
  w()
  for _, h in ipairs(req.heroes) do
    local d = derived.heroes[h.source_hero_id]
    w(string.format("## %s %s（%s，%s）— `%s`", h.source_hero_id, esc(names[h.source_hero_id]), h.roster, esc(h.role), tostring(d.status)))
    w()
    if h.notes then w(esc(h.notes)); w() end
    if #d.g_packages > 0 then
      local parts = {}
      for _, g in ipairs(d.g_packages) do parts[#parts + 1] = g.g .. " `" .. g.capability .. "`（" .. g.status .. "）" end
      w("需要的 G 包能力：" .. table.concat(parts, "、") .. "。")
      w()
    end
    w("| 來源鍵 | 技能 | 本遊戲 action | 狀態 |")
    w("|---|---|---|---|")
    for _, s in ipairs(h.slots) do
      local a = abilities[s.ability]
      w(string.format("| %s | `%s` %s | %s | `%s` |", esc(s.source_key or (a and a.slot) or ""), s.ability,
        esc(a and a.display_name), s.action, tostring(d.abilities[s.ability])))
    end
    w()
    for _, ab in ipairs(h.abilities) do
      local a = abilities[ab.id]
      w(string.format("### `%s` %s — `%s`", ab.id, esc(a and a.display_name), tostring(d.abilities[ab.id])))
      w()
      w("| 機制 | capability | 狀態 | 說明 |")
      w("|---|---|---|---|")
      for _, m in ipairs(ab.mechanics) do
        local st = m.status_override or (caps[m.capability] and caps[m.capability].status)
        local note = m.note or ""
        if m.status_override then note = note .. "（覆寫：" .. m.reason .. "）" end
        w(string.format("| %s | `%s` | `%s` | %s |", esc(m.what), m.capability, tostring(st), esc(note)))
      end
      if ab.kv_text_conflicts and #ab.kv_text_conflicts > 0 then
        w()
        for _, c in ipairs(ab.kv_text_conflicts) do w("- KV／描述不一致：" .. esc(c)) end
      end
      if ab.assumptions and #ab.assumptions > 0 then
        w()
        for _, c in ipairs(ab.assumptions) do w("- 假設（assumed）：" .. esc(c)) end
      end
      if ab.bot_usage then
        local b = ab.bot_usage
        w()
        w(string.format("- bot_usage（草案）：目標 `%s`；時機：%s%s", tostring(b.target), esc(b.when),
          b.avoid and ("；不要：" .. esc(b.avoid)) or ""))
      end
      w()
    end
  end
  return table.concat(L, "\n") .. "\n"
end

return M
