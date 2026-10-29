-- English text lint for package schema v3 (presentation/en-glossary.md §3).
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_en_lint.lua [--warnings] [FILE ...]
--
-- Checks every text.en name and body of the hero and item documents (all of them when no file
-- is given) against the glossary: banned variants, second person in hero documents, name
-- capitalization, internal jargon, developer wording (either language), long-vowel marks and
-- zh_TW / en placeholder parity are errors; one zh_TW name
-- with several English names, missing end punctuation and spacing are warnings (--warnings).
-- Exit code 1 when there is an error.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local M = {}

-- lowercase Lua patterns; `%f[%a]` / `%f[%A]` are word edges
M.BANNED = {
  { "magical damage", "magic damage" },
  { "true damage", "pure damage" },
  { "magic[ %-]immun", "spell immune / spell immunity" },
  { "%f[%a]max health", "maximum health" },
  { "%f[%a]max hp%f[%A]", "maximum health" },
  { "%f[%a]hp%f[%A]", "health" },
  { "hit points", "health" },
  { "%f[%a]move speed", "movement speed" },
  { "moving speed", "movement speed" },
  { "armour", "armor" },
  { "channelling", "channeling" },
  { "life steal", "lifesteal" },
  { "%f[%a]spell amp%f[%A]", "spell amplification" },
  { "%f[%a]aoe%f[%A]", "within {radius}" },
  { "area of effect", "within {radius}" },
  { "^use:", "Active:" },
  { "on use:", "Active:" },
  { "cool%-down", "cooldown" },
  { "%f[%a]health regen%f[%A]", "health regeneration" },
  { "%f[%a]mana regen%f[%A]", "mana regeneration" },
}
M.JARGON = { "%f[%a]nobu%f[%A]", "%f[%a]modifiers?%f[%A]", "%f[%a]handlers?%f[%A]", "%f[%a]lua%f[%A]",
  "%f[%a]kv%f[%A]" }
M.SECOND_PERSON = { "%f[%a]you%f[%A]", "%f[%a]your%f[%A]", "%f[%a]yourself%f[%A]" }
-- developer wording in player text, both languages (the reason belongs in quirks / notes)
M.DEV_ZH = { "原作", "腳本", "死碼", "執行期", "錯誤", "bug", "BUG", "Bug" }
M.DEV_EN = { "original script", "original game", "%f[%a]scripts?%f[%A]", "%f[%a]bugs?%f[%A]", "%f[%a]bugged%f[%A]",
  "errors? out", "at runtime", "commented out" }
-- romanization is written without long-vowel marks (Saito, Nohime, Meito)
M.MACRONS = { "ā", "ē", "ī", "ō", "ū", "Ā", "Ē", "Ī", "Ō", "Ū", "â", "ê", "î", "ô", "û", "Â", "Ê", "Î", "Ô", "Û" }

local function placeholders(s)
  local set = {}
  for p in tostring(s or ""):gmatch("{([^{}]+)}") do set[p] = true end
  return set
end
local function diff(a, b)
  local out = {}
  for k in pairs(a) do if not b[k] then out[#out + 1] = k end end
  table.sort(out)
  return out
end

-- every text.en string with its zh_TW counterpart: { path, key, zh, en }
function M.collect(doc)
  local rows = {}
  local function walk(t, path)
    if type(t) ~= "table" then return end
    if type(t.text) == "table" and type(t.text.en) == "table" then
      local zh = type(t.text.zh_TW) == "table" and t.text.zh_TW or {}
      for _, k in ipairs({ "name", "body" }) do
        if type(t.text.en[k]) == "string" then
          rows[#rows + 1] = { path = path .. (t.id and ("." .. t.id) or ""), key = k, zh = zh[k], en = t.text.en[k] }
        end
      end
    end
    local ks = {}
    for k in pairs(t) do if k ~= "text" and type(t[k]) == "table" then ks[#ks + 1] = k end end
    table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
    for _, k in ipairs(ks) do walk(t[k], path .. "/" .. tostring(k)) end
  end
  walk(doc, "")
  return rows
end

-- lint one document's rows; is_hero turns on the second-person rule
function M.lint_rows(rows, is_hero, report)
  for _, r in ipairs(rows) do
    local where = r.path .. " " .. r.key
    local low = r.en:gsub("{[^{}]*}", "{}"):lower() -- placeholder names are not prose
    for _, b in ipairs(M.BANNED) do
      if low:find(b[1]) then report("error", where, "寫成 " .. b[2] .. "（en-glossary §1）：" .. r.en) end
    end
    if r.key == "body" then
      for _, j in ipairs(M.JARGON) do
        if low:find(j) then report("error", where, "內部用語（en-glossary §2）：" .. r.en) end
      end
      for _, d in ipairs(M.DEV_EN) do
        if low:find(d) then report("error", where, "說明不寫開發用語，原因放 quirks／notes（en-glossary §2）：" .. r.en); break end
      end
      for _, d in ipairs(M.DEV_ZH) do
        if r.zh and r.zh:find(d, 1, true) then
          report("error", where, "zh_TW 說明不寫開發用語，原因放 quirks／notes（en-glossary §2）：" .. r.zh); break
        end
      end
    end
    for _, m in ipairs(M.MACRONS) do
      if r.en:find(m, 1, true) then report("error", where, "羅馬字不加長音符號（en-glossary §2）：" .. r.en); break end
    end
    if is_hero then
      for _, p in ipairs(M.SECOND_PERSON) do
        if low:find(p) then report("error", where, "英雄文件用第三人稱（en-glossary §2）：" .. r.en); break end
      end
    end
    if r.key == "name" then
      local first = r.en:match("%a")
      if first and first ~= first:upper() then report("error", where, "名稱要大寫開頭：" .. r.en) end
    end
    if r.zh then
      local zs, es = placeholders(r.zh), placeholders(r.en)
      local missing, extra = diff(zs, es), diff(es, zs)
      if #missing > 0 then report("error", where, "en 少了佔位符 " .. table.concat(missing, ", ")) end
      if #extra > 0 then report("error", where, "en 多了佔位符 " .. table.concat(extra, ", ")) end
    end
    if r.key == "body" and not r.en:match("[%.%?!%)]%s*$") then report("warning", where, "說明沒有以句號結尾：" .. r.en) end
    if r.en:find("  ", 1, true) or r.en:find(" [,%.;:]") then report("warning", where, "多餘的空白：" .. r.en) end
  end
end

function M.run(files, opts)
  opts = opts or {}
  PACKAGE_V3_AS_MODULE = true
  local v3 = dofile(tools_dir .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  local errors, warnings, lines = 0, 0, {}
  local names = {}
  for _, f in ipairs(files) do
    local doc, why = v3.load_file(f.path)
    if not doc then
      errors = errors + 1
      lines[#lines + 1] = "error " .. f.label .. ": " .. tostring(why)
    else
      local rows = M.collect(doc)
      M.lint_rows(rows, f.hero, function(level, where, msg)
        if level == "error" then errors = errors + 1 else warnings = warnings + 1 end
        if level == "error" or opts.warnings then lines[#lines + 1] = level .. " " .. f.label .. " " .. where .. ": " .. msg end
      end)
      for _, r in ipairs(rows) do
        if r.key == "name" and r.zh then
          names[r.zh] = names[r.zh] or {}
          names[r.zh][r.en] = names[r.zh][r.en] or (f.label .. " " .. r.path)
        end
      end
    end
  end
  local zhs = {}
  for zh, m in pairs(names) do
    local c = 0
    for _ in pairs(m) do c = c + 1 end
    if c > 1 then zhs[#zhs + 1] = zh end
  end
  table.sort(zhs)
  for _, zh in ipairs(zhs) do
    warnings = warnings + 1
    if opts.warnings then
      local parts = {}
      for en, at in pairs(names[zh]) do parts[#parts + 1] = en .. "（" .. at .. "）" end
      table.sort(parts)
      lines[#lines + 1] = "warning " .. zh .. " 有不同的英文名稱：" .. table.concat(parts, "；")
    end
  end
  return { errors = errors, warnings = warnings, lines = lines }
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  local sources = dofile(tools_dir .. "/v3_sources.lua")
  local opts, files = {}, {}
  for _, a in ipairs(argv or {}) do
    if a == "--warnings" then opts.warnings = true
    else files[#files + 1] = { path = a, label = a, hero = a:find("packages[/\\]") ~= nil } end
  end
  if #files == 0 then
    for _, h in ipairs(sources.heroes(root)) do
      files[#files + 1] = { path = h.path, label = "packages/" .. h.package, hero = true }
    end
    for _, it in ipairs(sources.items(root)) do
      local p = type(it) == "table" and it.path or it
      files[#files + 1] = { path = p, label = (p:gsub("^.*[/\\]items[/\\]", "items/")), hero = false }
    end
  end
  local result = M.run(files, opts)
  for _, l in ipairs(result.lines) do print(l) end
  print(("%d files: %d errors, %d warnings"):format(#files, result.errors, result.warnings))
  return result.errors == 0 and 0 or 1
end

if arg and arg[0] and arg[0]:match("v3_en_lint%.lua$") then os.exit(M.main(arg)) end
return M
