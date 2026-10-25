local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local p=b.lib('path')
local root=p.join(b.root,'scripts/lua_data')
local build=assert(loadfile(p.join(root,'templates/summon_guidance.lua')))()
local cases=0
local function guidance(rows)
  return build({include=function(file)assert(file=='templates/summons.lua');return rows end})
end
local nearest=guidance({{id='unrelated_fixture'}}).unrelated_fixture
assert(nearest:find('最近',1,true) and not nearest:find('主人',1,true));cases=cases+1
assert(guidance({{id='other',target_priority='nearest'}}).other==nearest);cases=cases+1
local focus=guidance({{id='renamed_fixture',target_priority='owner_focus'}}).renamed_fixture
for _,word in ipairs({'主人目前指定','可見','射程內','可被攻擊','停止攻擊','最近的合法敵人'})do assert(focus:find(word,1,true));cases=cases+1 end
assert(guidance({{id='renamed_fixture',target_priority='nearest'}}).renamed_fixture==nearest);cases=cases+1
for _,bad in ipairs({'unknown','OwnerFocus',false,42,{}})do assert(not pcall(guidance,{{id='bad',target_priority=bad}}));cases=cases+1 end
assert(not pcall(guidance,{{id='same'},{id='same'}}));cases=cases+1
assert(not pcall(guidance,{{id=''}}));cases=cases+1
-- Load the actual include graph as authoring does, then mutate only the summon
-- policy to prove the real skill description follows it without a UI edit.
local function abilities(override)
  local cache={};local ctx={}
  ctx.include=function(file)
    if file=='templates/summons.lua' and override then return {{id='saika_gunner',target_priority=override}} end
    if cache[file] then return cache[file] end
    local value=assert(loadfile(p.join(root,file)))()(ctx);cache[file]=value;return value
  end
  return ctx.include('templates/abilities.lua')
end
local function description(rows)
  for _,row in ipairs(rows)do if row.id=='saika_reinforcements' then return row.description end end
  error('actual summoning skill missing')
end
local actual=description(abilities());assert(actual:find(focus,1,true));cases=cases+1
local changed=description(abilities('nearest'));assert(changed:find(nearest,1,true) and not changed:find(focus,1,true));cases=cases+1
assert(not pcall(abilities,'unknown'));cases=cases+1
print(('summon guidance: %d/%d passed; actual authored tooltip and policy mutations'):format(cases,cases))
