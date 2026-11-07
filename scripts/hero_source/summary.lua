-- Renders summary.md from a built catalog. Counts are recomputed here from the
-- catalog rows so the summary cannot drift from the data it describes.
local classify = require("hero_source.classify")

local M = {}

local function count_by(rows, fn)
  local out = {}
  for _, r in ipairs(rows) do
    local k = fn(r)
    if k ~= nil then out[k] = (out[k] or 0) + 1 end
  end
  return out
end

local function sorted_pairs(t)
  local keys = {}
  for k in pairs(t) do keys[#keys + 1] = k end
  table.sort(keys)
  local i = 0
  return function() i = i + 1; local k = keys[i]; if k ~= nil then return k, t[k] end end
end

local function is_skill(a) return a.kind == "skill" or a.kind == "sub_ability" end

function M.expected_main_counts() return { A_Oda = 36, B_Unified = 36, C_Neutral = 25 } end

function M.render(catalog)
  local L = {}
  local function w(s) L[#L + 1] = s or "" end
  w("# 來源英雄索引摘要（D1／D2）")
  w()
  w("由 `tools\\lua\\lua.exe scripts\\hero_source_index.lua index` 生成，不要手改；完整資料在 `source-catalog.lua`。")
  w()
  w("- 來源：" .. catalog.source.repository .. " @ `" .. catalog.source.commit .. "`")
  w("- 只以文字讀取，未執行來源內任何腳本（`read_hero.lua`、`langtext.lua`、vscripts）。")
  w("- 每筆 ability／unit／tooltip 都帶 `path`、`line`、`start`／`stop`（原檔 byte offset，含 BOM）、`key_token`、`raw` 與 `raw_sha256`；`verify` 子命令逐筆回讀原檔比對。")
  w()

  w("## 檔案與英雄數")
  w()
  local expected = M.expected_main_counts()
  local main = count_by(catalog.files, function(f) return f.main_pattern and f.faction or nil end)
  local v2 = count_by(catalog.files, function(f) return f.variant == "_2" and f.faction or nil end)
  local other = count_by(catalog.files, function(f) return f.variant == "unrecognized" and f.faction or nil end)
  w("| 陣營 | `[ABC][0-9][0-9].txt` 主檔 | 預期 | `_2` 檔 | 其他 |")
  w("|---|---|---|---|---|")
  local total_main, total_expected = 0, 0
  for faction, exp in sorted_pairs(expected) do
    local n = main[faction] or 0
    total_main, total_expected = total_main + n, total_expected + exp
    w(string.format("| %s | %d | %d | %d | %d |", faction, n, exp, v2[faction] or 0, other[faction] or 0))
  end
  w(string.format("| 合計 | %d | %d | %d | |", total_main, total_expected,
    (v2.A_Oda or 0) + (v2.B_Unified or 0) + (v2.C_Neutral or 0)))
  w()
  if total_main == total_expected then
    w("主檔英雄數 " .. total_main .. " 與 36／36／25 一致。")
  else
    w("**主檔英雄數 " .. total_main .. " 與預期 " .. total_expected .. " 不一致，原因見下方診斷。**")
  end
  w()
  w("`_2` 檔不是另一套技能組：它們是 `DOTAHeroes` 單位定義（模型、攻擊、`Ability1..N` 技能欄）。主檔才是 `DOTAAbilities` 技能定義。英雄數只算主檔；`_2` 與 `_old`／`_20` 技能另列版本，不混算。")
  w()
  local listed = count_by(catalog.heroes, function(h) return h.listed_on_site and "listed" or "not_listed" end)
  w(string.format("- 英雄列：%d；有單位定義（`_2`）：%d；`langtext.lua` 有繁中名：%d；其中網站名單註解掉（未上架）：%d。",
    #catalog.heroes, #(function() local t = {} for _, h in ipairs(catalog.heroes) do if h.unit then t[#t + 1] = h end end return t end)(),
    #(function() local t = {} for _, h in ipairs(catalog.heroes) do if h.name_zhtw then t[#t + 1] = h end end return t end)(),
    listed.not_listed or 0))
  local unlisted = {}
  for _, h in ipairs(catalog.heroes) do
    if not h.listed_on_site then unlisted[#unlisted + 1] = h.source_hero_id .. (h.name_zhtw and ("（" .. h.name_zhtw .. "）") or "（無名稱）") end
  end
  if #unlisted > 0 then w("- 未上架：" .. table.concat(unlisted, "、") .. "。仍完整索引，選角時需注意資料可能未維護。") end
  w()

  w("## 技能條目")
  w()
  w("ID 文法：`<英雄><核心>[_20][_old]`。`kind`：`skill`＝單一 W/E/R/T/D/F；`sub_ability`＝以槽位字母開頭的衍生招（例 `A28TW`、`A13W2`）；`auxiliary`＝其他輔助塊（`_HIDE`、`_DATA`、`B16MM*` 等）；`precache` 不分類。`variant`：`current`、`old`（`_old`）、`_20`（來源未說明含義，只照實標記）。")
  w()
  local by_kind = count_by(catalog.abilities, function(a) return a.kind end)
  local by_variant = count_by(catalog.abilities, function(a) return a.variant end)
  w("| kind | 數量 |")
  w("|---|---|")
  for k, n in sorted_pairs(by_kind) do w(string.format("| %s | %d |", k, n)) end
  w()
  w("| variant | 數量 |")
  w("|---|---|")
  for k, n in sorted_pairs(by_variant) do w(string.format("| %s | %d |", k, n)) end
  w()
  local skills = {}
  for _, a in ipairs(catalog.abilities) do if is_skill(a) then skills[#skills + 1] = a end end
  local tip = count_by(skills, function(a) return a.tooltip and a.tooltip.description and "with" or "without" end)
  local vs = count_by(skills, function(a) return a.depends_on_missing_vscript and "yes" or "no" end)
  local unit_listed = count_by(skills, function(a) return a.in_unit_ability_list and "yes" or nil end)
  w(string.format("- 技能（skill＋sub_ability）共 %d；有繁中描述 %d、缺描述 %d。", #skills, tip.with or 0, tip.without or 0))
  w(string.format("- 出現在 `_2` 的 `Ability1..N` 技能欄：%d。", unit_listed.yes or 0))
  w(string.format("- 含 `RunScript` 呼叫來源缺少的 `scripts/vscripts/...`：%d／%d。這些技能的實際行為有一部分只存在於缺失的 Lua，KV 與描述只能說明意圖，不能冒稱已還原。", vs.yes or 0, #skills))
  w()

  w("## 粗分類（D2）")
  w()
  w("依 KV（`AbilityBehavior`、動作塊、modifier 屬性）與繁中描述關鍵字標能力族；每個 tag 都附 evidence（`kv_behavior`／`kv_block`／`kv_modifier`／`tooltip_text`）。只出現在描述關鍵字的 tag 是弱證據，D3 必須逐招閱讀確認。")
  w()
  local current = {}
  for _, a in ipairs(skills) do if a.variant == "current" then current[#current + 1] = a end end
  local status_all = count_by(skills, function(a) return a.classification.status end)
  local status_cur = count_by(current, function(a) return a.classification.status end)
  local aux = {}
  for _, a in ipairs(catalog.abilities) do if a.kind == "auxiliary" or a.kind == "bare_hero_id" or a.kind == "foreign" then aux[#aux + 1] = a end end
  local status_aux = count_by(aux, function(a) return a.classification.status end)
  w(string.format("- 全部技能 %d：classified %d、**unclassified %d**。", #skills, status_all.classified or 0, status_all.unclassified or 0))
  w(string.format("- 其中 current 版 %d：classified %d、**unclassified %d**。", #current, status_cur.classified or 0, status_cur.unclassified or 0))
  w(string.format("- auxiliary 塊 %d：classified %d、unclassified %d。", #aux, status_aux.classified or 0, status_aux.unclassified or 0))
  local basis = count_by(skills, function(a) return a.classification.basis end)
  local parts = {}
  for k, n in sorted_pairs(basis) do parts[#parts + 1] = k .. " " .. n end
  w("- 證據來源分布（技能）：" .. table.concat(parts, "、") .. "。")
  w()
  w("| 能力族（README §5） | 全部技能 | current |")
  w("|---|---|---|")
  for _, fam in ipairs(classify.FAMILIES) do
    local function has(a) for _, f in ipairs(a.classification.families) do if f == fam.id then return true end end end
    local na, nc = 0, 0
    for _, a in ipairs(skills) do if has(a) then na = na + 1 end end
    for _, a in ipairs(current) do if has(a) then nc = nc + 1 end end
    w(string.format("| `%s` %s | %d | %d |", fam.id, fam.label, na, nc))
  end
  w()
  local unc = {}
  for _, a in ipairs(current) do
    if a.classification.status == "unclassified" then unc[#unc + 1] = "`" .. a.id .. "`" .. (a.display_name and ("（" .. a.display_name .. "）") or "") end
  end
  if #unc > 0 then
    w("current 版 unclassified：" .. table.concat(unc, "、") .. "。")
    w()
  end

  w("## 來源 ID → 本地遊戲 ID")
  w()
  w("規則：" .. catalog.id_mapping.rule .. "。完整 97 列在 `source-catalog.lua` 的 `id_mapping.entries`；以下只列非 `unmapped` 的列。")
  w()
  w("| 來源 ID | 來源名稱 | 狀態 | 本地 game_hero_id | 備註 |")
  w("|---|---|---|---|---|")
  local status_count = count_by(catalog.id_mapping.entries, function(r) return r.status end)
  for _, r in ipairs(catalog.id_mapping.entries) do
    if r.status ~= "unmapped" or r.number_conflict then
      local note = r.note or ""
      if r.number_conflict then
        note = (note ~= "" and (note .. "；") or "") .. "**衝突**：本地 `" .. r.number_conflict.local_dir .. "`（" ..
          tostring(r.number_conflict.local_name_zhtw) .. "）與來源同編號但不同人，禁止按編號配對"
      end
      w(string.format("| %s | %s | `%s` | %s | %s |", r.source_hero_id, r.source_name_zhtw or "", r.status,
        r.game_hero_id and ("`" .. r.game_hero_id .. "`") or "—", note))
    end
  end
  w()
  parts = {}
  for k, n in sorted_pairs(status_count) do parts[#parts + 1] = "`" .. k .. "` " .. n end
  w("統計：" .. table.concat(parts, "、") .. "。")
  w()

  w("## 診斷與來源異常")
  w()
  for _, f in ipairs(catalog.files) do
    if f.main_pattern and f.root_key ~= "DOTAAbilities" then
      w("- `" .. f.path .. "` 的根 key 是 `" .. tostring(f.root_key) .. "`（預期 `DOTAAbilities`），內容仍是技能定義，照常索引。")
    elseif f.variant == "_2" and f.root_key ~= "DOTAHeroes" then
      w("- `" .. f.path .. "` 的根 key 是 `" .. tostring(f.root_key) .. "`（預期 `DOTAHeroes`），照常索引。")
    end
  end
  local bom = count_by(catalog.files, function(f) return f.bom and "bom" or nil end)
  w(string.format("- 帶 UTF-8 BOM 的檔案：%d（offset 一律含 BOM）。", bom.bom or 0))
  for _, f in ipairs(catalog.files) do
    if f.role == "tooltips_zhtw" then
      w(string.format("- `%s` 有 %d 個重複 token key；以第一次出現者為準。UTF-16 的 `addon_tchinese.txt` 未使用（與來源網站程式一致，取 `*2.txt`）。", f.path, f.duplicate_tokens or 0))
    end
  end
  if #catalog.diagnostics == 0 then
    w("- 解析診斷：無。")
  else
    for _, d in ipairs(catalog.diagnostics) do
      w(string.format("- 解析診斷 `%s`：`%s`:%s %s", d.kind, tostring(d.path), tostring(d.line or ""), d.detail and ("`" .. d.detail .. "`") or ""))
    end
  end
  local unresolved = 0
  for _, a in ipairs(catalog.abilities) do if a.unresolved_placeholders then unresolved = unresolved + 1 end end
  w(string.format("- 含未解析 `%%placeholder`（AbilitySpecial 與頂層欄位都沒有同名變數；多為引擎內建值，如 `%%attack_damage`）的條目：%d；名單見 catalog 的 `unresolved_placeholders`。", unresolved))
  w()
  return table.concat(L, "\n") .. "\n"
end

return M
