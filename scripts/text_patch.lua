-- Exact-text patcher that keeps each file's line endings (CRLF stays CRLF).
-- Use it instead of `sed -i` (Git Bash sed rewrites CRLF files to LF, E025/E061).
--
--   tools\lua\lua.exe scripts\text_patch.lua <spec.lua> [--dry]
--
-- The spec returns { { file = <path>, edits = { { old = <text>, new = <text> }
--   | { from = <text>, to = <text>, new = <text> }, ... } }, ... }.
-- `old` must occur exactly once; `from`/`to` replace the span from `from`
-- through the first `to` after it. Matching ignores CR, so a spec written
-- with LF matches CRLF files. Every edit of every file is checked before any
-- file is written: one failed match changes nothing. Write long Lua strings as
-- [==[ ... ]==] so code containing `]]` does not end them early (E026).
local M = {}

local function read(file)
  local f = io.open(file, "rb"); if not f then return nil end
  local text = f:read("a"); f:close(); return text
end

-- Returns { { file, text, crlf, edits } } or raises with the first problem.
function M.plan(spec)
  local out = {}
  for _, item in ipairs(spec) do
    local raw = assert(read(item.file), "missing file " .. tostring(item.file))
    local crlf = raw:find("\r\n", 1, true) ~= nil
    local text = raw:gsub("\r\n", "\n")
    for i, e in ipairs(item.edits or {}) do
      local where = item.file .. " edit " .. i
      local s, en
      if e.old then
        local old = e.old:gsub("\r\n", "\n")
        s, en = text:find(old, 1, true)
        assert(s, where .. ": old text not found")
        assert(not text:find(old, en + 1, true), where .. ": old text not unique")
      else
        assert(e.from and e.to, where .. ": needs old, or from and to")
        s = assert(text:find((e.from:gsub("\r\n", "\n")), 1, true), where .. ": from not found")
        local _, te = text:find((e.to:gsub("\r\n", "\n")), s, true)
        en = assert(te, where .. ": to not found")
      end
      text = text:sub(1, s - 1) .. e.new:gsub("\r\n", "\n") .. text:sub(en + 1)
    end
    out[#out + 1] = { file = item.file, text = crlf and text:gsub("\n", "\r\n") or text, crlf = crlf, edits = #(item.edits or {}) }
  end
  return out
end

function M.apply(spec, dry)
  local planned = M.plan(spec)
  if dry then return planned end
  for _, p in ipairs(planned) do
    local f = assert(io.open(p.file, "wb")); f:write(p.text); f:close()
  end
  return planned
end

if arg and arg[0] and arg[0]:match("text_patch%.lua$") then
  local spec_file = assert(arg[1], "usage: text_patch.lua <spec.lua> [--dry]")
  local dry = arg[2] == "--dry"
  local ok, result = pcall(M.apply, dofile(spec_file), dry)
  if not ok then io.stderr:write(tostring(result), "\n"); os.exit(1) end
  for _, p in ipairs(result) do
    print((dry and "would patch " or "patched ") .. p.file .. " (" .. p.edits .. " edits" .. (p.crlf and ", CRLF kept" or "") .. ")")
  end
  os.exit(0)
end
return M
