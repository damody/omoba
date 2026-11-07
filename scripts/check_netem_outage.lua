-- Verdict for a split run with --netem-outage (E060): proxy outage evidence,
-- client STALLED/READY or link-lost per window, a rejoin of the same match after
-- every link-lost window (hero held, resume token accepted), and no desync/termination.
-- Usage: tools/lua/lua.exe scripts/check_netem_outage.lua --run SPLIT_RUN_DIR
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}

-- Must match omoba_core::lockstep_timing::LINK_LOST_AFTER.
M.LINK_LOST_AFTER_MS=30000

local function read(file) return path.is_file(file) and path.read(file) or nil end
local function count(text,needle)
  local n,at=0,1
  while text do local i=text:find(needle,at,true); if not i then break end; n=n+1; at=i+#needle end
  return n
end

function M.check(root)
  local gates={}
  local function gate(name,ok,detail) gates[#gates+1]={name=name,status=ok and 'PASS' or 'FAIL',detail=detail} end
  local evidence_file=path.join(root,'netem','proxy-evidence.json')
  local proxy=path.is_file(evidence_file) and json.read(evidence_file) or nil
  gate('proxy-evidence',proxy~=nil and (proxy.schema_version or 0)>=2 and proxy.status=='PASS',
    proxy and ('schema=%s status=%s'):format(proxy.schema_version,proxy.status) or 'missing '..evidence_file)
  local plan_file=path.join(root,'server','match-plan.json')
  local plan=path.is_file(plan_file) and json.read(plan_file) or {players={}}
  local human_of_team={}
  for _,player in ipairs(plan.players or {}) do if player.bot==false then human_of_team[player.team_id]=player.player_id end end
  local server=read(path.join(root,'server','server.stdout.log')) or ''
  local any_lost=false
  for index,o in ipairs(proxy and proxy.outages or {}) do
    local label=('outage-%d-team-%d'):format(index,o.team_id)
    local lost=o.scheduled_duration_ms>=M.LINK_LOST_AFTER_MS
    any_lost=any_lost or lost
    -- A link-lost window may outlive the run (the client keeps rejoining until it ends).
    local ended=o.actual_end_ms~=nil or lost
    gate(label..'-applied',o.actual_start_ms~=nil and ended and (o.dropped_client_to_server or 0)>0 and (o.dropped_server_to_client or 0)>0,
      ('start=%s end=%s c2s=%s s2c=%s'):format(o.actual_start_ms,o.actual_end_ms,o.dropped_client_to_server,o.dropped_server_to_client))
    local player=human_of_team[o.team_id]
    local log=player and read(path.join(root,'p'..player,'runtime-p'..player..'.stderr.log')) or nil
    local stalled,recovered,link_lost=count(log,'authority link stalled'),count(log,'authority link recovered'),count(log,'authority link lost')
    if lost then
      gate(label..'-link-lost-verdict',link_lost==1 and stalled>=1,('player=%s stalled=%d lost=%d'):format(player,stalled,link_lost))
      -- Rejoin: the runtime reports it after the lost link, the server held the hero
      -- and admitted the same player again with its resume token.
      local log_text=log or ''
      local lost_at=log_text:find('authority link lost',1,true)
      local rejoined_at=lost_at and log_text:find('client-runtime rejoined',lost_at,true)
      local held=count(server,('hero held player=%s '):format(player))
      local readmitted=count(server,('KCP player rejoined player=%s '):format(player))
      gate(label..'-rejoined',rejoined_at~=nil and held>=1 and readmitted>=1,
        ('player=%s runtime_rejoined=%s hero_held=%d server_rejoined=%d'):format(player,tostring(rejoined_at~=nil),held,readmitted))
    else
      gate(label..'-stalled-then-recovered',stalled>=1 and recovered>=1 and link_lost==0,('player=%s stalled=%d recovered=%d lost=%d'):format(player,stalled,recovered,link_lost))
    end
  end
  gate('outages-present',proxy~=nil and #(proxy.outages or {})>0,'no --netem-outage windows in evidence')
  gate('server-link-lost',(count(server,'KCP link lost')>0)==any_lost,('expected=%s seen=%d'):format(any_lost,count(server,'KCP link lost')))
  gate('no-observer-mismatch',count(server,'observer mismatch')==0,('count=%d'):format(count(server,'observer mismatch')))
  gate('no-termination',count(server,'secure match terminated')==0,('count=%d'):format(count(server,'secure match terminated')))
  local ok=true
  for _,g in ipairs(gates) do ok=ok and g.status=='PASS' end
  return ok,gates
end

if arg and arg[0] and arg[0]:match('check_netem_outage%.lua$') then
  local options=b.lib('args').parse(arg)
  local root=assert(options.run,'--run SPLIT_RUN_DIR required')
  local ok,gates=M.check(path.absolute(root,b.root))
  for _,g in ipairs(gates) do print(('%-4s %-36s %s'):format(g.status,g.name,g.detail or '')) end
  print(ok and 'netem outage verdict: PASS' or 'netem outage verdict: FAIL')
  os.exit(ok and 0 or 1)
end
return M
