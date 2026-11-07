-- M6 waves (history-nobu full port): the heroes the game does not ship yet, in batches of 8-10 that
-- the executor unlocks one component group at a time. Built on v3_usage.lua's component sets.
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_waves.lua [--out FILE.md] [--json FILE.json]
--
-- Default output: docs/plans/2026-10-28-history-nobu-full-port/m6-waves.md.
-- Baseline: every component the shipped heroes need (M3 runs B01 and C17 as data; M5 moves the rest
-- of the shipped kits to data, so those components exist before M6). Then the same greedy order as
-- v3_usage (most heroes unlocked per new component) over the unshipped heroes; a wave closes before
-- a step once it has M.MIN heroes, and after a step that brings it to M.MAX. One step can unlock more
-- heroes than that at once (they share every component), so a wave can be larger; a last wave
-- shorter than half of M.MIN joins the one before it.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local lfs = require("lfs")
local M = {}
M.MIN, M.MAX = 9, 10

-- component -> mechanic label (for the wave description and the acceptance focus)
M.FEATURES = {
  { "action:create_unit", "召喚", "召喚物的數量上限、時限、控制權與擊殺獎勵" },
  { "action:create_illusion", "分身", "分身的輸出與承受傷害比例、道具複製、分身死亡" },
  { "action:dash", "位移", "位移路徑被地形阻擋、位移中被控制、終點" },
  { "action:leap", "位移", "跳躍落點、跳躍期間是否可被選取" },
  { "action:blink", "位移", "閃現距離上限與不可通行地形" },
  { "action:swap_positions", "位移", "交換位置的對象與距離" },
  { "action:impulse", "擊退／拉扯", "強制位移的距離、地形與碰撞" },
  { "action:knockback", "擊退／拉扯", "強制位移的距離、地形與碰撞" },
  { "action:zone", "地面區域", "區域的持續時間、每跳間隔、離開後的殘留" },
  { "action:projectile", "投射物", "投射物命中判定、被閃避、射程盡頭" },
  { "action:chain", "連鎖彈跳", "彈跳次數、不重複目標、彈跳半徑" },
  { "action:tether", "連結", "連結斷開條件與到期結算" },
  { "action:add_ability", "換技能", "技能替換後的冷卻與等級" },
  { "action:swap_abilities", "換技能", "技能替換後的冷卻與等級" },
  { "fn:ability_in_slot", "偷技能", "偷到的技能等級、時限與冷卻" },
  { "status:absorb", "吸收護盾", "護盾吸收量、傷害類型、破盾" },
  { "status:aura", "光環", "光環半徑、殘留時間、同名不疊加" },
  { "passive:aura", "光環", "光環半徑、殘留時間、同名不疊加" },
  { "passive:orb_slot", "法球", "法球與其他法球、道具法球的互斥" },
  { "cast:channel", "引導", "引導被打斷、引導中的每跳" },
  { "state:invisible", "隱形", "隱形被看穿、攻擊或施法解除隱形" },
  { "template:chase", "召喚物 AI", "召喚物追擊、放棄目標、瞬移回主人" },
  { "cost:charges", "充能", "充能上限、回充時間、升級時的充能" },
  { "event:on_order", "指令觸發", "依指令觸發的效果" },
  { "unit_state:scope:match", "全場共用狀態", "全場共用狀態的同步與揭露" },
}

function M.build(root)
  local prev = rawget(_G, "V3_USAGE_AS_MODULE")
  V3_USAGE_AS_MODULE = true
  local U = dofile(tools_dir .. "/v3_usage.lua")
  V3_USAGE_AS_MODULE = prev
  local S = dofile(tools_dir .. "/v3_sources.lua")
  local usage = U.build(root)
  local shipped = {}
  for _, h in ipairs(S.heroes(root)) do
    if lfs.attributes(root .. "/" .. S.PACKAGES .. "/" .. h.package .. "/character.lua") then shipped[h.source_id] = true end
  end
  local have, base_list = {}, {}
  local pending, blocked = {}, {}
  for _, u in ipairs(usage.units) do
    if u.kind == "hero" then
      if shipped[u.id] then
        for c in pairs(u.comps) do if not have[c] then have[c] = true; base_list[#base_list + 1] = c end end
      elseif u.blocked then blocked[#blocked + 1] = u
      else pending[#pending + 1] = u end
    end
  end
  table.sort(base_list)
  local comp = usage.components
  local function missing(u)
    local m = {}
    for c in pairs(u.comps) do if not have[c] then m[#m + 1] = c end end
    table.sort(m, function(a, b) if comp[a].heroes ~= comp[b].heroes then return comp[a].heroes > comp[b].heroes end return a < b end)
    return m
  end
  local function features(u)
    local seen, out = {}, {}
    for _, f in ipairs(M.FEATURES) do
      if u.comps[f[1]] and not seen[f[2]] then seen[f[2]] = true; out[#out + 1] = f end
    end
    return out
  end
  local done = {}
  local waves, cur = {}, nil
  local function open() cur = { heroes = {}, components = {} } end
  local function close() if cur and #cur.heroes > 0 then waves[#waves + 1] = cur end; cur = nil end
  open()
  local function unlock_ready()
    local n = 0
    for _, u in ipairs(pending) do
      if not done[u.id] and #missing(u) == 0 then
        done[u.id] = true
        cur.heroes[#cur.heroes + 1] = u
        n = n + 1
      end
    end
    return n
  end
  unlock_ready() -- heroes the baseline already covers
  while true do
    local best, best_score, best_cost
    for _, u in ipairs(pending) do
      if not done[u.id] then
        local m = missing(u)
        local trial = {}
        for _, c in ipairs(m) do trial[c] = true end
        local gain = 0
        for _, v in ipairs(pending) do
          if not done[v.id] then
            local ok = true
            for c in pairs(v.comps) do if not have[c] and not trial[c] then ok = false; break end end
            if ok then gain = gain + 1 end
          end
        end
        local score = gain / math.max(1, #m)
        if not best or score > best_score or (score == best_score and #m < best_cost) or (score == best_score and #m == best_cost and u.id < best.id) then
          best, best_score, best_cost = u, score, #m
        end
      end
    end
    if not best then break end
    -- the components this step adds; a wave that is full closes before the next step starts
    if #cur.heroes >= M.MIN then close(); open() end
    for _, c in ipairs(missing(best)) do
      have[c] = true
      cur.components[#cur.components + 1] = c
    end
    unlock_ready()
    if #cur.heroes >= M.MAX then close(); open() end
  end
  close()
  -- a short last wave joins the one before it (its components come after that wave's anyway)
  if #waves >= 2 and #waves[#waves].heroes < M.MIN / 2 then
    local last = table.remove(waves)
    for _, u in ipairs(last.heroes) do table.insert(waves[#waves].heroes, u) end
    for _, c in ipairs(last.components) do table.insert(waves[#waves].components, c) end
  end
  -- feature summary per wave
  for i, w in ipairs(waves) do
    w.index = i
    local fs, seen = {}, {}
    for _, u in ipairs(w.heroes) do
      u.features = features(u)
      for _, f in ipairs(u.features) do if not seen[f[2]] then seen[f[2]] = true; fs[#fs + 1] = f end end
    end
    w.features = fs
  end
  return { waves = waves, baseline = base_list, blocked = blocked, shipped = shipped, components = comp }
end

function M.render(r)
  local out = {}
  local function w(s) out[#out + 1] = s end
  w("# M6 批次規劃：尚未上架英雄的開放順序")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_waves.lua` 從全部 v3 文件產生，不要手改。依據的元件與計數見 [usage-stats.md](usage-stats.md)。")
  w("")
  w("- **基準**：已上架 10 位英雄需要的 " .. #r.baseline .. " 個元件。M3 讓 B01、C17 用資料執行，M5 把其餘上架英雄切成資料執行，所以這些元件在 M6 開始前就要完成。")
  w("- **排序**：其餘英雄依「每多實作一個元件、能解鎖最多英雄」排序，每 9～10 位一批。B 做完一批的「新元件」，這一批的英雄就能全部用資料執行。")
  w("- 同一組元件可能一次解鎖超過 10 位英雄，這種批次會比較大，驗收時可以再拆成兩組。最後一批太小時併入前一批。")
  w("- **機制特色**：依英雄用到的元件歸類，例如召喚、分身、位移。驗收重點列出這批第一次出現的機制要特別測的情況；逐招、逐 rank 的情境由 `v3_scenarios.lua` 產生。")
  w("")
  w("| 批次 | 英雄數 | 新元件數 | 首次出現的機制 |")
  w("|---|---|---|---|")
  local seen_feature = {}
  for _, wave in ipairs(r.waves) do
    local firsts = {}
    for _, f in ipairs(wave.features) do if not seen_feature[f[2]] then seen_feature[f[2]] = true; firsts[#firsts + 1] = f[2] end end
    wave.first_features = firsts
    w(("| %d | %d | %d | %s |"):format(wave.index, #wave.heroes, #wave.components, #firsts > 0 and table.concat(firsts, "、") or "—"))
  end
  w("")
  for _, wave in ipairs(r.waves) do
    w("## 第 " .. wave.index .. " 批")
    w("")
    w("**英雄**：")
    w("")
    w("| 英雄 | 機制特色 |")
    w("|---|---|")
    for _, u in ipairs(wave.heroes) do
      local fs = {}
      for _, f in ipairs(u.features or {}) do fs[#fs + 1] = f[2] end
      w(("| %s %s | %s |"):format(u.id, u.name or "", #fs > 0 and table.concat(fs, "、") or "一般技能"))
    end
    w("")
    if #wave.components > 0 then
      w("**執行器要新增的元件**（" .. #wave.components .. " 個）：" .. "`" .. table.concat(wave.components, "`、`") .. "`")
    else
      w("**執行器要新增的元件**：無（基準元件已足夠）")
    end
    w("")
    w("**驗收重點**：")
    w("")
    w("- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。")
    for _, f in ipairs(wave.features) do
      local first = false
      for _, x in ipairs(wave.first_features) do if x == f[2] then first = true end end
      if first then w("- " .. f[2] .. "（本批首次出現）：" .. f[3]) end
    end
    w("")
  end
  if #r.blocked > 0 then
    w("## 尚有 pending 的英雄")
    w("")
    for _, u in ipairs(r.blocked) do w("- " .. u.id .. "：" .. u.blocked) end
    w("")
  end
  w("## 基準元件（M3／M5 完成）")
  w("")
  w("`" .. table.concat(r.baseline, "`、`") .. "`")
  w("")
  return table.concat(out, "\n")
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  package.path = root .. "/scripts/?.lua;scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  local out = b.root .. "/docs/plans/2026-10-28-history-nobu-full-port/m6-waves.md"
  local json_out
  local i = 1
  while i <= #argv do
    if argv[i] == "--out" then out = argv[i + 1]; i = i + 1
    elseif argv[i] == "--json" then json_out = argv[i + 1]; i = i + 1
    else error("unknown argument " .. argv[i]) end
    i = i + 1
  end
  local r = M.build(b.root)
  local f = assert(io.open(out, "wb")); f:write(M.render(r), "\n"); f:close()
  if json_out then
    local waves = {}
    for _, wave in ipairs(r.waves) do
      local hs = {}
      for _, u in ipairs(wave.heroes) do
        local fs = {}
        for _, x in ipairs(u.features or {}) do fs[#fs + 1] = x[2] end
        hs[#hs + 1] = { id = u.id, name = u.name, features = fs }
      end
      waves[#waves + 1] = { index = wave.index, heroes = hs, components = wave.components }
    end
    local j = assert(io.open(json_out, "wb")); j:write(b.lib("json").encode({ baseline = r.baseline, waves = waves })); j:close()
  end
  local n = 0
  for _, wave in ipairs(r.waves) do n = n + #wave.heroes end
  print(("wrote %s: %d waves, %d heroes, baseline %d components"):format(out, #r.waves, n, #r.baseline))
  return 0
end

if rawget(_G, "V3_WAVES_AS_MODULE") then return M end
os.exit(M.main(arg))
