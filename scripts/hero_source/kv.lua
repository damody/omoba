-- Valve KeyValues (Dota 2 npc/addon text) parser for untrusted source data.
--
-- Every node keeps the byte span of its original text so an index entry can be
-- traced back to the exact token. Offsets are 1-based into the original bytes
-- (including a UTF-8 BOM when present), so `text:sub(node.start, node.stop)`
-- is the node's original text. The parser never evaluates anything; malformed
-- input produces diagnostics instead of errors.
local M = {}

local BOM = "\239\187\191"
local ESCAPES = { n = "\n", t = "\t", r = "\r", ['"'] = '"', ["\\"] = "\\" }

local function is_space(c) return c == " " or c == "\t" or c == "\r" or c == "\n" or c == "\f" or c == "\v" end

local Lexer = {}
Lexer.__index = Lexer

local function new_lexer(text, start)
  return setmetatable({ text = text, pos = start, line = 1, line_start = 1, diagnostics = {} }, Lexer)
end

function Lexer:diag(kind, line, col, detail)
  self.diagnostics[#self.diagnostics + 1] = { kind = kind, line = line, col = col, detail = detail }
end

function Lexer:advance_newlines(from, to)
  local text = self.text
  local i = from
  while true do
    local nl = text:find("\n", i, true)
    if not nl or nl > to then break end
    self.line = self.line + 1
    self.line_start = nl + 1
    i = nl + 1
  end
end

-- Returns the next significant token, collecting comments seen before it.
-- Token: { type = "string"|"open"|"close"|"eof", value, raw, quoted, start, stop, line, col, comments }
function Lexer:next()
  local text, comments = self.text, {}
  while true do
    local pos = self.pos
    local c = text:sub(pos, pos)
    if c == "" then
      return { type = "eof", start = pos, stop = pos - 1, line = self.line, col = pos - self.line_start + 1, comments = comments }
    elseif is_space(c) then
      local s, e = text:find("^[ \t\r\n\f\v]+", pos)
      self:advance_newlines(s, e)
      self.pos = e + 1
    elseif c == "/" and text:sub(pos + 1, pos + 1) == "/" then
      local e = (text:find("\n", pos, true) or (#text + 1)) - 1
      local body = text:sub(pos + 2, e):gsub("\r$", "")
      comments[#comments + 1] = { text = body, line = self.line, start = pos, stop = e }
      self.pos = e + 1
    elseif c == "/" and text:sub(pos + 1, pos + 1) == "*" then
      local line = self.line
      local close = text:find("*/", pos + 2, true)
      local e = close and close + 1 or #text
      if not close then self:diag("unterminated_block_comment", line, pos - self.line_start + 1) end
      comments[#comments + 1] = { text = text:sub(pos + 2, close and close - 1 or #text), line = line, start = pos, stop = e }
      self:advance_newlines(pos, e)
      self.pos = e + 1
    else
      local line, col = self.line, pos - self.line_start + 1
      if c == "{" or c == "}" then
        self.pos = pos + 1
        return { type = c == "{" and "open" or "close", raw = c, start = pos, stop = pos, line = line, col = col, comments = comments }
      elseif c == '"' then
        local parts, i = {}, pos + 1
        while true do
          local ch = text:sub(i, i)
          if ch == "" then
            self:diag("unterminated_string", line, col)
            break
          elseif ch == '"' then
            break
          elseif ch == "\\" then
            local nx = text:sub(i + 1, i + 1)
            if ESCAPES[nx] then
              parts[#parts + 1] = ESCAPES[nx]
              i = i + 2
            else
              -- Unknown escape: keep the backslash literally (Valve behaviour).
              parts[#parts + 1] = "\\"
              i = i + 1
            end
          else
            local s, e = text:find('^[^"\\]+', i)
            parts[#parts + 1] = text:sub(s, e)
            i = e + 1
          end
        end
        local stop = math.min(i, #text)
        self:advance_newlines(pos, stop)
        self.pos = stop + 1
        return { type = "string", quoted = true, value = table.concat(parts), raw = text:sub(pos, stop),
          start = pos, stop = stop, line = line, col = col, comments = comments }
      else
        local s, e = text:find('^[^ \t\r\n\f\v{}"]+', pos)
        local value = text:sub(s, e)
        -- An unquoted token stops before a line comment.
        local cut = value:find("//", 1, true)
        if cut and cut > 1 then e = s + cut - 2; value = text:sub(s, e) end
        self.pos = e + 1
        return { type = "string", quoted = false, value = value, raw = value, start = s, stop = e,
          line = line, col = col, comments = comments }
      end
    end
  end
end

function Lexer:peek()
  local saved = { self.pos, self.line, self.line_start, #self.diagnostics }
  local token = self:next()
  self.pos, self.line, self.line_start = saved[1], saved[2], saved[3]
  for i = #self.diagnostics, saved[4] + 1, -1 do self.diagnostics[i] = nil end
  return token
end

local function comment_texts(comments)
  if #comments == 0 then return nil end
  local out = {}
  for i, c in ipairs(comments) do out[i] = c.text end
  return out
end

-- Parses pairs until a closing brace (consumed by caller) or EOF.
local function parse_pairs(lx, depth)
  local nodes = {}
  local last
  while true do
    local tok = lx:next()
    -- A comment on the same line as the previous node's end is a trailing comment.
    if last and #tok.comments > 0 and tok.comments[1].line == last.stop_line then
      last.trailing_comment = table.remove(tok.comments, 1).text
    end
    if tok.type == "eof" then
      if depth > 0 then lx:diag("unclosed_block", tok.line, tok.col) end
      return nodes, tok, true
    elseif tok.type == "close" then
      if depth == 0 then
        lx:diag("unexpected_close", tok.line, tok.col)
      else
        return nodes, tok, false
      end
    elseif tok.type == "open" then
      lx:diag("block_without_key", tok.line, tok.col)
      local children, close = parse_pairs(lx, depth + 1)
      nodes[#nodes + 1] = { key = nil, children = children, start = tok.start, stop = close.stop,
        line = tok.line, col = tok.col, stop_line = close.line, missing_key = true }
      last = nodes[#nodes]
    else
      local node = { key = tok.value, key_token = tok.raw, key_quoted = tok.quoted,
        start = tok.start, line = tok.line, col = tok.col, comments = comment_texts(tok.comments) }
      local nx = lx:peek()
      if nx.type == "open" then
        local open = lx:next()
        if #open.comments > 0 then node.inner_comments = comment_texts(open.comments) end
        local children, close, at_eof = parse_pairs(lx, depth + 1)
        node.children = children
        node.stop, node.stop_line = close.stop, close.line
        if at_eof then node.unclosed = true end
      elseif nx.type == "string" and nx.line ~= tok.line then
        -- Source files keep one pair per line. A lone token followed by a
        -- string on a later line is an orphan (e.g. a deleted key); pairing it
        -- would shift every later key/value, so recover by line instead.
        node.missing_value = true
        node.stop, node.stop_line = tok.stop, tok.line
        lx:diag("orphan_token", tok.line, tok.col, tok.value)
      elseif nx.type == "string" then
        local v = lx:next()
        node.value, node.value_token, node.value_quoted = v.value, v.raw, v.quoted
        node.value_start, node.value_line, node.value_col = v.start, v.line, v.col
        node.stop, node.stop_line = v.stop, v.line
        if #v.comments > 0 then node.inner_comments = comment_texts(v.comments) end
      else
        -- Key followed by '}' or EOF: the value is missing.
        node.missing_value = true
        node.stop, node.stop_line = tok.stop, tok.line
        lx:diag("missing_value", tok.line, tok.col, tok.value)
      end
      -- Optional platform conditional such as [$WIN32].
      local cond = lx:peek()
      if cond.type == "string" and not cond.quoted and cond.value:match("^%[.*%]$") and #cond.comments == 0 then
        lx:next()
        node.condition = cond.value
        node.stop, node.stop_line = cond.stop, cond.line
      end
      nodes[#nodes + 1] = node
      last = node
    end
  end
end

-- parse(text) -> { bom = bool, nodes = {...}, diagnostics = {...}, invalid_utf8 = bool }
function M.parse(text)
  local bom = text:sub(1, 3) == BOM
  local lx = new_lexer(text, bom and 4 or 1)
  local nodes, eof = parse_pairs(lx, 0)
  local doc = { bom = bom, nodes = nodes, diagnostics = lx.diagnostics, trailing_comments = comment_texts(eof.comments) }
  if utf8.len(text, bom and 4 or 1) == nil then
    doc.invalid_utf8 = true
    doc.diagnostics[#doc.diagnostics + 1] = { kind = "invalid_utf8" }
  end
  return doc
end

-- Helpers over the node tree. Keys compare case-insensitively like Valve KV.
function M.find(nodes, key)
  if not nodes then return nil end
  local lower = key:lower()
  for _, n in ipairs(nodes) do
    if n.key and n.key:lower() == lower then return n end
  end
end

function M.find_all(nodes, key)
  local out, lower = {}, key:lower()
  for _, n in ipairs(nodes or {}) do
    if n.key and n.key:lower() == lower then out[#out + 1] = n end
  end
  return out
end

function M.value(nodes, key)
  local n = M.find(nodes, key)
  return n and n.value
end

function M.walk(nodes, fn, depth, parent)
  depth = depth or 0
  for _, n in ipairs(nodes or {}) do
    fn(n, depth, parent)
    if n.children then M.walk(n.children, fn, depth + 1, n) end
  end
end

-- Parses a rank array such as "900 1050 1200 1400". Returns a list of numbers
-- when every whitespace separated part is numeric, otherwise nil.
function M.rank_values(value)
  if value == nil then return nil end
  local out = {}
  for part in value:gmatch("%S+") do
    local n = tonumber(part)
    if n == nil or n ~= n or n == math.huge or n == -math.huge then return nil end
    out[#out + 1] = n
  end
  if #out == 0 then return nil end
  return out
end

-- "%bonus_atk" -> "bonus_atk"; nil when the value is not a placeholder.
function M.placeholder(value)
  if type(value) ~= "string" then return nil end
  return value:match("^%%([%w_]+)$")
end

M.BOM = BOM
return M
