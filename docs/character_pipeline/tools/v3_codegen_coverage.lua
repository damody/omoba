-- What v3_rust.lua does not generate yet, and in which order to add it (M3' ③; replaces
-- v3_ir_blockers.lua). Every hero and item document goes through the generator without writing;
-- each entry's blockers are the forms the generator does not lower or the ability_kit components it
-- has no call for. Ranked like ir-blockers.md: the listed heroes first (M5: the shipped ten,
-- including B01 and C17, which still run the IR), then the M6 waves (m6-waves.md); inside a stage,
-- the hero that unlocks the most heroes per new form goes first.
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua [--out FILE.md]
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local M = {}

local function keys(t) local o = {} for k in pairs(t or {}) do o[#o + 1] = k end table.sort(o) return o end

-- Who adds a form: the generator (expression and selector forms it can lower over the kit as it is)
-- or ability_kit (a host component or a kit call that does not exist yet). Approximate: read it as
-- "start here", the two lines agree the split per form.
function M.owner_of(feature)
  local head = feature:match("^routine [%w_]+: (.*)$") or feature
  if head:match("^fn:") or head:match("^attr:") or head:match("^token:") or head:match("^expr") or head:match("^op:")
    or head:match("^selector%.") or head:match("^target%.") or head:match("^value") or head:match("^name:")
    or head == "event:on_upgrade" then
    return "產生器"
  end
  return "ability_kit"
end

-- When the generated code of a supported entry returns Err (execute -> RErr): the cast checks and
-- the kit calls that refuse a world the data did not foresee. Read from the generated text.
M.REFUSALS = {
  { "Cx::cast(", "施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR）" },
  { "create_unit(", "召喚：附近放不下、lifetime 不在 0～600 秒" },
  { "makes one unit at a time", "召喚：count 不是 1" },
  { "cx.delay(", "延遲：時間超出 host delayed_effect 的上限" },
  { "cx.reveal(", "視野：半徑或位置不合法" },
  { "cx.tether(", "光鍊：host tether 規格不合法" },
  { "cx.ally_restore(", "友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離" },
  { "ability_kit::div(", "運算式除以零" },
  { "cx.pos(", "讀位置的單位已不在世界中" },
  { "cx.target_point()", "需要地點，但施放沒有地點也沒有單位目標" },
}
function M.refusals(text)
  local out = {}
  for _, r in ipairs(M.REFUSALS) do if text:find(r[1], 1, true) then out[#out + 1] = r[2] end end
  return out
end

-- reports of every document: { heroes = { {source_id, package, listed, entries} }, items = { entries } }
function M.scan(root)
  local prev = rawget(_G, "V3_RUST_AS_MODULE")
  V3_RUST_AS_MODULE = true
  local R = dofile(tools_dir .. "/v3_rust.lua")
  V3_RUST_AS_MODULE = prev
  local heroes = R.heroes(root, "all")
  local reports, problems = R.run(root, nil, heroes, { write = false, keep_files = true })
  local out = { heroes = {}, items = {}, problems = problems }
  for _, r in ipairs(reports) do
    -- runtime refusals of the generated entries (the cast fails with RErr; the host rolls back)
    for _, e in ipairs(r.entries) do
      if #e.blockers == 0 then e.refusals = M.refusals(r.files["src/generated/" .. e.id .. ".rs"] or "") end
    end
    out.heroes[#out.heroes + 1] = { source_id = r.id, package = r.package, listed = r.listed, entries = r.entries }
  end
  -- item documents: the generator lowers their entries the same way (items.dll, M7)
  local lfs = require("lfs")
  local P = R.load(root)
  local dir = root .. "/docs/character_pipeline/items"
  local rules = P.rules()
  for name in lfs.dir(dir) do
    if name:match("%.lua$") then
      local doc = P.load_file(dir .. "/" .. name)
      if doc and #P.validate_doc(doc) == 0 then
        local ok, files_or_err, report = pcall(R.generate, doc, { root = root, rules = rules, package = name:gsub("%.lua$", "") })
        if ok then for _, e in ipairs(report.entries) do out.items[#out.items + 1] = e end
        else out.problems[#out.problems + 1] = name .. ": " .. tostring(files_or_err) end
      end
    end
  end
  table.sort(out.items, function(a, b) return a.id < b.id end)
  return out
end

-- the forms of an entry for ranking: a routine's blockers count as the form itself
local function features(e)
  local seen, out = {}, {}
  for _, b in ipairs(e.blockers) do
    local f = b:gsub("^routine [%w_]+: ", "")
    if not seen[f] then seen[f] = true; out[#out + 1] = f end
  end
  return out
end

function M.build(scan, waves)
  local by_sid = {}
  for _, h in ipairs(scan.heroes) do by_sid[h.source_id] = h end
  local function hero_blockers(sid)
    local h = by_sid[sid]
    local out = {}
    for _, e in ipairs(h and h.entries or {}) do for _, f in ipairs(features(e)) do out[f] = (out[f] or 0) + 1 end end
    return out
  end
  local stages = {}
  local m5 = { name = "M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）", heroes = {} }
  for _, h in ipairs(scan.heroes) do if h.listed then m5.heroes[#m5.heroes + 1] = h.source_id end end
  table.sort(m5.heroes)
  stages[1] = m5
  for _, w in ipairs(waves or {}) do
    local s = { name = "M6 第 " .. w.index .. " 批", heroes = {} }
    for _, u in ipairs(w.heroes) do s.heroes[#s.heroes + 1] = u.id end
    stages[#stages + 1] = s
  end
  local totals = {}
  local function total(f) local t = totals[f] or { feature = f, abilities = 0, items = 0, heroes = {} }; totals[f] = t; return t end
  for _, h in ipairs(scan.heroes) do
    for _, e in ipairs(h.entries) do
      for _, f in ipairs(features(e)) do local t = total(f); t.abilities = t.abilities + 1; t.heroes[h.source_id] = true end
    end
  end
  for _, e in ipairs(scan.items) do for _, f in ipairs(features(e)) do local t = total(f); t.items = t.items + 1 end end
  local have = {}
  for _, stage in ipairs(stages) do
    local function missing(sid) local m = {} for f in pairs(hero_blockers(sid)) do if not have[f] then m[#m + 1] = f end end table.sort(m) return m end
    local function unlocked() local n = 0 for _, sid in ipairs(stage.heroes) do if #missing(sid) == 0 then n = n + 1 end end return n end
    stage.ready_at_start = unlocked()
    stage.steps = {}
    while true do
      local best, best_score, best_m
      for _, sid in ipairs(stage.heroes) do
        local m = missing(sid)
        if #m > 0 then
          local trial = {} for _, f in ipairs(m) do trial[f] = true end
          local gain = 0
          for _, o in ipairs(stage.heroes) do
            local ok = true
            for f in pairs(hero_blockers(o)) do if not have[f] and not trial[f] then ok = false end end
            if ok and #missing(o) > 0 then gain = gain + 1 end
          end
          local score = gain / #m
          if not best or score > best_score or (score == best_score and #m < #best_m) then best, best_score, best_m = sid, score, m end
        end
      end
      if not best then break end
      for _, f in ipairs(best_m) do
        have[f] = true
        stage.steps[#stage.steps + 1] = { feature = f, for_hero = best, unlocked = unlocked() }
      end
    end
    stage.unlocked_at_end = unlocked()
  end
  return { scan = scan, stages = stages, totals = totals }
end

function M.render(r)
  local out = {}
  local function w(s) out[#out + 1] = s end
  local n, gen, ni, igen, heroes_done = 0, 0, 0, 0, 0
  for _, h in ipairs(r.scan.heroes) do
    local all = true
    for _, e in ipairs(h.entries) do n = n + 1; if #e.blockers == 0 then gen = gen + 1 else all = false end end
    if all then heroes_done = heroes_done + 1 end
  end
  for _, e in ipairs(r.scan.items) do ni = ni + 1; if #e.blockers == 0 then igen = igen + 1 end end
  -- D7 / m3 §2.4: a form is shared when >= 2 entries (abilities and items) use it, a hook
  -- candidate when only 1 does. A routine's blocker counts as the routine's mechanism.
  local function base_of(feature) return (feature:gsub("^routine [%w_]+: ", "")) end
  local users = {}
  local function count_entry(e, owner)
    local seen = {}
    for _, b in ipairs(e.blockers) do
      local k = base_of(b)
      if not seen[k] then seen[k] = true; users[k] = users[k] or {}; table.insert(users[k], { id = e.id, owner = owner }) end
    end
  end
  for _, h in ipairs(r.scan.heroes) do for _, e in ipairs(h.entries) do count_entry(e, h.source_id .. " " .. h.package) end end
  for _, e in ipairs(r.scan.items) do count_entry(e, "道具") end
  local function class_of(feature)
    local u = users[base_of(feature)]
    return (u and #u >= 2) and "共用" or "hook 候選"
  end
  w("# 產生器還沒產生的寫法與實作優先序")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。")
  w("")
  w(("- 英雄技能：%d 招中 %d 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：%d / %d 位。"):format(n, gen, heroes_done, #r.scan.heroes))
  w(("- 道具（items.dll，M7）：%d 件中 %d 件產生成 Rust。"):format(ni, igen))
  w("- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。")
  w("- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。")
  w("- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。")
  w("")
  w("## 1. 實作順序")
  w("")
  local idx = 0
  for _, stage in ipairs(r.stages) do
    w("### " .. stage.name .. "（" .. #stage.heroes .. " 位）")
    w("")
    w(("開始時已全部產生：%d 位；做完本批的項目後：%d 位。"):format(stage.ready_at_start, stage.unlocked_at_end))
    w("")
    if #stage.steps == 0 then w("（本批不需要新的支援）") else
      w("| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |")
      w("|---|---|---|---|---|---|---|")
      for _, s in ipairs(stage.steps) do
        idx = idx + 1
        local t = r.totals[s.feature] or { abilities = 0, items = 0 }
        w(("| %d | `%s` | %s | %s | %s | %d 招、%d 件道具 | %d / %d |"):format(idx, s.feature, M.owner_of(s.feature), class_of(s.feature), s.for_hero,
          t.abilities, t.items, s.unlocked, #stage.heroes))
      end
    end
    w("")
  end
  w("## 2. 每位英雄、每招還缺什麼")
  w("")
  local heroes = {}
  for _, h in ipairs(r.scan.heroes) do heroes[#heroes + 1] = h end
  table.sort(heroes, function(a, b) if a.listed ~= b.listed then return a.listed end return a.source_id < b.source_id end)
  for _, h in ipairs(heroes) do
    local done = 0
    for _, e in ipairs(h.entries) do if #e.blockers == 0 then done = done + 1 end end
    w(("### %s `%s`（%d / %d 招）%s"):format(h.source_id, h.package, done, #h.entries, h.listed and "　已上架" or ""))
    w("")
    w("| 招式 | 還缺 |")
    w("|---|---|")
    for _, e in ipairs(h.entries) do
      w(("| `%s` | %s |"):format(e.id, #e.blockers == 0 and "✓ 已產生" or ("`" .. table.concat(e.blockers, "`、`") .. "`")))
    end
    w("")
  end
  w("## 3. 已產生的招式在執行期回 RErr 的情況")
  w("")
  w("產生的程式碼遇到資料沒預期的世界狀態時，施放回 `RErr`（host 回滾成本）。下表是已產生的招式裡各種情況出現的招數；未產生的招式是 stub，一律回 `RErr` 並列出 §2 的缺項。")
  w("")
  w("| 情況 | 招數 |")
  w("|---|---|")
  local seen = {}
  for _, h in ipairs(r.scan.heroes) do
    for _, e in ipairs(h.entries) do for _, x in ipairs(e.refusals or {}) do seen[x] = (seen[x] or 0) + 1 end end
  end
  for _, rr in ipairs(M.REFUSALS) do if seen[rr[2]] then w(("| %s | %d |"):format(rr[2], seen[rr[2]])) end end
  -- generated entries whose document does something the host does not (v3_rust.lua Gen:gap)
  w("")
  w("### 已產生但 host 沒有做到的地方")
  w("")
  w("產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。")
  w("")
  w("| 招式 | 沒有做到 |")
  w("|---|---|")
  local any = false
  for _, h in ipairs(r.scan.heroes) do
    for _, e in ipairs(h.entries) do
      if e.gaps and #e.gaps > 0 then any = true; w(("| `%s` | %s |"):format(e.id, table.concat(e.gaps, "；"))) end
    end
  end
  for _, e in ipairs(r.scan.items) do
    if e.gaps and #e.gaps > 0 then any = true; w(("| `%s` | %s |"):format(e.id, table.concat(e.gaps, "；"))) end
  end
  if not any then w("| （無） | |") end
  w("")
  w("## 4. 每項寫法擋住多少")
  w("")
  w("| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |")
  w("|---|---|---|---|---|---|")
  local rows = {}
  for _, t in pairs(r.totals) do local hc = 0 for _ in pairs(t.heroes) do hc = hc + 1 end t.hero_count = hc; rows[#rows + 1] = t end
  table.sort(rows, function(a, b) if a.hero_count ~= b.hero_count then return a.hero_count > b.hero_count end if a.abilities ~= b.abilities then return a.abilities > b.abilities end return a.feature < b.feature end)
  for _, t in ipairs(rows) do w(("| `%s` | %s | %s | %d | %d | %d |"):format(t.feature, M.owner_of(t.feature), class_of(t.feature), t.abilities, t.items, t.hero_count)) end
  w("")
  w("## 5. hook 候選")
  w("")
  w("只有 1 招（英雄技能與道具合計）用到的寫法。這些招式在 v3 標 `hook = \"<方法名>\"`、寫明行為描述與預期結果，由產生的 trait 手寫實作（`src/hooks/<招式>.rs`）；第 2 招也用到時改成共用、刪掉 hook。")
  w("")
  w("| 寫法 | 招式 | 所屬 |")
  w("|---|---|---|")
  local cands = {}
  for k, u in pairs(users) do if #u == 1 then cands[#cands + 1] = { feature = k, id = u[1].id, owner = u[1].owner } end end
  table.sort(cands, function(a, b) if a.owner ~= b.owner then return a.owner < b.owner end if a.id ~= b.id then return a.id < b.id end return a.feature < b.feature end)
  for _, c in ipairs(cands) do w(("| `%s` | `%s` | %s |"):format(c.feature, c.id, c.owner)) end
  if #cands == 0 then w("| （無） | | |") end
  w("")
  local by_entry, order = {}, {}
  for _, c in ipairs(cands) do
    if not by_entry[c.id] then by_entry[c.id] = { owner = c.owner, n = 0 }; order[#order + 1] = c.id end
    by_entry[c.id].n = by_entry[c.id].n + 1
  end
  w(("共 %d 種寫法、%d 招。"):format(#cands, #order))
  w("")
  if #r.scan.problems > 0 then
    w("## 6. 產生器無法處理的文件")
    w("")
    for _, p in ipairs(r.scan.problems) do w("- " .. p) end
    w("")
  end
  w("## 重跑")
  w("")
  w("```")
  w("tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_codegen_coverage.lua")
  w("```")
  w("")
  return table.concat(out, "\n")
end

function M.main(argv)
  local root = tools_dir:gsub("\\", "/"):match("^(.-)/?docs/character_pipeline/tools$")
  if not root or root == "" then root = "." end
  local opts = { out = root .. "/docs/plans/2026-10-28-history-nobu-full-port/codegen-coverage.md" }
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--out" then opts.out = argv[i + 1]; i = i + 1 else error("unknown argument " .. a) end
    i = i + 1
  end
  local prev = rawget(_G, "V3_WAVES_AS_MODULE")
  V3_WAVES_AS_MODULE = true
  local W = dofile(tools_dir .. "/v3_waves.lua")
  V3_WAVES_AS_MODULE = prev
  local scan = M.scan(root)
  local r = M.build(scan, W.build(root).waves)
  local o = assert(io.open(opts.out, "wb")); o:write(M.render(r), "\n"); o:close()
  local n, gen = 0, 0
  for _, h in ipairs(scan.heroes) do for _, e in ipairs(h.entries) do n = n + 1; if #e.blockers == 0 then gen = gen + 1 end end end
  print(("wrote %s: %d hero entries, %d generated, %d item entries, %d problems"):format(opts.out, n, gen, #scan.items, #scan.problems))
  for _, p in ipairs(scan.problems) do io.stderr:write(p, "\n") end
  return 0
end

if rawget(_G, "V3_CODEGEN_COVERAGE_AS_MODULE") then return M end
os.exit(M.main(arg))
