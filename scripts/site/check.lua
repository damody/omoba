-- Page checks shared by build_site (fails the build) and scripts/tests/site_build_test.lua:
--   1. every value span shows exactly what the catalog value renders to;
--   2. hero stat spans equal the catalog's unit stats; rank labels are 1..max;
--   3. no digit anywhere else in the page text (descriptions have no numbers that do
--      not come from a placeholder; identifiers in <code> and the hash are exempt).
local Q = require("site.rational")
local R = require("site.text_render")
local M = {}

local UNESC = { ["&amp;"] = "&", ["&lt;"] = "<", ["&gt;"] = ">", ["&quot;"] = '"' }
local function unescape(s) return (s:gsub("&%a+;", UNESC)) end
local function strip(s) return (s:gsub("<[^>]*>", "")) end

-- C, Cold: page contexts (site/pages.context); hero: the page's hero (for stat spans)
function M.page(html, C, Cold, hero)
  local errors = {}
  local function bad(msg) errors[#errors + 1] = msg end
  local body = html:match("<main>(.*)</main>") or html
  local counts = { values = 0, stats = 0 }

  -- 1. value spans (inner text holds only <b>)
  body = body:gsub('<span class="n[^"]*" data%-v="([^"]+)" data%-f="(%w+)" data%-r="(%d*)"([^>]*)>(.-)</span>',
    function(key, fmt, rank, extra, inner)
      counts.values = counts.values + 1
      local owner, name = key:match("^(.*)/([^/]+)$")
      local ctx = (extra:find('data-c="old"', 1, true) and Cold) or C
      local v = ctx and ctx.values[owner] and ctx.values[owner][name]
      if not v then bad("span refers to a missing value " .. key); return "" end
      local runs = R.value_runs(v, fmt, { rank = tonumber(rank) })
      local want = {}
      for _, r in ipairs(runs) do want[#want + 1] = r.b and ("<b>" .. R.escape(r.t) .. "</b>") or R.escape(r.t) end
      if table.concat(want) ~= inner then bad(key .. " shows " .. inner .. " but the catalog renders " .. table.concat(want)) end
      return ""
    end)
  -- 2. stats, rules, rank labels
  body = body:gsub('<span class="n" data%-stat="([%w_]+)">([^<]*)</span>', function(key, text)
    counts.stats = counts.stats + 1
    local x = hero and hero.stats and hero.stats[key]
    if x == nil then bad("stat span " .. key .. " has no catalog stat")
    elseif Q.decimal(Q.parse(x), 2) ~= text then bad("stat " .. key .. " shows " .. text) end
    return ""
  end)
  body = body:gsub('<span class="n" data%-rule="([%w_.]+)">([^<]*)</span>', function(path, text)
    local cur = C.catalog.rules
    for part in path:gmatch("[^.]+") do cur = type(cur) == "table" and cur[part] or nil end
    if cur == nil or R.number(Q.parse(cur), nil, nil) ~= text then bad("rules." .. path .. " shows " .. text) end
    return ""
  end)
  local last = 0
  body = body:gsub('<span class="lv">(%d+)</span>', function(k)
    k = tonumber(k)
    if k ~= 1 and k ~= last + 1 then bad("rank labels out of order at " .. k) end
    last = k
    return ""
  end)
  body = body:gsub("<code[^>]*>.-</code>", "")
  if body:find('class="n', 1, true) then bad("a number span did not match the expected form") end
  -- 3. no stray digits
  local text = unescape(strip(body))
  local s = text:find("%d") or text:find("\239\188[\144-\153]") -- ASCII or full-width (U+FF10..FF19)
  if s then bad("stray digit in page text: …" .. text:sub(math.max(1, s - 60), s + 20) .. "…") end
  return errors, counts
end

return M
