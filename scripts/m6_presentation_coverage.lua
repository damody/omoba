-- What an M6 batch's heroes will show in UE before they are listed (work line C): portrait and
-- ability icons (nobu-icons-v1.json, else the shared placeholder), every cue their documents use
-- (cue catalog kind -> generic placeholder; original art mapped or not), projectile cues, and the
-- statuses' names and descriptions (FOmTooltipCatalog matches a status by runtime_id, else by its own
-- id as the data-run executor applies it). Nothing here is per hero; the report lists missing art.
--
--   tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch N [--out FILE]
--   tools\lua\lua.exe scripts\m6_presentation_coverage.lua --missing-art   (every batch: m6-missing-art.md)
--
-- Inputs: docs/plans/2026-10-28-history-nobu-full-port/m6-waves.md (the batch's heroes),
-- target/site/source-catalog.json (gen_source_catalog), the nobu icon index and the cue catalog.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}
M.WAVES = "docs/plans/2026-10-28-history-nobu-full-port/m6-waves.md"
M.CATALOG = "target/site/source-catalog.json"
M.ICONS = "omfue/Content/Omoba/UI/nobu-icons-v1.json"
M.CUES = "docs/character_pipeline/presentation/cue-catalog.lua"

-- batch number -> ordered hero source ids ("| A01 德川家康 | ..." rows under "## 第 N 批")
function M.batch_heroes(waves_text, batch)
  local heroes, inside = {}, false
  for line in (waves_text .. "\n"):gmatch("([^\n]*)\n") do
    line = line:gsub("\r$", "")
    local n = line:match("^## 第 (%d+) 批")
    if n then inside = tonumber(n) == batch
    elseif line:match("^## ") then inside = false
    elseif inside then
      local id = line:match("^| (%u%d%d) ")
      if id then heroes[#heroes + 1] = id end
    end
  end
  return heroes
end

-- one hero's coverage from the three indexes
function M.hero(source_id, catalog, icons, cues)
  local r = { source_id = source_id, abilities = {}, cues = {}, statuses = {}, missing = {} }
  for _, h in ipairs(catalog.heroes) do
    if h.source_id == source_id then
      r.name = h.name and (h.name.zh_TW or h.name.en) or source_id
      r.playable = h.playable
    end
  end
  local by = {}
  for _, e in ipairs(icons.entries) do by[e.kind .. ":" .. (e.source_id or "")] = e end
  local portrait = by["hero:" .. source_id]
  r.portrait = portrait and portrait.status or "missing"
  if r.portrait ~= "original" then
    r.missing[#r.missing + 1] = "頭像（" .. (portrait and portrait.reason or "不在圖示索引") .. "）"
  end
  for _, a in ipairs(catalog.abilities) do
    if a.hero == source_id then
      local sid = a.v3 and a.v3.source_id
      local icon = sid and by["ability:" .. sid]
      local row = { id = a.id, icon = icon and icon.status or "missing" }
      r.abilities[#r.abilities + 1] = row
      if row.icon ~= "original" then
        r.missing[#r.missing + 1] = "技能圖示 `" .. a.id .. "`（" .. (icon and icon.reason or "不在圖示索引") .. "）"
      end
      for _, s in ipairs(a.v3 and a.v3.statuses or {}) do
        if s.visibility ~= "hidden" then
          local text = s.text and s.text.zh_TW
          r.statuses[#r.statuses + 1] = { id = s.id, described = text and text.name and text.body and true or false,
            matched_by = s.runtime_id and "runtime_id" or "own id" }
        end
      end
    end
  end
  local seen = {}
  for id, cue in pairs(cues.cues) do
    for _, use in ipairs(cue.uses or {}) do
      if use.doc == source_id and not seen[id] then
        seen[id] = true
        r.cues[#r.cues + 1] = { id = id, kind = cue.kind, art = cue.assets and #cue.assets > 0 }
      end
    end
  end
  table.sort(r.cues, function(x, y) return x.id < y.id end)
  return r
end

function M.render(batch, rows)
  local out = { "# M6 第 " .. batch .. " 批：UE 呈現準備",
    "",
    "由 `tools\\lua\\lua.exe scripts\\m6_presentation_coverage.lua --batch " .. batch .. "` 產生，不要手改。",
    "",
    "- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。",
    "- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。",
    "- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。",
    "- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。",
    "",
    "| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |",
    "|---|---|---|---|---|---|" }
  for _, r in ipairs(rows) do
    local icons, art, projectiles, described = 0, 0, 0, 0
    for _, a in ipairs(r.abilities) do if a.icon == "original" then icons = icons + 1 end end
    for _, c in ipairs(r.cues) do
      if c.art then art = art + 1 end
      if c.kind == "projectile" then projectiles = projectiles + 1 end
    end
    for _, s in ipairs(r.statuses) do if s.described then described = described + 1 end end
    out[#out + 1] = string.format("| %s %s | %s | %d／%d | %d／%d | %d | %d／%d |", r.source_id, r.name or "",
      r.portrait == "original" and "原作" or "佔位", icons, #r.abilities, art, #r.cues, projectiles, described, #r.statuses)
  end
  out[#out + 1] = ""
  out[#out + 1] = "## 仍缺的素材（目前用通用替代）"
  out[#out + 1] = ""
  local any = false
  for _, r in ipairs(rows) do
    local generic = {}
    for _, c in ipairs(r.cues) do if not c.art then generic[#generic + 1] = "`" .. c.id .. "`（" .. c.kind .. "）" end end
    if #r.missing > 0 or #generic > 0 then
      any = true
      out[#out + 1] = "### " .. r.source_id .. " " .. (r.name or "")
      out[#out + 1] = ""
      for _, m in ipairs(r.missing) do out[#out + 1] = "- " .. m end
      if #generic > 0 then out[#out + 1] = "- 沒有原作資產的 cue（通用佔位特效）：" .. table.concat(generic, "、") end
      out[#out + 1] = ""
    end
  end
  if not any then out[#out + 1] = "沒有。" end
  return table.concat(out, "\n") .. "\n"
end

-- One list of the art every batch still lacks, with what stands in for it now (for the user to decide).
function M.render_missing(batches)
  local out = { "# M6 仍缺的美術（第 1～" .. #batches .. " 批）",
    "",
    "由 `tools\\lua\\lua.exe scripts\\m6_presentation_coverage.lua --missing-art` 產生，不要手改。逐批細節在 `m6-batch<N>-presentation.md`。",
    "",
    "這份清單留給使用者決定要補哪些美術；目前不下載任何外部素材。替代方案都已經在 UE 生效：",
    "",
    "- **頭像**：共用佔位圖 `hero-placeholder-v1.png`，由 HUD sync 在英雄上架時自動補上。",
    "- **技能圖示**：共用佔位圖 `ability-placeholder-v1.png`，由 HUD sync 補上，建置時會印出警告。",
    "- **cue**：依 cue 目錄的 kind 播通用佔位特效（環、圓盤、連線；sound 不畫）。之後可以在 Editor 的 Om Presentation Cues 指定美術。",
    "", }
  local portraits, icons, cues = {}, {}, {}
  for _, b in ipairs(batches) do
    for _, r in ipairs(b.rows) do
      local who = r.source_id .. " " .. (r.name or "")
      for _, m in ipairs(r.missing) do
        local row = string.format("| %d | %s | %s |", b.batch, who, m)
        if m:find("^頭像") then portraits[#portraits + 1] = row else icons[#icons + 1] = row end
      end
      for _, c in ipairs(r.cues) do
        if not c.art then cues[#cues + 1] = string.format("| %d | %s | `%s` | %s |", b.batch, who, c.id, c.kind) end
      end
    end
  end
  local function section(title, header, rows)
    out[#out + 1] = "## " .. title .. "（" .. #rows .. "）"
    out[#out + 1] = ""
    if #rows == 0 then out[#out + 1] = "沒有。"; out[#out + 1] = ""; return end
    out[#out + 1] = header
    out[#out + 1] = header:gsub("[^|]+", "---")
    for _, row in ipairs(rows) do out[#out + 1] = row end
    out[#out + 1] = ""
  end
  section("頭像（替代：共用佔位圖）", "| 批 | 英雄 | 原因 |", portraits)
  section("技能圖示（替代：共用佔位圖）", "| 批 | 英雄 | 技能與原因 |", icons)
  section("沒有原作資產的 cue（替代：依 kind 的通用佔位特效）", "| 批 | 英雄 | cue | kind |", cues)
  return table.concat(out, "\n") .. "\n"
end

function M.main(args)
  if args[1] == "--missing-art" then
    local function read(rel) local f = assert(io.open(b.root .. "/" .. rel, "rb")); local s = f:read("a"); f:close(); return s end
    local waves = read(M.WAVES)
    local catalog, icons = json.decode(read(M.CATALOG)), json.decode(read(M.ICONS))
    local cues = dofile(b.root .. "/" .. M.CUES)
    local batches = {}
    for n = 1, 99 do
      local heroes = M.batch_heroes(waves, n)
      if #heroes == 0 then break end
      local rows = {}
      for _, h in ipairs(heroes) do rows[#rows + 1] = M.hero(h, catalog, icons, cues) end
      batches[#batches + 1] = { batch = n, rows = rows }
    end
    local out = "docs/plans/2026-10-28-history-nobu-full-port/m6-missing-art.md"
    local f = assert(io.open(b.root .. "/" .. out, "wb")); f:write(M.render_missing(batches)); f:close()
    print("wrote " .. out .. " (" .. #batches .. " batches)")
    return 0
  end
  local batch, out
  local i = 1
  while i <= #args do
    if args[i] == "--batch" then batch = tonumber(args[i + 1]); i = i + 1
    elseif args[i] == "--out" then out = args[i + 1]; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  assert(batch, "--batch N required")
  local function read(rel) local f = assert(io.open(b.root .. "/" .. rel, "rb")); local s = f:read("a"); f:close(); return s end
  local heroes = M.batch_heroes(read(M.WAVES), batch)
  assert(#heroes > 0, "batch " .. batch .. " has no heroes in " .. M.WAVES)
  local catalog, icons = json.decode(read(M.CATALOG)), json.decode(read(M.ICONS))
  local cues = dofile(b.root .. "/" .. M.CUES)
  local rows = {}
  for _, h in ipairs(heroes) do rows[#rows + 1] = M.hero(h, catalog, icons, cues) end
  out = out or ("docs/plans/2026-10-28-history-nobu-full-port/m6-batch" .. batch .. "-presentation.md")
  local f = assert(io.open(b.root .. "/" .. out, "wb")); f:write(M.render(batch, rows)); f:close()
  print("wrote " .. out .. " (" .. #rows .. " heroes)")
  return 0
end

if not M6_COVERAGE_AS_MODULE then os.exit(M.main(arg)) end
return M
