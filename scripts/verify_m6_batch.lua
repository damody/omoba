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
    elseif v == '--no-build' then options.no_build = true
    elseif v == '--report-only' then options.report_only = true; options.no_build = true
    elseif v == '--help' then
      print('Usage: tools/lua/lua.exe scripts/verify_m6_batch.lua --batch N [--group G] [--matches 6] [--parallel 6] [--ue] [--output DIR] [--no-build] [--report-only]')
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
local authored = assert(loadfile(path.join(b.root, 'scripts', 'lua_data', 'templates', 'authored_heroes.lua')))()({})
local playable = {}
for _, hero in ipairs(authored.heroes) do playable[hero.id] = hero.abilities end
for id, update in pairs(authored.hero_updates or {}) do playable[id] = update.abilities_by_slot end
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
local output
if options.output then output = path.absolute(options.output, b.root)
else output = path.join(b.root, 'target', 'm6-batch', ('batch%d-group%d-%d'):format(options.batch, options.group, os.time())) end
-- --report-only re-reads the match reports of an earlier run in that directory.
if options.report_only then assert(options.output and path.is_directory(output), '--report-only needs --output of an earlier run')
else assert(not path.exists(output), 'output must be a new directory: ' .. output) end
path.mkdir_p(output)
local function lua_list(list)
  local quoted = {}
  for _, v in ipairs(list) do quoted[#quoted + 1] = ('%q'):format(v) end
  return '{' .. table.concat(quoted, ',') .. '}'
end
local function recipe(name, swap, teamfight)
  local file = path.join(output, name .. '.lua')
  path.write(file, ([[
-- Generated by scripts/verify_m6_batch.lua (batch %d, group %d); recipe %s.
local root = %q
local plan = assert(loadfile(root .. '/scripts/lua_data/moba_role_match.lua'))()
plan.map_id = 'three_lane_quick_training'
local group, swap, teamfight = %s, %s, %s
local seats = {}
for _, team in ipairs(swap and {2, 1} or {1, 2}) do
  for _, player in ipairs(plan.players) do if player.team_id == team then seats[#seats + 1] = player end end
end
for i, hero in ipairs(group) do seats[i].hero = hero end
if teamfight then for _, player in ipairs(plan.players) do player.lane = 'mid' end end
plan.ability_policies = assert(loadfile(root .. '/scripts/lua_data/bot_auto_policies.lua'))()(plan.ability_policies, plan.players)
return plan
]]):format(options.batch, options.group, name, b.root:gsub('\\', '/'), lua_list(group), tostring(swap), tostring(teamfight)), true)
  return file
end
local recipes = {
  { name = 'quick', file = recipe('quick', false, false), group_team = 1 },
  { name = 'quick_swap', file = recipe('quick_swap', true, false), group_team = 2 },
  { name = 'teamfight', file = recipe('teamfight', false, true), group_team = 1 },
}

-- 3. Build once (the script DLL set and moba-headless), like run_moba_headless_batch.lua.
local env = compiled.environment({ OMB_HEADLESS_HERO_STATS = '1' })
-- The script DLLs, moba-headless and their private copies come from one content state (E332):
-- a landing between the two builds gave them different content hashes and every match panicked
-- at load. The repo build lock keeps land_to_live and other builds out until the copies are made.
local build_lock <close> = not options.report_only and require('build_lock').acquire(b.root, 'verify_m6_batch build') or nil
if not options.no_build then
  for _, args in ipairs({
    require('script_dlls').cargo_args({ 'build', '--manifest-path', path.join(b.root, 'scripts', 'Cargo.toml') }, compiled.feature, { '--release' }),
    { 'build', '--manifest-path', path.join(b.root, 'omb', 'Cargo.toml'), '-p', 'omobab', '--bin', 'moba-headless', '--features', compiled.feature, '--release' },
  }) do
    local result = process.run('cargo', args, { cwd = b.root, env = env, check = false })
    assert(result.exit_code == 0, 'build failed: ' .. (result.stderr or ''):sub(-2000))
  end
end
-- Run from a private copy so later builds never lock or change this batch's binaries (E125).
local bin = path.join(output, 'bin')
local exe = path.join(bin, 'moba-headless.exe')
if not options.report_only then
  path.mkdir_p(bin)
  local release = path.join(b.root, 'scripts', 'target', 'release')
  for _, module in ipairs(require('script_dlls').modules()) do
    path.write(path.join(bin, module.file), path.read(path.join(release, module.file), true), false, true)
  end
  path.write(exe, path.read(path.join(b.root, 'omb', 'target', 'release', 'moba-headless.exe'), true), false, true)
end
if build_lock then build_lock:release() end

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
for _, a in ipairs(authored.abilities) do kinds[a.id] = a.ability_type end
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
  local note = row.obtained == 0 and 'NEVER OBTAINED' or (item.active and row.used == 0) and 'NEVER USED' or ''
  if note ~= '' then unused_items = unused_items + 1 end
  summary.items[item.id] = { active = item.active ~= nil, obtained = row.obtained, used = row.used }
  md[#md + 1] = ('| %s | %s | %d | %d | %s |'):format(item.id, item.active and item.active.kind or '', row.obtained, row.used, note)
end
json.write(path.join(output, 'report.json'), summary, true)
if summary.ue then
  md[#md + 1] = ''
  md[#md + 1] = ('UE: exit %d, result UI %s, result screenshot %s, time-boxed %s (%s)'):format(summary.ue.exit_code,
    tostring(summary.ue.result_ui), tostring(summary.ue.result_screenshot), tostring(summary.ue.time_boxed), summary.ue.output)
end
path.write(path.join(output, 'report.md'), table.concat(md, '\n') .. '\n', true)
print(table.concat(md, '\n'))
print(('report: %s (%d castable abilities never cast, %d shop items never obtained or never used)'):format(
  path.join(output, 'report.md'), never, unused_items))
-- A batch where no match finished checked nothing (E332): fail, naming the first match's error.
local finished = 0
for _, row in ipairs(summary.recipes) do finished = finished + row.finished end
if finished == 0 and #jobs > 0 then
  local err = path.is_file(jobs[1].log .. '.err') and path.read(jobs[1].log .. '.err') or ''
  io.stderr:write('no match finished; first match log: ' .. jobs[1].log .. '\n' .. err:sub(-1500) .. '\n')
  os.exit(1)
end
