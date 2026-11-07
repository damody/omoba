-- The schema manuals as site pages: docs/character_pipeline/schema/*.md rendered as they are, so the
-- site has no second copy to maintain. Only the Markdown these files use: headings, paragraphs,
-- fenced code, tables, nested lists, quotes, inline code, bold and links. Links to another manual
-- become site links; links out of the manuals keep their text only (the site stands alone).
local I = require("site.i18n")

local M = {}

M.DOCS = {
  { file = "v3.md", slug = "schema" },
  { file = "v3-authoring.md", slug = "authoring" },
  { file = "v3-vocab.md", slug = "vocab" },
}

local function esc(s) return (s:gsub("&", "&amp;"):gsub("<", "&lt;"):gsub(">", "&gt;"):gsub('"', "&quot;")) end

local function link_target(href)
  if href:match("^https?://") then return href end
  local file, anchor = href:match("^([^#/]+%.md)(#?.*)$")
  for _, d in ipairs(M.DOCS) do
    if file == d.file then return d.slug .. ".html" .. anchor end
  end
  return nil
end

-- Code spans first, so nothing inside them is read as Markdown.
function M.inline(s)
  local parts, pos = {}, 1
  local function text(t)
    t = esc(t):gsub("%*%*(.-)%*%*", "<strong>%1</strong>")
    t = t:gsub("%[([^%]]+)%]%(([^%)]+)%)", function(label, href)
      local target = link_target(href)
      return target and ('<a href="' .. target .. '">' .. label .. "</a>") or label
    end)
    return t
  end
  while true do
    local a, b, code = s:find("`([^`]+)`", pos)
    if not a then parts[#parts + 1] = text(s:sub(pos)); break end
    parts[#parts + 1] = text(s:sub(pos, a - 1))
    parts[#parts + 1] = "<code>" .. esc(code) .. "</code>"
    pos = b + 1
  end
  return table.concat(parts)
end

-- Cells split on | outside code spans; \| is a literal bar.
local function cells(line)
  line = line:gsub("^%s*|", ""):gsub("|%s*$", "")
  local out, cur, in_code, i = {}, {}, false, 1
  while i <= #line do
    local c = line:sub(i, i)
    if c == "\\" and line:sub(i + 1, i + 1) == "|" then cur[#cur + 1] = "|"; i = i + 1
    elseif c == "`" then in_code = not in_code; cur[#cur + 1] = c
    elseif c == "|" and not in_code then out[#out + 1] = table.concat(cur); cur = {}
    else cur[#cur + 1] = c end
    i = i + 1
  end
  out[#out + 1] = table.concat(cur)
  for k, v in ipairs(out) do out[k] = M.inline((v:gsub("^%s+", ""):gsub("%s+$", ""))) end
  return out
end

local function is_table_rule(line) return line and line:match("^%s*|[%s:|%-]+|%s*$") and line:find("-", 1, true) end

-- -> html, headings { { level, id, text } }
function M.render(md)
  local lines = {}
  for line in (md:gsub("\r\n", "\n") .. "\n"):gmatch("(.-)\n") do lines[#lines + 1] = line end
  local out, heads, used = {}, {}, {}
  local para = {}
  local function flush()
    if #para > 0 then out[#out + 1] = "<p>" .. M.inline(table.concat(para, " ")) .. "</p>"; para = {} end
  end
  local open = {}
  local function close_lists(to)
    while #open > to do
      out[#out + 1] = "</li></" .. open[#open].tag .. ">"
      open[#open] = nil
    end
  end
  local i = 1
  while i <= #lines do
    local line = lines[i]
    local fence = line:match("^%s*```")
    local level, title = line:match("^(#+)%s+(.*)$")
    local indent, marker, item = line:match("^(%s*)([%-%*])%s+(.*)$")
    if not marker then indent, marker, item = line:match("^(%s*)(%d+%.)%s+(.*)$") end
    if fence then
      flush(); close_lists(0)
      local code = {}
      i = i + 1
      while i <= #lines and not lines[i]:match("^%s*```") do code[#code + 1] = lines[i]; i = i + 1 end
      out[#out + 1] = "<pre><code>" .. esc(table.concat(code, "\n")) .. "</code></pre>"
    elseif level then
      flush(); close_lists(0)
      local n = #level
      local base = "s" .. tostring(#heads + 1)
      used[base] = true
      heads[#heads + 1] = { level = n, id = base, text = title }
      out[#out + 1] = string.format('<h%d id="%s">%s</h%d>', n, base, M.inline(title), n)
    elseif line:match("^%s*|") and is_table_rule(lines[i + 1]) then
      flush(); close_lists(0)
      out[#out + 1] = '<div class="tbl"><table><thead><tr>'
      for _, c in ipairs(cells(line)) do out[#out + 1] = "<th>" .. c .. "</th>" end
      out[#out + 1] = "</tr></thead><tbody>"
      i = i + 2
      while i <= #lines and lines[i]:match("^%s*|") do
        out[#out + 1] = "<tr>"
        for _, c in ipairs(cells(lines[i])) do out[#out + 1] = "<td>" .. c .. "</td>" end
        out[#out + 1] = "</tr>"
        i = i + 1
      end
      i = i - 1
      out[#out + 1] = "</tbody></table></div>"
    elseif marker then
      flush()
      local width = #indent
      local tag = marker:match("%d") and "ol" or "ul"
      while #open > 0 and open[#open].width > width do close_lists(#open - 1) end
      if #open > 0 and open[#open].width == width then
        out[#out + 1] = "</li><li>" .. M.inline(item)
      else
        open[#open + 1] = { width = width, tag = tag }
        out[#out + 1] = "<" .. tag .. "><li>" .. M.inline(item)
      end
    elseif line:match("^%s*$") then
      flush()
      local nxt = lines[i + 1]
      if not (nxt and (nxt:match("^%s*[%-%*]%s") or nxt:match("^%s*%d+%.%s"))) then close_lists(0) end
    elseif line:match("^>%s?") then
      flush(); close_lists(0)
      out[#out + 1] = "<blockquote>" .. M.inline((line:gsub("^>%s?", ""))) .. "</blockquote>"
    elseif #open > 0 and line:match("^%s+%S") then
      out[#out + 1] = " " .. M.inline((line:gsub("^%s+", "")))
    else
      close_lists(0)
      para[#para + 1] = line
    end
    i = i + 1
  end
  flush(); close_lists(0)
  return table.concat(out, "\n"), heads
end

-- Each manual's table of contents: its second- and third-level headings.
local function contents(heads)
  local out = { '<nav class="toc"><ul>' }
  for _, h in ipairs(heads) do
    if h.level == 2 then
      out[#out + 1] = '<li><a href="#' .. h.id .. '">' .. M.inline(h.text) .. "</a></li>"
    end
  end
  out[#out + 1] = "</ul></nav>"
  return table.concat(out)
end

-- root: repo root; read(path) -> text. Returns the pages of one language; a manual that is missing
-- (a fixture root) has no page.
function M.pages(root, lang, dir, read)
  local L = I.strings(lang)
  local pages, links = {}, {}
  for _, d in ipairs(M.DOCS) do
    local md = read(root .. "/docs/character_pipeline/schema/" .. d.file)
    if md then
      local html, heads = M.render(md)
      local title = L.manual_titles[d.slug]
      pages[#pages + 1] = { lang = lang, path = dir .. "/manual/" .. d.slug .. ".html", title = title, manual = true,
        body = '<p class="sub">' .. esc(L.manual_language) .. "</p>" .. contents(heads) .. html }
      links[#links + 1] = '<li><a href="' .. d.slug .. '.html">' .. esc(title) .. "</a></li>"
    end
  end
  pages[#pages + 1] = { lang = lang, path = dir .. "/manual/index.html", title = L.manual, manual = true,
    body = "<h1>" .. esc(L.manual) .. "</h1><p>" .. esc(L.manual_intro) .. "</p><ul>" .. table.concat(links) .. "</ul>" }
  return pages
end

return M
