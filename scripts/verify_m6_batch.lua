-- M6 batch acceptance (E265): when line A unlocks a batch of docs/plans/2026-10-28-history-nobu-full-port/
-- m6-waves.md, seat that batch's heroes and check them in play.
--   * headless quick, the batch roster on team 1 and swapped to team 2, `--matches` seeds each;
--   * teamfight stress: the same roster with every seat on the mid lane;
--   * optionally (`--ue`) one live Unreal match of at most 10 minutes (machine lock, time box).
-- Bots cast every ability through the derived v3 policies (scripts/lua_data/bot_auto_policies.lua);
-- every match records per-hero, per-ability statistics (OMB_HEADLESS_HERO_STATS, E265). The report
-- (report.md, report.json) lists, per batch hero, each ability's bot inputs, accepted casts and cast
-- trace results, and flags the active abilities no bot ever cast.
--
-- Usage: tools/lua/lua.exe scripts/verify_m6_batch.lua (--batch N [--group G] | --heroes a,b,...) [--matches 6]
--   [--parallel 6] [--ue] [--output NEW_DIRECTORY] [--no-build] [--report-only]
-- --report-only --output DIR writes DIR/report.md again from the match reports already in DIR.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, process, json, time = b.lib('path'), b.lib('process'), b.lib('json'), b.lib('time')
local compiled = require('moba_compiled_content')

local options = { matches = 6, parallel = 6, group = 1 }
do
  local i = 1
  while i <= #arg do
    local v = arg[i]
    if v == '--batch' then options.batch = assert(math.tointeger(tonumber(arg[i + 1])), '--batch N'); i = i + 1
    elseif v == '--group' then options.group = assert(math.tointeger(tonumber(arg[i + 1])), '--group G'); i = i + 1
    elseif v == '--matches' then options.matches = assert(math.tointeger(tonumber(arg[i + 1])), '--matches K'); i = i + 1
    elseif v == '--parallel' then options.parallel = assert(math.tointeger(tonumber(arg[i + 1])), '--parallel P'); i = i + 1
    elseif v == '--output' then options.output = arg[i + 1]; i = i + 1
    elseif v == '--heroes' then options.heroes = arg[i + 1]; i = i + 1
    elseif v == '--ue' then options.ue = true
    elseif v == '--item-coverage' then options.item_coverage = true
    elseif v == '--no-build' then options.no_build = true
    elseif v == '--report-only' then options.report_only = true; options.no_build = true
    elseif v == '--help' then
      print('Usage: tools/lua/lua.exe scripts/verify_m6_batch.lua --batch N [--group G] [--matches 6] [--parallel 6] [--ue] [--item-coverage] [--output DIR] [--no-build] [--report-only]')
      return
    else error('unknown argument ' .. v) end
    i = i + 1
  end
end
assert(options.batch or options.heroes, '--batch N or --heroes a,b,... is required')
options.batch = options.batch or 0
assert(options.matches >= 1 and options.matches <= 100 and options.parallel >= 1 and options.parallel <= 16, 'matches 1..100, parallel 1..16')

-- 1. The batch's heroes (source ids) from v3_waves, then their packages (= hero ids).
V3_WAVES_AS_MODULE = true
local waves = dofile(path.join(b.root, 'docs', 'character_pipeline', 'tools', 'v3_waves.lua')).build(b.root).waves
V3_WAVES_AS_MODULE = nil
-- `--heroes` seats those hero ids instead (re-checking shipped heroes, or a batch subset).
local wave = { heroes = {} }
if options.heroes then
  for id in options.heroes:gmatch('[^,]+') do wave.heroes[#wave.heroes + 1] = { package = id } end
else
  wave = assert(waves[options.batch], 'no batch ' .. options.batch .. ' (1..' .. #waves .. ')')
end
local package_of = {}
local lfs = require('lfs')
local packages_dir = path.join(b.root, 'docs', 'character_pipeline', 'packages')
for name in lfs.dir(packages_dir) do
  local file = path.join(packages_dir, name, 'v3.lua')
  if name:sub(1, 1) ~= '.' and path.is_file(file) then
    -- The whole document (E339): a long header put the hero id past the first 2000 bytes.
    local id = path.read(file):match('kind%s*=%s*"hero"%s*,%s*id%s*=%s*"([%w_]+)"')
    if id then package_of[id] = name end
  end
end
-- Playable: every hero of the content hero table (templates/heroes.lua, which merges the authored
-- heroes and the v3 heroes of templates/v3_test_heroes.lua) that is neither test_only nor a
-- tombstone (E363: a v3 hero marked playable is a v3_test_heroes row, not an authored one).
local content = require('content_builder').new(path.join(b.root, 'scripts', 'lua_data'))
local playable = {}
for _, hero in ipairs(content.include('templates/heroes.lua')) do
  if not hero.test_only and not hero.tombstone then playable[hero.id] = true end
end
local batch_heroes, missing = {}, {}
for _, hero in ipairs(wave.heroes) do
  local package = hero.package or package_of[hero.id]
  if package and playable[package] then batch_heroes[#batch_heroes + 1] = package
  else missing[#missing + 1] = (hero.id or package) .. (package and hero.id and (' (' .. package .. ')') or '') end
end
local first = (options.group - 1) * 10 + 1
local group = {}
for i = first, math.min(first + 9, #batch_heroes) do group[#group + 1] = batch_heroes[i] end
assert(#group > 0, ('batch %d group %d has no playable hero; not yet playable: %s'):format(options.batch, options.group,
  table.concat(missing, ', ')))

-- 2. Recipes: the shipped role plan with the group seated (team 1 first, then team 2; seats the group
-- does not fill keep the shipped heroes), on the quick map; policies re-derived for the new seats.
-- Each team's share of the group takes the lanes its heroes fit best, derived from their data
-- (E357, hero_lane_roles.lua: attack range, primary attribute, heal / mobility hints, control).
local output
if options.output then output = path.absolute(options.output, b.root)
else output = path.join(b.root, 'target', 'm6-batch', ('batch%d-group%d-%d'):format(options.batch, options.group, os.time())) end
-- --report-only re-reads the match reports of an earlier run in that directory.
if options.report_only then assert(options.output and path.is_directory(output), '--report-only needs --output of an earlier run')
else assert(not path.exists(output), 'output must be a new directory: ' .. output) end
path.mkdir_p(output)
local lanes = require('hero_lane_roles')
local seating = {}
do
  local shipped = assert(loadfile(path.join(b.root, 'scripts', 'lua_data', 'moba_role_match.lua')))()
  local next_hero = 1
  for _, team in ipairs({ 1, 2 }) do
    local roles = {}
    for _, player in ipairs(shipped.players) do if player.team_id == team then roles[#roles + 1] = player.role end end
    local share = {}
    while #share < #roles and next_hero <= #group do share[#share + 1] = group[next_hero]; next_hero = next_hero + 1 end
    seating[#seating + 1] = lanes.assign(share, roles) or {}
    for _, role in ipairs(roles) do
      if seating[#seating][role] then print(('seat: %s lane role %s'):format(seating[#seating][role], role)) end
    end
  end
  assert(next_hero > #group, ('group of %d heroes does not fit the plan seats'):format(#group))
end
local function lua_role_map(map)
  local roles = {}
  for role in pairs(map) do roles[#roles + 1] = role end
  table.sort(roles)
  local parts = {}
  for _, role in ipairs(roles) do parts[#parts + 1] = ('[%q]=%q'):format(role, map[role]) end
  return '{' .. table.concat(parts, ',') .. '}'
end
-- --item-coverage (E358): the shop items no derived build reaches go into the builds
-- (bot_item_builds.lua cover / uncovered), so every shop item is obtained in every match.
if options.item_coverage then
  local builds = assert(loadfile(path.join(b.root, 'scripts', 'lua_data', 'bot_item_builds.lua')))()
  local shipped = assert(loadfile(path.join(b.root, 'scripts', 'lua_data', 'moba_role_match.lua')))()
  local uncovered = builds.uncovered(shipped.item_builds)
  print('item coverage: ' .. (#uncovered > 0 and table.concat(uncovered, ', ') or 'every shop item is in a build already'))
  for _, build in ipairs(builds.cover(shipped.item_builds, uncovered)) do
    print(('item coverage: %s builds %s'):format(build.role, table.concat(build.items, ', ')))
  end
end
local function recipe(name, swap, teamfight)
  local file = path.join(output, name .. '.lua')
  path.write(file, ([[
-- Generated by scripts/verify_m6_batch.lua (batch %d, group %d); recipe %s.
local root = %q
local plan = assert(loadfile(root .. '/scripts/lua_data/moba_role_match.lua'))()
plan.map_id = 'three_lane_quick_training'
local seating, swap, teamfight = {%s, %s}, %s, %s
for order, team in ipairs(swap and {2, 1} or {1, 2}) do
  for _, player in ipairs(plan.players) do
    if player.team_id == team and seating[order][player.role] then player.hero = seating[order][player.role] end
  end
end
if teamfight then for _, player in ipairs(plan.players) do player.lane = 'mid' end end
if %s then
  local builds = assert(loadfile(root .. '/scripts/lua_data/bot_item_builds.lua'))()
  plan.item_builds = builds.cover(plan.item_builds, builds.uncovered(plan.item_builds))
end
plan.ability_policies = assert(loadfile(root .. '/scripts/lua_data/bot_auto_policies.lua'))()(plan.ability_policies, plan.players)
return plan
]]):format(options.batch, options.group, name, b.root:gsub('\\', '/'), lua_role_map(seating[1]), lua_role_map(seating[2]), tostring(swap), tostring(teamfight),
  tostring(options.item_coverage == true)), true)
  return file
end
local recipes = {
  { name = 'quick', file = recipe('quick', false, false), group_team = 1 },
  { name = 'quick_swap', file = recipe('quick_swap', true, false), group_team = 2 },
  { name = 'teamfight', file = recipe('teamfight', false, true), group_team = 1 },
}

-- 3. Build once (the script DLL set and moba-headless) from this batch's own source snapshot into
-- its own cargo target (E347, source_snapshot.lua): no repo build lock. Both come from one snapshot
-- and their content hashes are checked equal (E332); they run from a private copy (E125).
local env = compiled.environment({ OMB_HEADLESS_HERO_STATS = '1' })
local bin = path.join(output, 'bin')
local exe = path.join(bin, 'moba-headless.exe')
if not options.report_only then
  require('source_snapshot').build_headless(b.root, 'verify_m6_batch',
    { env = { OMB_HEADLESS_HERO_STATS = '1' }, into = bin, no_build = options.no_build })
end

-- 4. Matches, `--parallel` at a time; never during a performance measurement (target/locks/performance).
local function wait_performance_lock()
  local lock = path.join(b.root, 'target', 'locks', 'performance')
  while path.exists(lock) do print('waiting: a performance measurement holds ' .. lock); time.sleep_ms(30000) end
end
local jobs = {}
for _, r in ipairs(recipes) do
  r.plan = path.join(output, r.name .. '.json')
  require('export_moba_role_plan').export(r.file, r.plan)
  for seed = 1, options.matches do
    jobs[#jobs + 1] = { recipe = r, seed = seed, report = path.join(output, ('%s-%02d.json'):format(r.name, seed)),
      log = path.join(output, ('%s-%02d.log'):format(r.name, seed)) }
  end
end
local running, next_job = {}, options.report_only and #jobs + 1 or 1
while next_job <= #jobs or #running > 0 do
  while next_job <= #jobs and #running < options.parallel do
    wait_performance_lock()
    local job = jobs[next_job]; next_job = next_job + 1
    job.pid = process.spawn(exe, { '--role-plan', job.recipe.plan, '--profile', '60', '--seed', tostring(job.seed),
      '--scripts-dir', bin, '--max-game-seconds', '1200', '--stall-game-seconds', '300', '--report', job.report },
      { cwd = b.root, env = env, stdout = job.log, stderr = job.log .. '.err' })
    running[#running + 1] = job
  end
  local still = {}
  for _, job in ipairs(running) do
    if process.wait(job.pid, 2000) then print(('done %s seed %d'):format(job.recipe.name, job.seed)) else still[#still + 1] = job end
  end
  running = still
end

-- 5. Report.
local function median(list)
  if #list == 0 then return 0 end
  table.sort(list); return list[(#list + 1) // 2]
end
local in_group = {}
for _, hero in ipairs(group) do in_group[hero] = true end
local summary = { batch = options.batch, group = options.group, heroes = group, not_playable = missing, recipes = {}, abilities = {} }
local ability_rows = {}
local item_rows = {}
-- Balance (E342): per hero over every seat, and per item over the seats that obtained it.
local hero_rows, holder_rows = {}, {}
for _, r in ipairs(recipes) do
  local row = { name = r.name, matches = 0, finished = 0, group_wins = 0, minutes = {}, failures = {} }
  for _, job in ipairs(jobs) do
    if job.recipe == r then
      row.matches = row.matches + 1
      local ok, report = pcall(json.read, job.report)
      if ok and report.success then
        row.finished = row.finished + 1
        row.minutes[#row.minutes + 1] = report.game_seconds / 60
        if report.winner_team == r.group_team then row.group_wins = row.group_wins + 1 end
        for _, hero in ipairs(report.hero_stats or {}) do
          local won = hero.team ~= nil and hero.team == report.winner_team
          local h = hero_rows[hero.hero] or { games = 0, wins = 0, kills = 0, deaths = 0, assists = 0, damage = 0, hero_damage = 0 }
          hero_rows[hero.hero] = h
          h.games, h.wins = h.games + 1, h.wins + (won and 1 or 0)
          h.kills, h.deaths, h.assists = h.kills + (hero.kills or 0), h.deaths + (hero.deaths or 0), h.assists + (hero.assists or 0)
          h.damage, h.hero_damage = h.damage + (hero.damage or 0), h.hero_damage + (hero.hero_damage or 0)
          for id, item in pairs(hero.items or {}) do
            if (item.obtained or 0) > 0 then
              local holder = holder_rows[id] or { games = 0, wins = 0 }
              holder_rows[id] = holder
              holder.games, holder.wins = holder.games + 1, holder.wins + (won and 1 or 0)
            end
          end
          -- Shop items over every seat (E336): how often each was obtained and used.
          for id, item in pairs(hero.items or {}) do
            local row = item_rows[id] or { obtained = 0, used = 0 }
            item_rows[id] = row
            row.obtained, row.used = row.obtained + (item.obtained or 0), row.used + (item.used or 0)
          end
          if in_group[hero.hero] then
            for ability, cast in pairs(hero.casts) do
              local key = hero.hero .. '|' .. ability
              local a = ability_rows[key] or { hero = hero.hero, ability = ability, slot = cast.slot, inputs = 0, accepted = 0, results = {} }
              ability_rows[key] = a
              a.inputs, a.accepted = a.inputs + cast.inputs, a.accepted + cast.accepted
              for result, n in pairs(cast.results or {}) do a.results[result] = (a.results[result] or 0) + n end
            end
          end
        end
      else
        row.failures[#row.failures + 1] = job.seed
      end
    end
  end
  row.median_minutes = median(row.minutes)
  summary.recipes[#summary.recipes + 1] = row
end
local kinds = {}
for _, file in ipairs({ 'templates/authored_heroes.lua', 'templates/v3_test_heroes.lua' }) do
  for _, a in ipairs(content.include(file).abilities or {}) do kinds[a.id] = a.ability_type end
end
for _, a in pairs(ability_rows) do a.kind = kinds[a.ability] or '?'; summary.abilities[#summary.abilities + 1] = a end
table.sort(summary.abilities, function(x, y) return x.hero == y.hero and x.slot < y.slot or x.hero < y.hero end)

-- 6. Optional live Unreal match (10 minutes at most, machine lock, autopilot screenshots).
if options.ue then
  local ue_out = path.join(output, 'ue')
  local result = process.run(b.lib('platform').lua_executable, { path.join(b.root, 'scripts', 'run_moba_role_ue.lua'),
    '--recipe', recipes[1].file, '--output', ue_out, '--wait-live-lock', '1800', '--autopilot', 'lane_v0',
    '--autopilot-shot-interval', '45', '--finish-timeout-seconds', '540', '--round-time-box-seconds', '560' },
    { cwd = b.root, check = false })
  local ue_log = path.join(ue_out, 'ue-p1.log')
  local text = path.is_file(ue_log) and path.read(ue_log) or ''
  summary.ue = { exit_code = result.exit_code, output = ue_out,
    result_ui = text:find('OM_MATCH_RESULT_UI', 1, true) ~= nil,
    result_screenshot = path.is_file(path.join(ue_out, 'result-p1.png')),
    time_boxed = path.is_file(path.join(ue_out, 'round-time-box.json')) }
end

json.write(path.join(output, 'report.json'), summary, true)
local md = { ('# M6 batch %d, group %d'):format(options.batch, options.group), '',
  'Heroes: ' .. table.concat(group, ', '), '' }
if #missing > 0 then md[#md + 1] = 'Not yet playable (skipped): ' .. table.concat(missing, ', '); md[#md + 1] = '' end
md[#md + 1] = '| recipe | finished | batch heroes win | median minutes | failed seeds |'
md[#md + 1] = '|---|---|---|---|---|'
for _, row in ipairs(summary.recipes) do
  md[#md + 1] = ('| %s | %d/%d | %d | %.1f | %s |'):format(row.name, row.finished, row.matches, row.group_wins,
    row.median_minutes, table.concat(row.failures, ' '))
end
md[#md + 1] = ''
-- `casts` is the larger of the cast trace's accepted count and the authority's Ability facts
-- (E316: B08 無盡牙 left Ability facts but no trace row and was reported as never cast; such a
-- row is now NO TRACE, an anomaly to hand over, not a never-cast); `bot inputs` counts every
-- cast order the bots sent, repeats included.
md[#md + 1] = '| hero | ability | kind | bot inputs | casts | trace | note |'
md[#md + 1] = '|---|---|---|---|---|---|---|'
local never = 0
for _, a in ipairs(summary.abilities) do
  local results = {}
  for result, n in pairs(a.results) do results[#results + 1] = result .. ' ' .. n end
  table.sort(results)
  if (a.accepted or 0) ~= (a.results.accepted or 0) then results[#results + 1] = 'facts ' .. (a.accepted or 0) end
  local castable = a.kind == 'active' or a.kind == 'ultimate' or a.kind == 'toggle'
  local casts = math.max(a.results.accepted or 0, a.accepted or 0)
  local note = castable and casts == 0 and 'NEVER CAST'
    or castable and (a.results.accepted or 0) == 0 and 'NO TRACE' or ''
  if note == 'NEVER CAST' then never = never + 1 end
  md[#md + 1] = ('| %s | %s | %s | %d | %d | %s | %s |'):format(a.hero, a.ability, a.kind, a.inputs, casts, table.concat(results, ', '), note)
end
-- Shop items (E336): every item the shop sells, obtained (bought or assembled) and used over all
-- seats; an item never obtained, or an active item obtained but never used, is flagged.
local shop_items = {}
for _, item in ipairs(require('content_builder').new(path.join(b.root, 'scripts', 'lua_data')).include('templates/moba_items.lua')) do
  if not item.test_only then shop_items[#shop_items + 1] = item end
end
table.sort(shop_items, function(x, y) return x.id < y.id end)
summary.items = {}
local unused_items = 0
md[#md + 1] = ''
md[#md + 1] = '| item | active | obtained | used | note |'
md[#md + 1] = '|---|---|---|---|---|'
for _, item in ipairs(shop_items) do
  local row = item_rows[item.id] or { obtained = 0, used = 0 }
  -- An unused active that only passed through on the way to an obtained item it builds into
  -- (dark_boots -> dark_boots_1) was assembled, not ignored (E354).
  local upgraded = false
  for _, other in ipairs(shop_items) do
    for _, part in ipairs(other.recipe or {}) do
      if part == item.id and (item_rows[other.id] or {}).obtained then upgraded = true end
    end
  end
  local note = row.obtained == 0 and 'NEVER OBTAINED'
    or (item.active and row.used == 0) and (upgraded and 'unused, built into an upgrade' or 'NEVER USED') or ''
  if note == 'unused, built into an upgrade' then unused_items = unused_items - 1 end
  if note ~= '' then unused_items = unused_items + 1 end
  summary.items[item.id] = { active = item.active ~= nil, obtained = row.obtained, used = row.used }
  md[#md + 1] = ('| %s | %s | %d | %d | %s |'):format(item.id, item.active and item.active.kind or '', row.obtained, row.used, note)
end
json.write(path.join(output, 'report.json'), summary, true)
-- Balance (E342): heroes whose win rate, KDA or damage per game sits far from the field, and
-- items whose holders win far more or less than half. A flag is a lead to read, not a verdict:
-- a few games per hero carry wide noise, and a bot policy can cause it as well as a value.
local BALANCE = { min_games = 6, win_high = 0.75, win_low = 0.25, ratio_high = 1.5, ratio_low = 0.5 }
summary.balance = { heroes = {}, items = {} }
local heroes_sorted, total_damage, total_kda, counted = {}, 0, 0, 0
local function kda(h) return (h.kills + h.assists) / math.max(1, h.deaths) end
for id, h in pairs(hero_rows) do
  heroes_sorted[#heroes_sorted + 1] = id
  total_damage, total_kda, counted = total_damage + h.damage / h.games, total_kda + kda(h), counted + 1
end
table.sort(heroes_sorted)
local mean_damage, mean_kda = total_damage / math.max(1, counted), total_kda / math.max(1, counted)
local flagged = 0
md[#md + 1] = ''
md[#md + 1] = ('| hero | games | win rate | K/D/A | KDA | damage/game | hero damage/game | note |')
md[#md + 1] = '|---|---|---|---|---|---|---|---|'
for _, id in ipairs(heroes_sorted) do
  local h = hero_rows[id]
  local rate, per_game = h.wins / h.games, h.damage / h.games
  local notes = {}
  if h.games >= BALANCE.min_games then
    if rate >= BALANCE.win_high then notes[#notes + 1] = 'WIN HIGH' elseif rate <= BALANCE.win_low then notes[#notes + 1] = 'WIN LOW' end
    if mean_kda > 0 then
      if kda(h) >= mean_kda * BALANCE.ratio_high then notes[#notes + 1] = 'KDA HIGH' elseif kda(h) <= mean_kda * BALANCE.ratio_low then notes[#notes + 1] = 'KDA LOW' end
    end
    -- No damage recorded at all (E352: a report without the tally) is no reading, not a balance flag.
    if mean_damage > 0 then
      if per_game >= mean_damage * BALANCE.ratio_high then notes[#notes + 1] = 'DAMAGE HIGH' elseif per_game <= mean_damage * BALANCE.ratio_low then notes[#notes + 1] = 'DAMAGE LOW' end
    end
  end
  if #notes > 0 then flagged = flagged + 1 end
  summary.balance.heroes[id] = { games = h.games, wins = h.wins, kills = h.kills, deaths = h.deaths, assists = h.assists,
    damage_per_game = per_game, hero_damage_per_game = h.hero_damage / h.games, notes = notes }
  md[#md + 1] = ('| %s | %d | %.0f%% | %d/%d/%d | %.2f | %.0f | %.0f | %s |'):format(id, h.games, rate * 100,
    h.kills, h.deaths, h.assists, kda(h), per_game, h.hero_damage / h.games, table.concat(notes, ', '))
end
md[#md + 1] = ''
md[#md + 1] = '| item | holders | holder win rate | note |'
md[#md + 1] = '|---|---|---|---|'
for _, item in ipairs(shop_items) do
  local holder = holder_rows[item.id]
  if holder then
    local rate = holder.wins / holder.games
    local note = holder.games >= BALANCE.min_games and (rate >= BALANCE.win_high and 'WIN HIGH' or rate <= BALANCE.win_low and 'WIN LOW') or ''
    if note ~= '' then flagged = flagged + 1 end
    summary.balance.items[item.id] = { holders = holder.games, wins = holder.wins, note = note }
    md[#md + 1] = ('| %s | %d | %.0f%% | %s |'):format(item.id, holder.games, rate * 100, note)
  end
end
json.write(path.join(output, 'report.json'), summary, true)
if summary.ue then
  md[#md + 1] = ''
  md[#md + 1] = ('UE: exit %d, result UI %s, result screenshot %s, time-boxed %s (%s)'):format(summary.ue.exit_code,
    tostring(summary.ue.result_ui), tostring(summary.ue.result_screenshot), tostring(summary.ue.time_boxed), summary.ue.output)
end
path.write(path.join(output, 'report.md'), table.concat(md, '\n') .. '\n', true)
print(table.concat(md, '\n'))
print(('report: %s (%d castable abilities never cast, %d shop items never obtained or never used, %d balance flags)'):format(
  path.join(output, 'report.md'), never, unused_items, flagged))
-- A batch where no match finished checked nothing (E332): fail, naming the first match's error.
local finished = 0
for _, row in ipairs(summary.recipes) do finished = finished + row.finished end
if finished == 0 and #jobs > 0 then
  local err = path.is_file(jobs[1].log .. '.err') and path.read(jobs[1].log .. '.err') or ''
  io.stderr:write('no match finished; first match log: ' .. jobs[1].log .. '\n' .. err:sub(-1500) .. '\n')
  os.exit(1)
end
