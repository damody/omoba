-- Generated art for heroes, abilities and items that have no original nobu art (M6, README D17:
-- missing art is generated with Grok Build's image_gen). Each image is reviewed by eye before it is
-- imported; the import resamples it and records its provenance.
--
--   hero / ability / item  omfue/Content/Omoba/UI/gen-<kind>-<id>.png (EDGE pixels)
--   cue                    omfue/SourceArt/CueReferences/gen-cue-<id>.png (CUE_EDGE pixels; a
--                          reference for building the effect, never loaded by the game)
--
-- All rows live in omfue/Content/Omoba/UI/generated-art-v1.json. HUD sync uses a row only where
-- the nobu index has no original (original nobu art wins, hand-assigned rows win over both).
--
--   tools\lua\lua.exe scripts\generated_art.lua --import STAGE.json [--root DIR]
--   tools\lua\lua.exe scripts\generated_art.lua --check [--root DIR]
--
-- STAGE.json: { "generated": "YYYY-MM-DD", "assets": [ { "kind", "id", "source" (the original
-- image grok wrote), "prompt", "attempts", "rejected" ([{attempt, reason}]), "review", "model",
-- "session" } ] }. Importing an id again replaces its row and file.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")
local hash = b.lib("hash")

local M = {}
M.SCHEMA = "omoba.generated-art.v1"
M.UI_DIR = "omfue/Content/Omoba/UI"
M.CUE_DIR = "omfue/SourceArt/CueReferences"
M.INDEX = "generated-art-v1.json"
M.EDGE = 256
M.CUE_EDGE = 512
M.TOOL = "grok-build-image_gen"
M.KINDS = { hero = true, ability = true, item = true, cue = true }

function M.file(kind, id)
  assert(M.KINDS[kind], "generated art kind " .. tostring(kind))
  assert(type(id) == "string" and id:match("^[a-z0-9_]+$"), kind .. " id " .. tostring(id))
  return "gen-" .. kind .. "-" .. id .. ".png"
end

-- index table -> { [kind] = { [id] = row } }; nil -> empty
function M.rows(index)
  local out = { hero = {}, ability = {}, item = {}, cue = {} }
  if not index then return out end
  assert(index.schema == M.SCHEMA and type(index.entries) == "table", "invalid generated art index")
  for _, row in ipairs(index.entries) do
    assert(row.file == M.file(row.kind, row.id), "generated art file does not match its id: " .. tostring(row.id))
    assert(not out[row.kind][row.id], "duplicate generated art: " .. row.kind .. ":" .. row.id)
    out[row.kind][row.id] = row
  end
  return out
end

-- The nobu icon index with every non-original entry that has generated art turned into
-- status "generated" (file gen-<kind>-<id>.png). Originals are never replaced. Returns a copy.
function M.overlay(nobu_index, generated_index)
  local gen = M.rows(generated_index)
  local out = { schema = nobu_index.schema, entries = {} }
  for k, v in pairs(nobu_index) do if k ~= "entries" then out[k] = v end end
  for _, e in ipairs(nobu_index.entries or {}) do
    local row = e
    local g = e.status ~= "original" and gen[e.kind] and gen[e.kind][e.id]
    if g then
      row = {}
      for k, v in pairs(e) do row[k] = v end
      row.status, row.file, row.reason = "generated", g.file, nil
    end
    out.entries[#out.entries + 1] = row
  end
  return out
end

local function read_json(p)
  local f = io.open(p, "rb")
  if not f then return nil end
  local s = f:read("a"); f:close()
  return json.decode(s)
end

function M.read_index(root)
  return read_json(root .. "/" .. M.UI_DIR .. "/" .. M.INDEX)
end

function M.encode_index(index)
  table.sort(index.entries, function(x, y) return x.kind .. ":" .. x.id < y.kind .. ":" .. y.id end)
  local lines = {}
  for _, row in ipairs(index.entries) do lines[#lines + 1] = "  " .. json.encode(row) end
  return '{"schema":' .. json.encode(M.SCHEMA) .. ',"edge":' .. M.EDGE .. ',"cue_edge":' .. M.CUE_EDGE
    .. ',"entries":[\n' .. table.concat(lines, ",\n") .. "\n]}\n"
end

function M.import(root, stage)
  local host = b.lib("host")
  local index = M.read_index(root) or { schema = M.SCHEMA, entries = {} }
  local by = {}
  for i, row in ipairs(index.entries) do by[row.kind .. ":" .. row.id] = i end
  assert(type(stage.generated) == "string" and stage.generated:match("^%d%d%d%d%-%d%d%-%d%d$"), "stage needs generated=YYYY-MM-DD")
  local count = 0
  for _, a in ipairs(stage.assets) do
    local file = M.file(a.kind, a.id)
    assert(type(a.prompt) == "string" and #a.prompt > 0, a.id .. ": prompt")
    assert(type(a.review) == "string" and #a.review > 0, a.id .. ": every image is reviewed by eye before import")
    local dir = root .. "/" .. (a.kind == "cue" and M.CUE_DIR or M.UI_DIR)
    b.lib("path").mkdir_p(dir)
    local edge = a.kind == "cue" and M.CUE_EDGE or M.EDGE
    local report = host.call("png_square_icon", { source = a.source, output = dir .. "/" .. file, edge = edge })
    local row = { kind = a.kind, id = a.id, file = file, edge = edge, sha256 = report.output_sha256,
      original_sha256 = report.source_sha256, original_size = { report.source_width, report.source_height },
      tool = M.TOOL, model = a.model or "not exposed", session = a.session, generated = stage.generated,
      attempts = a.attempts or 1, rejected = a.rejected, review = a.review, prompt = a.prompt }
    if by[a.kind .. ":" .. a.id] then index.entries[by[a.kind .. ":" .. a.id]] = row
    else index.entries[#index.entries + 1] = row; by[a.kind .. ":" .. a.id] = #index.entries end
    count = count + 1
  end
  local f = assert(io.open(root .. "/" .. M.UI_DIR .. "/" .. M.INDEX, "wb"))
  f:write(M.encode_index(index)); f:close()
  return count, #index.entries
end

function M.check(root)
  local index = M.read_index(root)
  if not index then return 0 end
  M.rows(index)
  for _, row in ipairs(index.entries) do
    local p = root .. "/" .. (row.kind == "cue" and M.CUE_DIR or M.UI_DIR) .. "/" .. row.file
    assert(hash.sha256(p) == row.sha256, "generated art file changed or missing: " .. p)
  end
  return #index.entries
end

function M.main(args)
  local root, stage, check = b.root, nil, false
  local i = 1
  while i <= #args do
    if args[i] == "--import" then stage = args[i + 1]; i = i + 1
    elseif args[i] == "--root" then root = args[i + 1]; i = i + 1
    elseif args[i] == "--check" then check = true
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  if stage then
    local n, total = M.import(root, assert(read_json(stage), "cannot read " .. stage))
    print("imported " .. n .. " generated image(s); index has " .. total)
  end
  if check or not stage then print("generated art index ok: " .. M.check(root) .. " row(s)") end
  return 0
end

if not GENERATED_ART_AS_MODULE then os.exit(M.main(arg)) end
return M
