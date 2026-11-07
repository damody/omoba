-- Source text files as UTF-8 (history_nobu, E169 migration). The Dota localisation files
-- (resource/addon_*.txt) are UTF-16LE with a BOM; everything else is UTF-8. Decoding is plain
-- Lua: no PowerShell, Python or source script runs. Offsets recorded in a catalog refer to the
-- decoded UTF-8 text, and verify decodes the same way, so spans stay checkable.
local M = {}

-- UTF-16LE (after its FF FE BOM) -> UTF-8. Surrogate pairs join; a lone surrogate is an error.
function M.utf16le_to_utf8(bytes)
  local out, n = {}, #bytes
  assert(n % 2 == 0, "UTF-16LE text has an odd byte count")
  local i = 1
  while i <= n do
    local lo, hi = bytes:byte(i, i + 1)
    local unit = lo + hi * 256
    i = i + 2
    if unit >= 0xD800 and unit <= 0xDBFF then
      assert(i <= n, "UTF-16LE text ends inside a surrogate pair")
      local lo2, hi2 = bytes:byte(i, i + 1)
      local low = lo2 + hi2 * 256
      assert(low >= 0xDC00 and low <= 0xDFFF, "UTF-16LE high surrogate without a low one")
      unit = 0x10000 + (unit - 0xD800) * 0x400 + (low - 0xDC00)
      i = i + 2
    else
      assert(unit < 0xDC00 or unit > 0xDFFF, "UTF-16LE lone low surrogate")
    end
    out[#out + 1] = utf8.char(unit)
  end
  return table.concat(out)
end

-- UTF-8 -> UTF-16LE with a BOM (test fixtures write source-shaped localisation files).
function M.utf8_to_utf16le(text)
  local out = { "\255\254" }
  for _, cp in utf8.codes(text) do
    if cp >= 0x10000 then
      local v = cp - 0x10000
      local hi, lo = 0xD800 + (v >> 10), 0xDC00 + (v & 0x3FF)
      out[#out + 1] = string.char(hi & 0xFF, hi >> 8, lo & 0xFF, lo >> 8)
    else
      out[#out + 1] = string.char(cp & 0xFF, cp >> 8)
    end
  end
  return table.concat(out)
end

-- A file's bytes exactly as stored (whole-file hashes identify the file, not its decoding).
function M.read_raw(file)
  local f = assert(io.open(file, "rb"))
  local bytes = f:read("a")
  f:close()
  return bytes
end

-- A file's text as UTF-8: a UTF-16LE BOM is decoded (and dropped); other files are returned as is.
function M.read(file)
  local f = assert(io.open(file, "rb"))
  local bytes = f:read("a")
  f:close()
  if bytes:sub(1, 2) == "\255\254" then return M.utf16le_to_utf8(bytes:sub(3)) end
  return bytes
end

-- Expands Dota tooltip placeholders: %name% -> the special's value (`values[name]`), %% -> %.
-- An unknown name is left as written and reported, so a missing value is never invented.
function M.expand(text, values)
  local unknown = {}
  local expanded = tostring(text or ""):gsub("%%([%w_]*)%%", function(name)
    if name == "" then return "%" end
    local value = values[name]
    if value == nil then unknown[#unknown + 1] = name; return nil end
    return tostring(value)
  end)
  return expanded, unknown
end

return M
