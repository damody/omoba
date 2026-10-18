-- Owns generations; all builds, selection, admission and cleanup remain in the role workflow.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local launch=require('moba_role_launch')
local M={}
function M.supported(options)
 return not (options.prepare_only or options.server_only or options.connect or options.finish_timeout_seconds
  or next(options.local_players or {}))
end
function M.round_args(arguments,recipe,config,output,bootstrap)
 local result={};local index=1
 while index<=#arguments do
  local key=arguments[index]
  assert(key~='--managed-session-bootstrap','managed bootstrap is coordinator-owned')
  if key=='--recipe' or key=='--config' or key=='--output' then index=index+2
  else result[#result+1]=key;index=index+1 end
 end
 for _,pair in ipairs({{'--recipe',recipe},{'--config',config},{'--output',output},{'--managed-session-bootstrap',bootstrap}}) do
  result[#result+1]=pair[1];result[#result+1]=pair[2]
 end
 return result
end
function M.run(arguments,options,root,run_role)
 assert(M.supported(options),'unsupported managed provider')
 assert(type(run_role)=='function','role workflow required')
 assert(not p.exists(root),'session output must be new')
 p.mkdir_p(p.parent(root));assert(require('lfs').mkdir(root),'cannot reserve session output')
 local ok,result=xpcall(function()
  local author=launch.load_recipe(options)
  local players={}
  for _,player in ipairs(assert(author.players)) do
   if player.bot==false then players[#players+1]=player.player_id end
  end
  table.sort(players);assert(#players>=1 and #players<=10,'managed provider needs 1..10 humans')
  for index,id in ipairs(players)do
   assert(math.type(id)=='integer' and id>0 and id<=4294967295 and id~=players[index-1],'invalid managed human roster')
  end
  local recipe=p.join(root,'author-plan.json')
  local config=p.join(root,'base-game.toml')
  j.write(recipe,author,false)
  p.write(config,p.read(p.absolute(options.config or 'omb/game.toml',b.root),true),false,true)
  local hashes={recipe=h.sha256(recipe),config=h.sha256(config)}
  local identity=p.join(root,'session-origin.json')
  j.write(identity,{schema='omoba-managed-session-origin/1',output=root,created=b.lib('time').utc_timestamp(),
   recipe_sha256=hashes.recipe,config_sha256=hashes.config,players=players},false)
  local session=h.sha256(identity)
  local generation=1
  while true do
   assert(h.sha256(recipe)==hashes.recipe and h.sha256(config)==hashes.config,'managed author snapshot changed')
   local round=p.join(root,'round-'..generation)
   assert(not p.exists(round) and require('lfs').mkdir(round),'generation directory must be new')
   local bootstrap=p.join(round,'bootstrap.json')
   j.write(bootstrap,{schema=require('moba_managed_control').bootstrap_schema,session_id=session,generation=generation},false)
   local plan=assert(run_role(M.round_args(arguments,recipe,config,p.join(round,'match'),bootstrap)),'role workflow returned no plan')
   local next_round=plan.managed_next
   if not next_round then
    j.write(p.join(root,'completed.json'),{schema='omoba-managed-session-completion/1',generation=generation,
     next_match_started=false},false)
    return plan
   end
   assert(next_round.kind=='start_selection' and next_round.session_id==session and next_round.generation==generation+1,
    'role returned a foreign generation')
   assert(type(next_round.players)=='table' and #next_round.players==#players,'managed roster changed')
   for index,id in ipairs(players) do assert(next_round.players[index]==id,'managed human changed') end
   -- A fresh role invocation reopens selection and creates every runtime, renderer and authority anew.
   generation=next_round.generation
  end
 end,debug.traceback)
 if not ok then
  p.append(p.join(root,'errors.md'),'# Managed session failed\n\n'..tostring(result)..
   '\n\nDecision: stop this session; do not reuse finalized selection, adopt processes or automatically retry.\n')
  error(result,0)
 end
 return result
end
return M
