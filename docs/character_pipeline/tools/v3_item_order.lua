-- Item unlock order for M7 (history-nobu full port, work line A): which shop items the behavior IR
-- interpreter can run today, which only need stat rows, and which interpreter support unlocks the
-- most items next. Uses the same scan as v3_ir_blockers.lua (ir.rs load rules).
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_item_order.lua [--content FILE] [--out FILE.md]
--
-- Reads target/v3-content/v3-content.json (scripts/gen_v3_content.lua) and the shelf
-- (templates/item_shop_v3.lua: the items the native item engine already sells).
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
package.path = tools_dir .. "/../../../scripts/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")
V3_IR_BLOCKERS_AS_MODULE = true
local B = dofile(tools_dir .. "/v3_ir_blockers.lua")
V3_IR_BLOCKERS_AS_MODULE = nil
local M = {}

local function keys(t) local o = {} for k in pairs(t or {}) do o[#o + 1] = k end table.sort(o) return o end

-- a pure stat item: no active, no statuses, the passive only carries stats
function M.pure_stats(ir)
  if ir.entries and next(ir.entries) then return false end
  if ir.statuses and #ir.statuses > 0 then return false end
  if ir.hooks and #ir.hooks > 0 then return false end
  for _, k in ipairs(keys(ir.passive)) do if k ~= "stats" and k ~= "reveal" then return false end end
  return ir.passive ~= nil and ir.passive.stats ~= nil
end

function M.build(content, shelf)
  local items = {}
  for id, blk in pairs(content.items or {}) do
    local name = blk.text and blk.text.zh_TW and blk.text.zh_TW.name or id
    local row = { id = id, name = name, shelf = shelf[(id:gsub("^item_", ""))] ~= nil, blockers = {} }
    if blk.ir then
      row.has_ir = true
      row.pure = M.pure_stats(blk.ir)
      for f in pairs(B.scan(blk.ir)) do row.blockers[#row.blockers + 1] = f end
      table.sort(row.blockers)
    end
    items[#items + 1] = row
  end
  table.sort(items, function(a, b2) return a.id < b2.id end)
  -- greedy: the feature that completes the most items next
  local done, order, supported = {}, {}, {}
  local function open(row)
    local n = 0
    for _, f in ipairs(row.blockers) do if not supported[f] then n = n + 1 end end
    return n
  end
  for _, row in ipairs(items) do if row.has_ir and #row.blockers == 0 then done[row.id] = "now" end end
  while true do
    local best, best_done, best_near = nil, -1, -1
    local cand = {}
    for _, row in ipairs(items) do
      if row.has_ir and not done[row.id] then
        for _, f in ipairs(row.blockers) do if not supported[f] then cand[f] = true end end
      end
    end
    for _, f in ipairs(keys(cand)) do
      supported[f] = true
      local completes, near = 0, 0
      for _, row in ipairs(items) do
        if row.has_ir and not done[row.id] then
          local n = open(row)
          if n == 0 then completes = completes + 1 elseif n == 1 then near = near + 1 end
        end
      end
      supported[f] = nil
      if completes > best_done or (completes == best_done and near > best_near) then best, best_done, best_near = f, completes, near end
    end
    if not best then break end
    supported[best] = true
    local unlocked = {}
    for _, row in ipairs(items) do
      if row.has_ir and not done[row.id] and open(row) == 0 then done[row.id] = #order + 1; unlocked[#unlocked + 1] = row end
    end
    order[#order + 1] = { feature = best, unlocked = unlocked }
  end
  return items, order
end

function M.render(items, order)
  local o = {}
  local function w(s) o[#o + 1] = s end
  local n, shelf, now, pure, noir = #items, 0, 0, 0, 0
  for _, r in ipairs(items) do
    if r.shelf then shelf = shelf + 1 end
    if r.has_ir and #r.blockers == 0 then now = now + 1 end
    if r.pure then pure = pure + 1 end
    if not r.has_ir then noir = noir + 1 end
  end
  w("# M7 道具解鎖順序")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_item_order.lua` 產生，不要手改。掃描規則與 [ir-blockers.md](ir-blockers.md) 相同（ir.rs 的載入規則）；B 每放回一批就和 ir-blockers 一起重跑。")
  w("")
  w(("- 道具共 %d 件：%d 件已由原生道具引擎上架（`templates/item_shop_v3.lua`，價錢、配方與屬性都由 v3 values 產生），%d 件沒有 IR（deferred 或沒有行為）。"):format(n, shelf, noir))
  w(("- 直譯器今天就能完整載入：%d 件。只有屬性加成（passive.stats，沒有主動、狀態與觸發）的道具：%d 件。"):format(now, pure))
  w("")
  w("## 1. 實作順序")
  w("")
  w("每一步選「多支援這一項就能完整載入最多道具」的寫法；同分時選讓最多道具只差一項的。")
  w("")
  w("| # | 要支援的寫法 | 這一步解鎖 | 累計 |")
  w("|---|---|---|---|")
  local total = now
  for i, step in ipairs(order) do
    total = total + #step.unlocked
    local names = {}
    for _, r in ipairs(step.unlocked) do names[#names + 1] = r.name .. (r.shelf and "（已上架）" or "") end
    w(("| %d | `%s` | %d：%s | %d |"):format(i, step.feature, #step.unlocked, table.concat(names, "、"), total))
    if i >= 60 then w("| … | 其餘見 §3 | | |"); break end
  end
  w("")
  w("## 2. 只有屬性加成的道具")
  w("")
  w("這些道具只要 stat 列對應得到 StatKey（rules/vocab.rs `STATS`）就能由資料執行。")
  w("")
  w("| 道具 | 已上架 | 還缺的 stat |")
  w("|---|---|---|")
  for _, r in ipairs(items) do
    if r.pure then w(("| %s（`%s`） | %s | %s |"):format(r.name, r.id, r.shelf and "是" or "否", #r.blockers == 0 and "—（今天就能載入）" or table.concat(r.blockers, "、"))) end
  end
  w("")
  w("## 3. 每件道具的阻擋項")
  w("")
  w("| 道具 | 已上架 | 阻擋項 |")
  w("|---|---|---|")
  for _, r in ipairs(items) do
    local why = not r.has_ir and "沒有 IR" or (#r.blockers == 0 and "—" or table.concat(r.blockers, "、"))
    w(("| %s（`%s`） | %s | %s |"):format(r.name, r.id, r.shelf and "是" or "否", why))
  end
  w("")
  w("## 4. 重跑")
  w("")
  w("```")
  w("tools\\lua\\lua.exe scripts\\gen_v3_content.lua")
  w("tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_item_order.lua")
  w("```")
  return table.concat(o, "\n") .. "\n"
end

-- the interpreter's vocabulary that the scan checks against (ir.rs, vocab.rs); call before build()
function M.load_rules(root)
  B.STATS = B.load_stats(root)
  B.ABSOLUTE_STATS = B.load_vocab_list(root, "ABSOLUTE_STATS", "StatKey::")
  B.STATE_EFFECTS = B.load_vocab_list(root, "STATE_EFFECTS", "StateEffect::")
end

function M.main(argv)
  local opts = { content = b.root .. "/target/v3-content/v3-content.json", out = b.root .. "/docs/plans/2026-10-28-history-nobu-full-port/m7-items.md" }
  local i = 1
  while i <= #argv do
    if argv[i] == "--content" then opts.content = argv[i + 1]; i = i + 1
    elseif argv[i] == "--out" then opts.out = argv[i + 1]; i = i + 1
    else error("unknown argument " .. argv[i]) end
    i = i + 1
  end
  M.load_rules(b.root)
  local content = json.read(opts.content)
  local shelf = require("content_builder").new(b.root .. "/scripts/lua_data").include("templates/item_shop_v3.lua")
  local items, order = M.build(content, shelf)
  local f = assert(io.open(opts.out, "wb")); f:write(M.render(items, order)); f:close()
  local now = 0 for _, r in ipairs(items) do if r.has_ir and #r.blockers == 0 then now = now + 1 end end
  print(("wrote %s: %d items, %d load today, %d steps"):format(opts.out, #items, now, #order))
  return 0
end

if rawget(_G, "V3_ITEM_ORDER_AS_MODULE") then return M end
os.exit(M.main(arg))
