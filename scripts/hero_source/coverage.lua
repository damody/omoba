-- Month delivery coverage: for each listed kit, the source hero, every
-- ability's source id, action, status and capabilities, the kit status and
-- the acceptance status, read from the packages themselves (not restated).
-- Blind-test heroes appear with their package once one exists.
local M = {}

M.KITS = {
  { plan = "H01", source = "B01", package = "saika_magoichi" },
  { plan = "H02", source = "B04", package = "date_masamune" },
  { plan = "H03", source = "A34", package = "omatsu" },
  { plan = "H04", source = "C17", package = "oichi" },
  { plan = "H05", source = "A02", package = "inahime" },
  { plan = "H06", source = "B08", package = "asai_nagamasa" },
  { plan = "I1", source = "C15", blind = true },
  { plan = "I2", source = "B29", blind = true },
}

local function load_package_file(file)
  local f = io.open(file, "rb"); if not f then return nil end
  local text = f:read("a"); f:close()
  local env = { pairs = pairs, ipairs = ipairs }
  local chunk = assert(load(text, "@" .. file, "t", env))
  return chunk()({})
end

local function acceptance_status(dir)
  local f = io.open(dir .. "/acceptance.md", "rb"); if not f then return "missing" end
  local text = f:read("a"):gsub("\r\n", "\n"); f:close()
  return text:match("\nstatus:%s*([%w_]+)") or "unstated"
end

-- E170: an ability re-checked against the source scripts carries `parity` in source.lua;
-- one acknowledged as changed by a source switch carries `stale_review`.
M.PARITY_REVIEWED = "已對照原作"
M.PARITY_STALE = "待重審"
M.PARITY_NONE = "未對照"
function M.parity_status(source_entry)
  if not source_entry then return M.PARITY_NONE end
  if source_entry.stale_review then return M.PARITY_STALE end
  if source_entry.parity then return M.PARITY_REVIEWED end
  return M.PARITY_NONE
end

-- packages_root: docs/character_pipeline/packages; blind_packages: optional
-- map source id -> package dir name for blind heroes once authored.
function M.collect(packages_root, catalog, blind_packages)
  local names = {}
  for _, h in ipairs(catalog and catalog.heroes or {}) do names[h.source_hero_id] = h.name_zhtw end
  local rows = {}
  for _, kit in ipairs(M.KITS) do
    local pkg = kit.package or (blind_packages or {})[kit.source]
    local row = { plan = kit.plan, source = kit.source, name = names[kit.source], blind = kit.blind or nil, package = pkg }
    local dir = pkg and (packages_root .. "/" .. pkg)
    local abilities = dir and load_package_file(dir .. "/abilities.lua")
    if abilities then
      local caps = load_package_file(dir .. "/capabilities.lua") or {}
      local source = load_package_file(dir .. "/source.lua") or {}
      local source_by = {}
      for _, s in ipairs(source.abilities or {}) do source_by[s.source_ability_id] = s end
      row.kit_status = caps.kit_status
      row.acceptance = acceptance_status(dir)
      row.abilities = {}
      local reviewed = 0
      for _, ab in ipairs(abilities.abilities) do
        local list = {}
        for i, c in ipairs(ab.capabilities or {}) do list[i] = c end
        row.abilities[#row.abilities + 1] = { id = ab.id, source_ability_id = ab.source_ability_id, action = ab.action,
          status = ab.status, auto_learn = ab.auto_learn or nil, capabilities = list,
          parity = M.parity_status(source_by[ab.source_ability_id]) }
        if row.abilities[#row.abilities].parity == M.PARITY_REVIEWED then reviewed = reviewed + 1 end
      end
      row.parity = reviewed == #row.abilities and M.PARITY_REVIEWED or (reviewed .. "/" .. #row.abilities)
    else
      row.kit_status = "not_authored"
    end
    rows[#rows + 1] = row
  end
  return rows
end

function M.render(rows)
  local out = { "# 本月交付 kit 與盲測能力覆蓋", "",
    "由 `scripts/hero_source_index.lua coverage` 從各 package 產生，不手寫。來源目錄、版本與能力分類見 `source-catalog.lua`；能力狀態見 `../capability-registry.md`。", "",
    "「原作對照」：`已對照原作` 表示該招已依 history_nobu 原作 KV 與腳本逐項對照並移植（package `source.lua` 的 `parity`，對照卡在 package 的 `parity.md`）；`待重審` 表示來源改變後尚未重新對照（`stale_review`）。", "",
    "| 計畫 | 來源 | 名稱 | package | kit 狀態 | 驗收 | 原作對照 | 技能數 |", "|---|---|---|---|---|---|---|---|" }
  for _, r in ipairs(rows) do
    out[#out + 1] = string.format("| %s%s | %s | %s | %s | %s | %s | %s | %s |", r.plan, r.blind and "（盲測）" or "", r.source, r.name or "",
      r.package and ("`" .. r.package .. "`") or "—", r.kit_status or "", r.acceptance or "—", r.parity or "—", r.abilities and #r.abilities or 0)
  end
  for _, r in ipairs(rows) do
    if r.abilities then
      out[#out + 1] = ""
      out[#out + 1] = "## " .. r.plan .. " " .. r.source .. " " .. (r.name or "") .. "（`" .. r.package .. "`）"
      out[#out + 1] = ""
      out[#out + 1] = "| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |"
      out[#out + 1] = "|---|---|---|---|---|---|"
      for _, a in ipairs(r.abilities) do
        out[#out + 1] = string.format("| %s | `%s`%s | %s | %s | %s | %s |", a.action, a.id, a.auto_learn and "（auto_learn）" or "",
          a.source_ability_id, a.status, a.parity or M.PARITY_NONE, table.concat(a.capabilities, "、"))
      end
    end
  end
  out[#out + 1] = ""
  return table.concat(out, "\n")
end

return M
