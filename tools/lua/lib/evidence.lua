local json = require("tools.lua.lib.json")
local path = require("tools.lua.lib.path")
local hash = require("tools.lua.lib.hash")
local time = require("tools.lua.lib.time")
local M = {}

function M.create_run(directory)
  assert(not path.exists(directory), "run already exists; refusing overwrite: " .. directory)
  return path.mkdir_p(directory)
end
function M.write_manifest(directory, manifest)
  manifest.created_utc = manifest.created_utc or time.utc_timestamp()
  json.write(path.join(directory, "manifest.json"), manifest, false)
end
function M.timeline(directory, event) json.append_jsonl(path.join(directory,"timeline.jsonl"),event) end
function M.verdict(directory,status,reasons)
  assert(status=='PASS' or status=='FAIL' or status=='UNVERIFIED','invalid verdict')
  json.write(path.join(directory,'verdict.json'),{status=status,reasons=reasons or {},created_utc=time.utc_timestamp()},false)
end
function M.process_record(pid, executable, role, team_id, player_id)
  return {pid=pid,path=path.absolute(executable),sha256=hash.sha256(executable),role=role,team_id=team_id or 0,player_id=player_id or 0}
end
function M.weights20(values)
  assert(type(values)=='table' and #values==20,'weights must contain exactly 20 entries')
  local total=0;for i,value in ipairs(values) do value=math.tointeger(value);assert(value and value>=0,'weight '..i..' must be a non-negative integer');total=total+value;values[i]=value end
  assert(total>0,'weight sum must be greater than zero');return values
end
function M.read(file) return json.read(file) end
return M
