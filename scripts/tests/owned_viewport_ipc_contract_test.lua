-- Artifact contract for the owned viewport IPC verifier. No process, build, or fixture is started.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local path = b.lib('path')
local ipc = require('moba_owned_viewport_ipc')
local fixture = require('moba_renderer_fixture')

local RUN = 'ipc-contract'
local HASH = 'c41fcc1e80e95ad7'
local GENERATED = 'd5553bbb31c459a4'
local SURFACE = '1e2dbe976a9ee6d7'
local CATALOG_IDENTITY = 'ff3ef5e2957aa89f'
local names = ipc.expected_case_names()
local declared_wires = 0
for _, spec in ipairs(ipc.expected_cases()) do
  if spec[2] then declared_wires = declared_wires + (type(spec[3])=='table' and #spec[3] or spec[3] == 'twice' and 2 or 1) end
end
assert(#names == 458 and declared_wires == 222, 'owned viewport case inventory changed')
assert(names[414]=='combat-idle-projected' and names[415]=='input-point-ready' and names[432]=='input-locked','original 414-case prefix changed')
assert(names[397]=='hud-status-retired-click' and names[398]=='selection-ready' and names[409]=='selection-final-clear','original 397-case prefix changed')
assert(names[294] == 'shop-recipe-buy', 'shop prefix did not stay the 294th case')
assert(names[313] == 'shop-txn-epoch', 'shop transaction prefix did not stay the 313th case')
assert(names[314] == 'shop-txn-full', 'full shop case was not appended')
local ward_names = {
  'ward-ready-status', 'ward-point', 'ward-esc', 'ward-dead', 'ward-quick', 'ward-indicator',
  'ward-normal-release', 'ward-source-retire', 'ward-status-uncertain-then-settled', 'ward-status-rejected',
}
for index, name in ipairs(ward_names) do
  assert(names[294 + index] == name, 'ward case order drifted at ' .. name)
end
local stages = ipc.expected_stages()
assert(#stages == 157, 'control stage inventory changed')
do
  local tail,wires=ipc.expected_cases('tail'),0
  for _,spec in ipairs(tail)do if spec[2]then wires=wires+(type(spec[3])=='table' and #spec[3] or spec[3]=='twice' and 2 or 1)end end
  assert(#tail==164 and wires==102 and #ipc.expected_stages('tail')==54,'owned tail inventory changed')
  assert(tail[1][1]=='ward-ready-status' and tail[#tail][1]=='accessible-restored','tail boundary differs')
  tail[1][1]='mutated';assert(ipc.expected_cases('tail')[1][1]=='ward-ready-status','tail declarations alias callers')
end
assert(stages[103] == 'owner_shop_ready', 'shop prefix did not stay the 103rd stage')
assert(stages[122] == 'controlled_owner_shop_epoch', 'shop transaction prefix did not stay the 122nd stage')
assert(stages[123] == 'controlled_owner_shop_full', 'full shop stage was not appended')
local ward_stages = {
  'controlled_owner_ward_projection',
  'controlled_owner_ward_dead',
  'controlled_owner_ward_projection',
  'controlled_owner_ward_projection',
  'controlled_owner_ward_projection',
  'controlled_owner_ward_projection',
  'controlled_owner_ward_same_epoch_source',
  'controlled_owner_ward_projection',
  'controlled_owner_ward_forwarded',
  'controlled_owner_ward_uncertain_mapped',
  'controlled_owner_ward_settled_mapped',
  'controlled_owner_ward_rejected_mapped',
}
for index, stage in ipairs(ward_stages) do
  assert(stages[103 + index] == stage, 'ward stage order drifted at ' .. stage)
end
local cpp = path.read(path.join(b.root, 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmOwnedViewportIpcTests.cpp'))
assert(cpp:find('#include "OmOwnedAccessibleInput.inl"',1,true),'accessible fixture fragment is not compiled')
cpp = cpp .. path.read(path.join(b.root,'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmOwnedAccessibleInput.inl'))
for _, name in ipairs(names) do
  assert(cpp:find('TEXT("' .. name .. '")', 1, true), 'C++ case inventory is missing ' .. name)
end
local ui_at = assert(cpp:find('bool AttackUi()', 1, true), 'AttackUi is missing')
local armed_at = assert(cpp:find('bool ArmedA()', ui_at, true), 'ArmedA is missing')
local ui = cpp:sub(ui_at, armed_at)
local thumb = ui:find('EKeys::ThumbMouseButton2', 1, true)
local rmb = ui:find('EKeys::RightMouseButton', 1, true)
assert(thumb and rmb and thumb < rmb, 'attack-only UI must send ThumbMouseButton2 before the ordinary right click')
assert(cpp:find('EOmCastMode::QuickWithIndicator', 1, true), 'Q is not set to QuickWithIndicator')
local owner_cpp = path.read(path.join(b.root, 'omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmPlayerActionOwnerComponent.cpp'))
assert(owner_cpp:find('source_death keeps the owned hero', 1, true), 'source death does not retire the physical cast hold')
assert(not owner_cpp:find('SPI_GETKEYBOARDDELAY', 1, true), 'held world cadence still reads the Windows keyboard delay')
assert(not owner_cpp:find('InitialButtonRepeatDelay', 1, true), 'held world cadence still probes a future input-settings field')
assert(not owner_cpp:find('QueryPlatformKeyboardRepeat', 1, true), 'held world cadence still has a platform keyboard fallback')

local PUBLISHED_BASE = 2
local function ack(sequence, stage, source_id, source_epoch, target_id, target_epoch, preceding_frames)
  return {
    schema = fixture.schemas.ack, run_id = RUN, authority_ack = false, accepted = true,
    sequence = sequence, stage = stage, published_sequence = PUBLISHED_BASE + sequence + (preceding_frames or 0),
    source_render_id = source_id, source_disclosure_epoch = source_epoch,
    target_render_id = target_id, target_disclosure_epoch = target_epoch,
  }
end

-- Startup source epoch 3 and target epoch 5 stay unequal through the first source_death.
-- The following baseline refreshes target 21 to floor 3. Removal therefore cannot keep epoch 5.
local history = {
  ack(1, 'source_death', 11, 3, 21, 5),
  ack(2, 'baseline', 11, 3, 21, 3),
  ack(3, 'source_death', 11, 3, 21, 3),
  ack(4, 'baseline', 11, 3, 21, 3),
  ack(5, 'source_replacement', 12, 4, 21, 3),
  ack(6, 'source_removal', json.null, json.null, 21, 3),
  ack(7, 'baseline', 11, 4, 21, 4),
  ack(8, 'disclosure_epoch', 11, 8, 21, 8),
  ack(9, 'replace_target', 11, 8, 22, 9),
  ack(10, 'baseline', 11, 9, 21, 9),
  ack(11, 'owner_cast_ready', 11, 9, 21, 9),
  ack(12, 'owner_upgrade_point', 11, 9, 21, 9),
  ack(13, 'owner_short_mana', 11, 9, 21, 9),
  ack(14, 'owner_cooldown', 11, 9, 21, 9),
  ack(15, 'owner_hide_target', 11, 9, 21, 9),
  ack(16, 'owner_far_unit', 11, 9, 21, 9),
  ack(17, 'owner_apprentice', 11, 9, 21, 9),
  ack(18, 'owner_point_ready', 11, 9, 21, 9),
  ack(19, 'owner_cast_ready', 11, 9, 21, 9),
  ack(20, 'source_death', 11, 9, 21, 9),
  ack(21, 'baseline', 11, 9, 21, 9),
  ack(22, 'owner_cast_ready', 11, 9, 21, 9),
  ack(23, 'source_replacement', 12, 10, 21, 9),
  ack(24, 'baseline', 11, 10, 21, 10),
  ack(25, 'owner_cast_ready', 11, 10, 21, 10),
  ack(26, 'source_removal', json.null, json.null, 21, 10),
  ack(27, 'baseline', 11, 10, 21, 10),
  ack(28, 'owner_cast_ready', 11, 10, 21, 10),
  ack(29, 'disclosure_epoch', 11, 11, 21, 11),
  ack(30, 'death', 11, 11, 21, 11),
  ack(31, 'owner_support_ready', 11, 11, 21, 11),
  ack(32, 'owner_support_enemy', 11, 11, 21, 11),
  ack(33, 'owner_support_conceal', 11, 11, 21, 11),
  ack(34, 'owner_support_dead', 11, 11, 21, 11),
  ack(35, 'owner_support_neutral', 11, 11, 21, 11),
  ack(36, 'owner_point_ready', 11, 11, 21, 11),
  ack(37, 'owner_summoner_ready', 11, 11, 21, 11),
  ack(38, 'owner_summoner_cooldown', 11, 11, 21, 11),
  ack(39, 'owner_summoner_unavailable', 11, 11, 21, 11),
  ack(40, 'owner_summoner_malformed', 11, 11, 21, 11),
  ack(41, 'owner_summoner_unsupported', 11, 11, 21, 11),
  ack(42, 'owner_summoner_ready', 11, 11, 21, 11),
  ack(43, 'source_death', 11, 11, 21, 11),
  ack(44, 'baseline', 11, 11, 21, 11),
  ack(45, 'owner_summoner_ready', 11, 11, 21, 11),
  ack(46, 'source_replacement', 12, 12, 21, 11),
  ack(47, 'baseline', 11, 12, 21, 12),
  ack(48, 'owner_summoner_ready', 11, 12, 21, 12),
  ack(49, 'source_removal', json.null, json.null, 21, 12),
  ack(50, 'baseline', 11, 12, 21, 12),
  ack(51, 'owner_summoner_ready', 11, 12, 21, 12),
  ack(52, 'owner_recall_ready', 11, 12, 21, 12),
  ack(53, 'owner_recall_channel', 11, 12, 21, 12),
  ack(54, 'owner_recall_unavailable', 11, 12, 21, 12),
  ack(55, 'owner_recall_ready', 11, 12, 21, 12),
  ack(56, 'source_death', 11, 12, 21, 12),
  ack(57, 'baseline', 11, 12, 21, 12),
  ack(58, 'owner_item_ready', 11, 12, 21, 12),
  ack(59, 'owner_item_empty', 11, 12, 21, 12),
  ack(60, 'owner_item_cooldown', 11, 12, 21, 12),
  ack(61, 'owner_item_ready', 11, 12, 21, 12),
  ack(62, 'owner_item_far', 11, 12, 21, 12),
  ack(63, 'owner_item_ready', 11, 12, 21, 12),
  ack(64, 'owner_item_replaced', 11, 12, 21, 12),
  ack(65, 'owner_item_ready', 11, 12, 21, 12),
  ack(66, 'source_death', 11, 12, 21, 12),
  ack(67, 'baseline', 11, 12, 21, 12),
  ack(68, 'owner_item_ready', 11, 12, 21, 12),
  ack(69, 'source_death', 11, 12, 21, 12),
  ack(70, 'baseline', 11, 12, 21, 12),
  ack(71, 'owner_item_ready', 11, 12, 21, 12),
  ack(72, 'source_replacement', 12, 13, 21, 12),
  ack(73, 'baseline', 11, 13, 21, 13),
  ack(74, 'owner_item_ready', 11, 13, 21, 13),
  ack(75, 'source_removal', json.null, json.null, 21, 13),
  ack(76, 'baseline', 11, 13, 21, 13),
  ack(77, 'source_death', 11, 13, 21, 13),
  ack(78, 'baseline', 11, 13, 21, 13),
  ack(79, 'owner_recall_channel', 11, 13, 21, 13),
  ack(80, 'source_death', 11, 13, 21, 13),
  ack(81, 'owner_item_ready', 11, 13, 21, 13),
  ack(82, 'owner_item_empty', 11, 13, 21, 13),
  ack(83, 'owner_item_cooldown', 11, 13, 21, 13),
  ack(84, 'source_death', 11, 13, 21, 13),
  ack(85, 'baseline', 11, 13, 21, 13),
  ack(86, 'owner_item_ready', 11, 13, 21, 13),
  ack(87, 'source_death', 11, 13, 21, 13),
  ack(88, 'baseline', 11, 13, 21, 13),
  ack(89, 'owner_item_ready', 11, 13, 21, 13),
  ack(90, 'source_replacement', 12, 14, 21, 13),
  ack(91, 'baseline', 11, 14, 21, 14),
  ack(92, 'owner_item_ready', 11, 14, 21, 14),
  ack(93, 'source_removal', json.null, json.null, 21, 14),
  ack(94, 'baseline', 11, 14, 21, 14),
  ack(95, 'mode_tower_defense', 11, 14, 21, 14),
  ack(96, 'mode_td_defeat', 11, 14, 21, 14),
  ack(97, 'mode_td_no_hero', 11, 14, 21, 14),
  ack(98, 'mode_moba_dead', 11, 14, 21, 14),
  ack(99, 'mode_unavailable', 11, 14, 21, 14),
  ack(100, 'mode_tower_defense', 11, 14, 21, 14),
  ack(101, 'mode_td_epoch', 11, 15, 21, 15),
  ack(102, 'baseline', 11, 15, 21, 15),
  ack(103, 'owner_shop_ready', 11, 15, 21, 15),
  ack(104, 'controlled_owner_ward_projection', 11, 15, 21, 15),
  ack(105, 'controlled_owner_ward_dead', 11, 15, 21, 15),
  ack(106, 'controlled_owner_ward_projection', 11, 15, 21, 15),
  ack(107, 'controlled_owner_ward_projection', 11, 15, 21, 15),
  ack(108, 'controlled_owner_ward_projection', 11, 15, 21, 15),
  ack(109, 'controlled_owner_ward_projection', 11, 15, 21, 15),
  ack(110, 'controlled_owner_ward_same_epoch_source', 12, 79, 21, 15),
  ack(111, 'controlled_owner_ward_projection', 12, 79, 21, 15),
  ack(112, 'controlled_owner_ward_forwarded', 12, 79, 21, 15),
  ack(113, 'controlled_owner_ward_uncertain_mapped', 12, 79, 21, 15),
  ack(114, 'controlled_owner_ward_settled_mapped', 12, 79, 21, 15),
  ack(115, 'controlled_owner_ward_rejected_mapped', 12, 79, 21, 15),
  ack(116, 'controlled_owner_shop_current', 12, 79, 21, 15),
  ack(117, 'controlled_owner_shop_admitted', 12, 79, 21, 15),
  ack(118, 'controlled_owner_shop_uncertain', 12, 79, 21, 15),
  ack(119, 'controlled_owner_shop_settled', 12, 79, 21, 15),
  ack(120, 'controlled_owner_shop_rejected', 12, 79, 21, 15),
  ack(121, 'controlled_owner_shop_stale', 12, 79, 21, 15),
  ack(122, 'controlled_owner_shop_epoch', 12, 16, 21, 16),
  ack(123, 'controlled_owner_shop_full', 12, 16, 21, 16),
  ack(124, 'communication_ready', 11, 16, 21, 16),
  ack(125, 'communication_retire', 11, 17, 21, 17, 1),
  ack(126, 'communication_received', 11, 17, 21, 17, 2),
  ack(127, 'communication_retire', 11, 18, 21, 18, 3),
  ack(128, 'communication_ready', 11, 18, 21, 18, 4),
  ack(129, 'communication_received', 11, 18, 21, 18, 5),
  ack(130, 'communication_retire', 11, 19, 21, 19, 6),
  ack(131, 'communication_ready', 11, 19, 21, 19, 7),
  ack(132, 'communication_retire', 11, 20, 21, 20, 8),
  ack(133, 'communication_ready', 11, 20, 21, 20, 9),
  ack(134, 'communication_signals', 11, 20, 21, 20, 10),
  ack(135, 'communication_signals_expired', 11, 20, 21, 20, 11),
  ack(136, 'communication_retire', 11, 21, 21, 21, 12),
  ack(137, 'communication_hud', 11, 21, 21, 21, 13),
  ack(138, 'communication_retire', 11, 22, 21, 22, 14),
  ack(139, 'baseline', 11, 22, 21, 22, 15),
  ack(140, 'combat_defaults', 11, 22, 21, 22, 15),
  ack(141, 'combat_cursor_projection', 11, 22, 21, 22, 15),
  ack(142, 'combat_idle_off_projection', 11, 22, 21, 22, 15),
  ack(143, 'input_point', 11, 22, 21, 22, 15),
  ack(144, 'input_keyboard', 11, 22, 21, 22, 15),
  ack(145, 'input_point', 11, 22, 21, 22, 15),
  ack(146, 'input_locked', 11, 22, 21, 22, 15),
  ack(147, 'owner_point_ready', 11, 22, 21, 22, 15),
  ack(148, 'owner_upgrade_point', 11, 22, 21, 22, 15),
  ack(149, 'owner_cast_ready', 11, 22, 21, 22, 15),
  ack(150, 'owner_support_team_cards', 11, 22, 21, 22, 15),
  ack(151, 'owner_support_team_cards_dead', 11, 22, 21, 22, 15),
  ack(152, 'baseline', 11, 22, 21, 22, 15),
  ack(153, 'input_point', 11, 22, 21, 22, 15),
  ack(154, 'owner_point_ready', 11, 22, 21, 22, 15),
  ack(155, 'disclosure_epoch', 11, 23, 21, 23, 15),
  ack(156, 'input_keyboard', 11, 23, 21, 23, 15),
  ack(157, 'input_point', 11, 23, 21, 23, 15),
}

local function wire(ordinal, kind, view, target_id, target_epoch, ability, published)
  local intent = {kind = kind}
  if kind == 'InputScheme' then
    intent.schema_version=1;intent.scheme=tonumber(ability)
  elseif kind == 'Direction' then
    intent.schema_version=1;intent.source_render_id='11';intent.phase=ability[1]
    intent.x=ability[2];intent.y=ability[3];intent.press_id=ability[4]
  elseif kind == 'CombatPreferences' then
    intent.schema_version=1;intent.acquisition=1;intent.idle_auto_attack=ability.idle
  elseif kind == 'AbilityCast' or kind == 'AbilityUpgrade' then
    assert(type(ability) == 'table', 'synthetic ability wire is missing its observation')
    intent.ability_index = ability.index
    intent.catalog_id = ability.catalog
    intent.ability_id = ability.id
    if kind == 'AbilityCast' then
      intent.target_kind = ability.shape
      local cast_target = 0
      if ability.shape == 'unit' then cast_target = ability.unit == 'source' and 11 or target_id end
      intent.target_render_id = cast_target
      intent.x_raw = ability.shape == 'point' and 4096 or 0
      intent.y_raw = ability.shape == 'point' and 8192 or 0
    end
  elseif kind == 'SummonerCast' then
    assert(type(ability) == 'table', 'synthetic summoner wire is missing its observation')
    intent.slot = ability.slot
    intent.catalog_id = ability.catalog
    intent.spell_id = ability.id
    intent.target_kind = ability.shape
    if ability.shape == 'point' then
      intent.x_raw = 4096
      intent.y_raw = 8192
    else
      intent.x_raw = json.null
      intent.y_raw = json.null
    end
  elseif kind == 'Stop' or kind == 'Recall' then
    -- Cursor-free commands. Stop omits queued; Recall omits a point.
  elseif kind == 'HoldPosition' then
    intent.queued = false
  elseif kind == 'ItemUse' then
    assert(type(ability) == 'table', 'synthetic item wire is missing its observation')
    intent.item_slot = ability.slot
    intent.catalog_id = ability.catalog
    intent.item_id = ability.id
    intent.target_render_id = ability.policy == 'unit' and target_id or 0
    intent.x_raw = ability.policy == 'point' and 4096 or 0
    intent.y_raw = ability.policy == 'point' and 8192 or 0
    if ability.policy == 'point' then
      intent.explicit_x_raw = intent.x_raw
      intent.explicit_y_raw = intent.y_raw
    else
      intent.explicit_x_raw = json.null
      intent.explicit_y_raw = json.null
    end
  elseif kind == 'ItemBuy' then
    assert(type(ability) == 'table', 'synthetic item buy wire is missing its catalog identity')
    intent.catalog_id = ability.catalog
  elseif kind == 'ItemSell' then
    assert(type(ability) == 'table', 'synthetic item sell wire is missing its slot')
    intent.item_slot = ability.slot
  elseif kind == 'Communication' then
    assert(type(ability) == 'table')
    intent.communication_kind = ability.hud and 6 or ability.objective and 3 or ability.semantic and 1 or ability.channel and 2 or ability.mute ~= nil and 4 or 5
    intent.chat_channel = ability.channel or 0
    intent.content_id = ability.content or 0
    intent.source_generation, intent.presentation_epoch = 1, 1
    intent.text = ability.channel and (ability.text or '集合') or ''
    intent.has_point, intent.muted = false, false
    for _, field in ipairs({'semantic','point_x_raw','point_y_raw','target_render_id','target_disclosure_epoch','hud_kind','hud_index','hud_player_id','mute_sender_id'}) do intent[field] = 0 end
    intent.objective_id = ability.objective or ''
    if ability.hud then intent.hud_kind=ability.hud;intent.hud_index=ability.index;intent.hud_player_id=ability.player end
    if ability.semantic then intent.has_point=true; intent.semantic=ability.semantic; intent.point_x_raw=1024; intent.point_y_raw=2048 end
    if ability.mute ~= nil then intent.mute_sender_id = 2; intent.muted = ability.mute end
  elseif kind == 'WardPlacement' then
    assert(type(ability) == 'table', 'synthetic ward wire is missing its source identity')
    intent.slot = 0
    intent.x_raw = 178
    intent.y_raw = 9062
  else
    intent.queued = false
    if kind == 'AttackTarget' then intent.target_render_id = target_id
    else intent.x_raw = 1024; intent.y_raw = 2048 end
  end
  local line = {
    schema = 'omoba-renderer-fixture-intent/1', run_id = RUN, observation_ordinal = ordinal,
    authority_ack = false, player_id = 1, content_hash = HASH, moba_map_id = 'three_lane_training',
    disclosure_epoch = view, entity_epoch_floor = view, published_sequence = published,
    target_render_id = target_id, target_disclosure_epoch = target_epoch,
    intent = intent, ipc_sequence = published + (kind == 'Communication' and 1 or 0),
  }
  if kind == 'WardPlacement' then
    line.source_render_id = ability.source
    line.source_disclosure_epoch = ability.generation
  end
  return line
end

-- Intents before the first baseline still carry target epoch 5. After that baseline the floor is 3.
local expected_wires = {
  {'MoveTo', 3, 21, 5},
  {'AttackTarget', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'AttackMove', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 5},
  {'MoveTo', 3, 21, 3},
  {'MoveTo', 4, 21, 3},
  {'AttackTarget', 8, 21, 8},
  {'AttackTarget', 9, 22, 9},
  {'AttackMove', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'MoveTo', 9, 21, 9},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_touch', catalog = 10, index = 1, shape = 'none'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_touch', catalog = 10, index = 1, shape = 'none'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_touch', catalog = 10, index = 1, shape = 'none'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_lance', catalog = 11, index = 2, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_lance', catalog = 11, index = 2, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_lance', catalog = 11, index = 2, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_mend', catalog = 12, index = 3, shape = 'none'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_mend', catalog = 12, index = 3, shape = 'none'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_mend', catalog = 12, index = 3, shape = 'none'}},
  {'MoveTo', 9, 21, 9},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0, shape = 'unit'}},
  {'AbilityCast', 9, 21, 9, {id = 'lumen_touch', catalog = 10, index = 1, shape = 'none'}},
  {'AbilityUpgrade', 9, 21, 9, {id = 'lumen_bolt', catalog = 9, index = 0}},
  {'AbilityUpgrade', 9, 21, 9, {id = 'apprentice_bolt', catalog = 13, index = 0}},
  {'AbilityCast', 9, 21, 9, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'AbilityCast', 9, 21, 9, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'AbilityCast', 9, 21, 9, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'AbilityCast', 11, 21, 11, {id = 'lumen_aid', catalog = 25, index = 3, shape = 'unit'}},
  {'AbilityCast', 11, 21, 11, {id = 'lumen_aid', catalog = 25, index = 3, shape = 'unit', unit = 'source'}},
  {'MoveTo', 11, 21, 11},
  {'AbilityCast', 11, 21, 11, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'AbilityCast', 11, 21, 11, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'AbilityCast', 11, 21, 11, {id = 'vanguard_resolve', catalog = 20, index = 3, shape = 'point'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_point_blink_v1', catalog = 1, slot = 0, shape = 'point'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_point_blink_v1', catalog = 1, slot = 0, shape = 'point'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_point_blink_v1', catalog = 1, slot = 0, shape = 'point'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_self_heal_v1', catalog = 2, slot = 1, shape = 'none'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_self_heal_v1', catalog = 2, slot = 1, shape = 'none'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_self_heal_v1', catalog = 2, slot = 1, shape = 'none'}},
  {'MoveTo', 11, 21, 11},
  {'SummonerCast', 11, 21, 11, {id = 'training_self_heal_v1', catalog = 2, slot = 1, shape = 'none'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_point_blink_v1', catalog = 1, slot = 0, shape = 'point'}},
  {'SummonerCast', 11, 21, 11, {id = 'training_self_heal_v1', catalog = 2, slot = 1, shape = 'none'}},
  {'SummonerCast', 12, 21, 11, {id = 'training_point_blink_v1', catalog = 1, slot = 0, shape = 'point'}},
  {'Stop', 12, 21, 12},
  {'Stop', 12, 21, 12},
  {'HoldPosition', 12, 21, 12},
  {'HoldPosition', 12, 21, 12},
  {'Stop', 12, 21, 12},
  {'Stop', 12, 21, 12},
  {'HoldPosition', 12, 21, 12},
  {'Recall', 12, 21, 12},
  {'Recall', 12, 21, 12},
  {'ItemUse', 12, 21, 12, {slot = 0, policy = 'self', catalog = 5, id = 'moba_guard_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 0, policy = 'self', catalog = 5, id = 'moba_guard_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 0, policy = 'self', catalog = 5, id = 'moba_guard_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 1, policy = 'point', catalog = 10, id = 'moba_training_flare'}},
  {'ItemUse', 12, 21, 12, {slot = 1, policy = 'point', catalog = 10, id = 'moba_training_flare'}},
  {'ItemUse', 12, 21, 12, {slot = 1, policy = 'point', catalog = 10, id = 'moba_training_flare'}},
  {'ItemUse', 12, 21, 12, {slot = 2, policy = 'unit', catalog = 11, id = 'moba_training_hex'}},
  {'ItemUse', 12, 21, 12, {slot = 2, policy = 'unit', catalog = 11, id = 'moba_training_hex'}},
  {'ItemUse', 12, 21, 12, {slot = 2, policy = 'unit', catalog = 11, id = 'moba_training_hex'}},
  {'ItemUse', 12, 21, 12, {slot = 0, policy = 'self', catalog = 5, id = 'moba_guard_charm'}},
  {'MoveTo', 12, 21, 12},
  {'ItemUse', 12, 21, 12, {slot = 1, policy = 'self', catalog = 6, id = 'moba_stride_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 1, policy = 'point', catalog = 10, id = 'moba_training_flare'}},
  {'ItemUse', 12, 21, 12, {slot = 4, policy = 'self', catalog = 6, id = 'moba_stride_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 4, policy = 'self', catalog = 6, id = 'moba_stride_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 4, policy = 'self', catalog = 6, id = 'moba_stride_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 4, policy = 'self', catalog = 6, id = 'moba_stride_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 5, policy = 'self', catalog = 7, id = 'moba_mana_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 5, policy = 'self', catalog = 7, id = 'moba_mana_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 5, policy = 'self', catalog = 7, id = 'moba_mana_charm'}},
  {'ItemUse', 12, 21, 12, {slot = 5, policy = 'self', catalog = 7, id = 'moba_mana_charm'}},
  {'Stop', 12, 21, 12},
  {'HoldPosition', 12, 21, 12},
  {'Stop', 12, 21, 12},
  {'HoldPosition', 12, 21, 12},
  {'Stop', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'HoldPosition', 13, 21, 13},
  {'HoldPosition', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'HoldPosition', 13, 21, 13},
  {'HoldPosition', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'Stop', 13, 21, 13},
  {'ItemUse', 13, 21, 13, {slot = 0, policy = 'self', catalog = 5, id = 'moba_guard_charm'}},
  {'ItemUse', 13, 21, 13, {slot = 1, policy = 'point', catalog = 10, id = 'moba_training_flare'}},
  {'ItemUse', 13, 21, 13, {slot = 2, policy = 'unit', catalog = 11, id = 'moba_training_hex'}},
  {'MoveTo', 13, 21, 13},
  {'ItemBuy', 15, 21, 15, {catalog = 4, id = 'moba_greatsword'}},
  {'WardPlacement', 15, 21, 15, {source = 11, generation = 15}},
  {'WardPlacement', 15, 21, 15, {source = 11, generation = 15}},
  {'WardPlacement', 15, 21, 15, {source = 11, generation = 15}},
  {'WardPlacement', 15, 21, 15, {source = 11, generation = 15}},
  {'WardPlacement', 15, 21, 15, {source = 12, generation = 79}},
  {'ItemBuy', 15, 21, 15, {catalog = 4, id = 'moba_greatsword'}},
  {'ItemSell', 15, 21, 15, {slot = 0}},
  {'ItemBuy', 15, 21, 15, {catalog = 4, id = 'moba_greatsword'}},
}

-- Preserve the original 128 wires verbatim as the ordered prefix.
expected_wires[#expected_wires+1] = {'Communication',16,21,16,{channel=1}}
expected_wires[#expected_wires+1] = {'Communication',16,21,16,{channel=2}}
for content=1,5 do expected_wires[#expected_wires+1] = {'Communication',16,21,16,{content=content}} end
expected_wires[#expected_wires+1] = {'Communication',17,21,17,{mute=true}}
expected_wires[#expected_wires+1] = {'Communication',18,21,18,{mute=false}}
expected_wires[#expected_wires+1] = {'Communication',18,21,18,{channel=1}}
expected_wires[#expected_wires+1] = {'Communication',18,21,18,{channel=2}}
expected_wires[#expected_wires+1] = {'Communication',18,21,18,{channel=1,text='QWER A 集合'}}for _, policy in ipairs({{semantic=9},{semantic=10},{semantic=1},{content=101}}) do
  expected_wires[#expected_wires+1] = {'Communication',19,21,19,policy}
end
for _=1,4 do expected_wires[#expected_wires+1] = {'Communication',20,21,20,{semantic=9}} end
local additional_wheel_policies = {{semantic=1},{content=101},{semantic=1},{content=101}}
for sector=2,8 do additional_wheel_policies[#additional_wheel_policies+1] = {semantic=sector} end
for sector=2,8 do additional_wheel_policies[#additional_wheel_policies+1] = {content=100+sector} end
additional_wheel_policies[#additional_wheel_policies+1] = {semantic=9}
for _, policy in ipairs(additional_wheel_policies) do expected_wires[#expected_wires+1] = {'Communication',20,21,20,policy} end
local shortcut_policies = {}
for index=0,3 do shortcut_policies[#shortcut_policies+1]={hud=1,index=index,player=0} end
for index=0,2 do shortcut_policies[#shortcut_policies+1]={hud=2,index=index,player=0} end
shortcut_policies[#shortcut_policies+1]={hud=3,index=0,player=2}
shortcut_policies[#shortcut_policies+1]={objective='north_guardian'}
shortcut_policies[#shortcut_policies+1]={objective='south_guardian'}
for _,policy in ipairs(shortcut_policies) do expected_wires[#expected_wires+1]={'Communication',21,21,21,policy} end
expected_wires[#expected_wires+1]={'AttackTarget',22,21,22}
expected_wires[#expected_wires+1]={'AttackMove',22,21,22}
expected_wires[#expected_wires+1]={'CombatPreferences',22,21,22,{idle=true}}
expected_wires[#expected_wires+1]={'CombatPreferences',22,21,22,{idle=false}}
for index,spec in ipairs(ipc.expected_cases()) do
  if index>414 and spec[2] then
    if type(spec[3])=='table' then
      for _,policy in ipairs(spec[3])do local epoch=index>454 and 23 or 22; expected_wires[#expected_wires+1]={spec[2],epoch,21,epoch,policy} end
    elseif spec[2]=='AbilityCast' or spec[2]=='AbilityUpgrade' then
      local slot,shape=spec[3]:match('^card:([0-3]):?(%a*)$');slot=tonumber(slot)
      expected_wires[#expected_wires+1]={spec[2],22,21,22,{index=slot,catalog=1,id=slot==3 and 'lumen_mend' or 'lumen_bolt',shape=shape}}
    else expected_wires[#expected_wires+1]={spec[2],22,21,22,spec[3]} end
  end
end

local function case_row(name, first, last, repeats)
  if first then
    local raw = {}
    for _ = 1, repeats do raw[#raw + 1] = 'raw-line' end
    return {name = name, first_ordinal = first, last_ordinal = last, intents = raw}
  end
  return {name = name, first_ordinal = 0, last_ordinal = 0, intents = {}}
end

local cases, intents = {}, {}
local wire_index = 0
local published = PUBLISHED_BASE
local observed_stage = 0
local function consume_stages(list)
  for _, stage in ipairs(list) do
    observed_stage = observed_stage + 1
    assert(history[observed_stage].stage == stage, 'synthetic stage order changed')
    published = history[observed_stage].published_sequence
  end
end
for _, spec in ipairs(ipc.expected_cases()) do
  local before, after = ipc.stages_around(spec[1])
  consume_stages(before)
  if spec[2] then
    local repeats = type(spec[3])=='table' and #spec[3] or spec[3] == 'twice' and 2 or 1
    local first = wire_index + 1
    local raw = {}
    for _ = 1, repeats do
      wire_index = wire_index + 1
      local expected = expected_wires[wire_index]
      assert(expected and expected[1] == spec[2], 'synthetic wire kind drifted for ' .. spec[1])
      intents[wire_index] = wire(wire_index, expected[1], expected[2], expected[3], expected[4], expected[5], published)
      raw[#raw + 1] = json.encode(intents[wire_index])
    end
    local row = case_row(spec[1], first, wire_index, repeats)
    row.intents = raw
    cases[#cases + 1] = row
  else
    cases[#cases + 1] = case_row(spec[1])
  end
  consume_stages(after)
end
assert(wire_index == #expected_wires and wire_index == declared_wires, 'synthetic wire inventory drifted')
assert(#history == 157 and observed_stage == #history and published == PUBLISHED_BASE + #history + 15, 'control stage inventory changed')

local startup = {
  schema = fixture.schemas.startup, run_id = RUN, authority = false, critical_input_result = 'never',
  moba_map_id = 'three_lane_training', address = '127.0.0.1:43123',
  source_render_id = 11, target_render_id = 21,
  source_disclosure_epoch = 3, target_disclosure_epoch = 5,
  content_hash = HASH, content_catalog_identity_hash = CATALOG_IDENTITY, moba_map_catalog_hash = 'map-live',
  published_sequence = PUBLISHED_BASE,
}
local native = {
  schema = 'omoba-owned-viewport-ipc-native/1', run_id = RUN, success = true, authority_ack = false, failure = '',
  content_hash = HASH, catalog_data_hash = HASH, content_catalog_identity_hash = startup.content_catalog_identity_hash,
  moba_map_id = startup.moba_map_id, moba_map_catalog_hash = startup.moba_map_catalog_hash,
  generated_content_hash = GENERATED, compiled_content_hash = GENERATED,
  class_surface_signature = SURFACE, compiled_class_surface = SURFACE,
  lockstep_step_fps = 60, catalog_compatible = true, configured_player_id = 1,
  address = startup.address, source_render_id = 11, target_render_id = 21,
  source_disclosure_epoch = 3, target_disclosure_epoch = 5, cases = cases,
}
local ue = {
  succeeded = 1, succeededWithWarnings = 0, failed = 0, notRun = 0, inProcess = 0,
  tests = {{fullTestPath = 'Om.Runtime.OwnedViewportIpc', state = 'Success', warnings = 0, errors = 0}},
}
local bundle = {
  startup = startup, native = native, intents = intents, history = history, ack = history[#history],
  ue_index = json.encode(ue),
  cleanup = {schema = fixture.schemas.cleanup, run_id = RUN, authority = false, listener_released = true},
}

assert(native.generated_content_hash ~= native.catalog_data_hash, 'the synthetic fixture forced the template hash to equal catalog data')
assert(ipc.verify_artifacts(bundle) == true)
local function parsed_policy(text, slot, policy, catalog, item)
  local got_slot, got_policy, got_catalog, got_item = ipc.parse_item_policy(text)
  assert(got_slot == slot and got_policy == policy and got_catalog == catalog and got_item == item, 'parse_item_policy drifted for ' .. text)
end
parsed_policy('0:self:5:moba_guard_charm', '0', 'self', '5', 'moba_guard_charm')
parsed_policy('1:point:10:moba_training_flare', '1', 'point', '10', 'moba_training_flare')
parsed_policy('2:unit:11:moba_training_hex', '2', 'unit', '11', 'moba_training_hex')
local function rejected_policy(text, message)
  local got_slot, got_message = ipc.parse_item_policy(text)
  assert(got_slot == nil and got_message == message, text .. ' -> ' .. tostring(got_slot) .. ' / ' .. tostring(got_message))
end
rejected_policy('0:ally:5:moba_guard_charm', 'ItemUse case policy is unknown')
rejected_policy('0:self|point:5:x', 'ItemUse case policy is missing')
rejected_policy('1:self:5', 'ItemUse case policy is missing')
rejected_policy('0:Self:5:moba_guard_charm', 'ItemUse case policy is unknown')
local buy_catalog, buy_id = ipc.parse_shop_buy_policy('4:moba_greatsword')
assert(buy_catalog == '4' and buy_id == 'moba_greatsword', 'ItemBuy policy did not keep the authored identity')
local buy_missing, buy_message = ipc.parse_shop_buy_policy('4')
assert(buy_missing == nil and buy_message == 'ItemBuy case policy is missing', tostring(buy_message))
local count = 8
local function copy()
  return json.decode(json.encode(bundle))
end
local function reject(label, mutate, needle)
  local sample = copy()
  mutate(sample)
  local ok, err = pcall(ipc.verify_artifacts, sample)
  assert(not ok and tostring(err):find(needle, 1, true), label .. ': ' .. tostring(err))
  count = count + 1
end
reject('AbilityCast is not a successful wire line', function(sample)
  sample.intents[2].intent.kind = 'AbilityCast'
end, 'unexpected wire intent kind')
reject('ability cast must omit queued', function(sample)
  sample.intents[31].intent.queued = false
end, 'carried queued')
reject('friendly unit cast is not a self cast', function(sample)
  local found = false
  for _, line in ipairs(sample.intents) do
    if line.intent and line.intent.ability_id == 'lumen_aid' and line.intent.target_render_id == 21 then
      line.intent.target_render_id = 11
      found = true
      break
    end
  end
  assert(found)
end, 'did not use the disclosed target')
reject('self unit cast is not the envelope target', function(sample)
  local found = false
  for _, line in ipairs(sample.intents) do
    if line.intent and line.intent.ability_id == 'lumen_aid' and line.intent.target_render_id == 11 then
      line.intent.target_render_id = line.target_render_id
      found = true
      break
    end
  end
  assert(found)
end, 'did not use the disclosed source')
reject('intent content hash must echo startup', function(sample)
  sample.intents[1].content_hash = 'other-hash'
end, 'intent content hash mismatch')
reject('native hash must echo the same startup', function(sample)
  sample.native.content_hash = 'other-hash'
end, 'does not echo startup')
reject('missing case', function(sample)
  table.remove(sample.native.cases)
end, 'case inventory mismatch')
reject('extra intent line', function(sample)
  sample.intents[#sample.intents + 1] = {schema = 'extra'}
end, 'outside the expected case sequence')
reject('ItemBuy must not carry queued', function(sample)
  local found = false
  for _, line in ipairs(sample.intents) do
    if line.intent and line.intent.kind == 'ItemBuy' then
      line.intent.queued = false
      found = true
    end
  end
  assert(found, 'synthetic ItemBuy wire is missing')
end, 'ItemBuy carried queued')
reject('ItemBuy must not carry a point', function(sample)
  local found = false
  for _, line in ipairs(sample.intents) do
    if line.intent and line.intent.kind == 'ItemBuy' then
      line.intent.x_raw = 0
      line.intent.y_raw = 0
      found = true
    end
  end
  assert(found, 'synthetic ItemBuy wire is missing')
end, 'ItemBuy carried a point')
reject('listener still held', function(sample)
  sample.cleanup.listener_released = false
end, 'listener was not released')
reject('authority ack on a wire line', function(sample)
  sample.intents[1].authority_ack = true
end, 'claimed authority')
reject('compiled template hash mismatch', function(sample)
  sample.native.compiled_content_hash = 'eeeeeeeeeeeeeeee'
end, 'compiled template hash')
reject('catalog data hash is a different domain from the template hash', function(sample)
  sample.native.catalog_data_hash = sample.native.generated_content_hash
end, 'catalog data hash does not match startup')
reject('class surface mismatch', function(sample)
  sample.native.compiled_class_surface = 'other-surface'
end, 'class surface')
reject('unadmitted step rate', function(sample)
  sample.native.lockstep_step_fps = 0
end, 'lockstep step fps')
local function find_stage(sample, stage)
  for index, ack in ipairs(sample.history) do
    if ack.stage == stage then return ack, index end
  end
  error('missing stage ' .. stage)
end
local function case_named(sample, name)
  for _, row in ipairs(sample.native.cases) do
    if row.name == name then return row end
  end
  error('missing case ' .. name)
end
local function last_ward(sample)
  local line
  for _, candidate in ipairs(sample.intents) do
    if candidate.intent and candidate.intent.kind == 'WardPlacement' then line = candidate end
  end
  assert(line, 'synthetic WardPlacement wire is missing')
  return line
end
reject('missing ward stage', function(sample)
  table.remove(sample.history)
end, 'stage count mismatch')
reject('extra ward stage', function(sample)
  sample.history[#sample.history + 1] = sample.history[#sample.history]
end, 'stage count mismatch')
reject('reordered ward stage', function(sample)
  local _, forwarded = find_stage(sample, 'controlled_owner_ward_forwarded')
  local _, uncertain = find_stage(sample, 'controlled_owner_ward_uncertain_mapped')
  sample.history[forwarded].stage, sample.history[uncertain].stage = sample.history[uncertain].stage, sample.history[forwarded].stage
end, 'stage order mismatch')
reject('ward control ack claims authority', function(sample)
  find_stage(sample, 'controlled_owner_ward_same_epoch_source').authority_ack = true
end, 'claimed authority')
reject('same-view replacement uses the view floor as the actor generation', function(sample)
  local ack = find_stage(sample, 'controlled_owner_ward_same_epoch_source')
  ack.source_disclosure_epoch = ack.target_disclosure_epoch
end, 'used the view floor as the actor generation')
reject('same-view replacement actor generation is not floor plus 64', function(sample)
  local ack = find_stage(sample, 'controlled_owner_ward_same_epoch_source')
  ack.source_disclosure_epoch = ack.target_disclosure_epoch + 63
end, 'actor generation mismatch')
reject('same-view replacement publishes the wrong source', function(sample)
  find_stage(sample, 'controlled_owner_ward_same_epoch_source').source_render_id = 13
end, 'did not publish hero 12')
reject('same-view replacement changes the target', function(sample)
  find_stage(sample, 'controlled_owner_ward_same_epoch_source').target_render_id = 22
end, 'changed the target')
reject('same-view replacement changes the target epoch', function(sample)
  local ack = find_stage(sample, 'controlled_owner_ward_same_epoch_source')
  ack.target_disclosure_epoch = ack.source_disclosure_epoch
end, 'changed the target epoch')
reject('later ward projection drops the replacement source', function(sample)
  local _, index = find_stage(sample, 'controlled_owner_ward_same_epoch_source')
  sample.history[index + 1].source_render_id = 11
end, 'changed the current source')
reject('WardPlacement slot is not 0', function(sample)
  last_ward(sample).intent.slot = 1
end, 'slot must be 0')
reject('WardPlacement carries queued', function(sample)
  last_ward(sample).intent.queued = false
end, 'WardPlacement carried queued')
reject('WardPlacement aliases an ability slot', function(sample)
  last_ward(sample).intent.ability_index = 0
end, 'WardPlacement aliased ability_index')
reject('WardPlacement point is null', function(sample)
  last_ward(sample).intent.x_raw = json.null
end, 'WardPlacement field is malformed')
reject('WardPlacement point is not an integer', function(sample)
  last_ward(sample).intent.y_raw = 1.5
end, 'WardPlacement field is malformed')
reject('WardPlacement point is outside fixed32', function(sample)
  last_ward(sample).intent.x_raw = 2147483648
end, 'outside fixed32')
reject('WardPlacement point is missing', function(sample)
  last_ward(sample).intent.y_raw = nil
end, 'missing slot or point')
reject('missing final wire intent', function(sample)
  table.remove(sample.intents)
end, 'missing wire intent for ' .. (function()
  local declared = ipc.expected_cases()
  for index = #declared, 1, -1 do if declared[index][2] then return declared[index][1] end end
  error('no wire-bearing case in declared inventory')
end)())
reject('extra final wire intent', function(sample)
  sample.intents[#sample.intents + 1] = sample.intents[#sample.intents]
end, 'outside the expected case sequence')
reject('native ward case order', function(sample)
  case_named(sample, 'ward-point').name = 'ward-esc'
end, 'native case order mismatch')
reject('recorded ward raw intent does not match the wire', function(sample)
  case_named(sample, 'ward-point').intents[1] = '{"schema":"omoba-renderer-fixture-intent/1"}'
end, 'recorded raw intent does not match the wire line')
reject('ward wire uses the actor generation as the view', function(sample)
  last_ward(sample).disclosure_epoch = 79
end, 'wire intent view epoch mismatch')
reject('ward wire view floor follows the actor generation', function(sample)
  last_ward(sample).entity_epoch_floor = 79
end, 'confused the view floor')
local function communication(sample, channel)
  for _, line in ipairs(sample.intents) do
    if line.intent.kind == 'Communication' and line.intent.chat_channel == channel then return line end
  end
  error('communication fixture missing')
end
reject('control ordinal cannot stand in for communication IPC sequence', function(sample)
  sample.history[125].published_sequence = PUBLISHED_BASE + 125
end, 'does not match actual IPC envelopes')
reject('communication stage cannot hide an extra IPC envelope', function(sample)
  sample.history[124].published_sequence = sample.history[124].published_sequence + 1
end, 'does not match actual IPC envelopes')
reject('chat changes audience', function(sample)
  communication(sample, 1).intent.chat_channel = 2
end, 'Chinese chat mismatch')
reject('chat loses Chinese text', function(sample)
  communication(sample, 1).intent.text = 'different'
end, 'Chinese chat mismatch')
reject('chat smuggles origin point', function(sample)
  communication(sample, 1).intent.has_point = true
end, 'carried point')
reject('chat aliases queued', function(sample)
  communication(sample, 1).intent.queued = false
end, 'Communication carried queued')
reject('numeric objective zero is not an absent objective string', function(sample)
  communication(sample, 1).intent.objective_id = 0
end, 'carried or malformed objective id')
reject('chat cannot smuggle objective id', function(sample)
  communication(sample, 1).intent.objective_id = 'objective'
end, 'carried or malformed objective id')
reject('chat uses zero scope', function(sample)
  communication(sample, 1).intent.source_generation = 0
end, 'communication scope invalid')
reject('chat IPC precedes snapshot', function(sample)
  communication(sample, 1).ipc_sequence = 0
end, 'IPC precedes its snapshot')
reject('interaction catalog is not arbitrary', function(sample)
  communication(sample, 0).intent.content_id = 101
end, 'interaction mismatch')
local function mute_wire(sample)
  for _, line in ipairs(sample.intents) do
    if line.intent.kind == 'Communication' and line.intent.communication_kind == 4 then return line end
  end
  error('missing synthetic explicit mute wire')
end
reject('mute desired state is a boolean', function(sample)
  mute_wire(sample).intent.muted = 1
end, 'muted must be boolean')
reject('mute does not guess an inverse desired state', function(sample)
  mute_wire(sample).intent.muted = false
end, 'desired mute differs')
reject('mute stays bound to the disclosed sender', function(sample)
  mute_wire(sample).intent.mute_sender_id = 3
end, 'mute sender differs')
reject('mute cannot carry chat text', function(sample)
  mute_wire(sample).intent.text = '偽造'
end, 'mute shape differs')
local function ping_wire(sample)
  for _, line in ipairs(sample.intents) do if line.intent.communication_kind == 1 then return line end end
  error('missing ping wire')
end
reject('ping requires its actual semantic', function(sample) ping_wire(sample).intent.semantic=10 end, 'ping semantic differs')
reject('ping requires a point', function(sample) ping_wire(sample).intent.has_point=false end, 'ping semantic differs')
reject('ping point is fixed32', function(sample) ping_wire(sample).intent.point_x_raw=2147483648 end, 'outside fixed32')
reject('ping point cannot be null', function(sample) ping_wire(sample).intent.point_y_raw=json.null end, 'is not an integer')
reject('ping cannot smuggle emote content', function(sample) ping_wire(sample).intent.content_id=101 end, 'ping shape differs')
reject('wheel emote uses canonical content', function(sample)
  for _, line in ipairs(sample.intents) do if line.intent.content_id == 101 then line.intent.content_id=109; return end end
end, 'emote mismatch')
-- Independent communication-only fixture: no retained Saved files and no full-prefix claim.
local phase_bundle = copy()
phase_bundle.phase = 'communication'
phase_bundle.native.phase = 'communication'
phase_bundle.native.full_case_inventory = false
phase_bundle.history = {
  ack(1, 'communication_ready', 11, 3, 21, 3, 0),
  ack(2, 'communication_retire', 11, 4, 21, 4, 1),
  ack(3, 'communication_received', 11, 4, 21, 4, 2),
  ack(4, 'communication_retire', 11, 5, 21, 5, 3),
  ack(5, 'communication_ready', 11, 5, 21, 5, 4),
  ack(6, 'communication_received', 11, 5, 21, 5, 5),
  ack(7, 'communication_retire', 11, 6, 21, 6, 6),
  ack(8, 'communication_ready', 11, 6, 21, 6, 7),
  ack(9, 'communication_retire', 11, 7, 21, 7, 8),
  ack(10, 'communication_ready', 11, 7, 21, 7, 9),
  ack(11, 'communication_signals', 11, 7, 21, 7, 10),
  ack(12, 'communication_signals_expired', 11, 7, 21, 7, 11),
  ack(13, 'communication_retire', 11, 8, 21, 8, 12),
  ack(14, 'communication_hud', 11, 8, 21, 8, 13),
  ack(15, 'communication_retire', 11, 9, 21, 9, 14),
}
phase_bundle.ack = phase_bundle.history[#phase_bundle.history]
phase_bundle.native.cases, phase_bundle.intents = {}, {}
local phase_cases = ipc.expected_cases('communication')
assert(#phase_cases == 83 and #ipc.expected_stages('communication') == 15)
local phase_wire_policies = {
  {channel=1}, {channel=2}, {content=1}, {content=2}, {content=3}, {content=4}, {content=5},
  {mute=true}, {mute=false}, {channel=1}, {channel=2}, {channel=1,text='QWER A 集合'},
  {semantic=9}, {semantic=10}, {semantic=1}, {content=101},
  {semantic=9}, {semantic=9}, {semantic=9}, {semantic=9},
}
for _, policy in ipairs(additional_wheel_policies) do phase_wire_policies[#phase_wire_policies+1] = policy end
for _, policy in ipairs(shortcut_policies) do phase_wire_policies[#phase_wire_policies+1] = policy end
local phase_stage, phase_ordinal, phase_view, phase_published = 0, 0, 3, PUBLISHED_BASE
for _, spec in ipairs(phase_cases) do
  local before, after = ipc.stages_around(spec[1])
  assert(#after == 0, 'communication phase unexpectedly has after stages')
  for _, stage in ipairs(before) do
    phase_stage = phase_stage + 1
    local current = phase_bundle.history[phase_stage]
    assert(current.stage == stage)
    phase_view, phase_published = current.source_disclosure_epoch, current.published_sequence
  end
  local row = {name=spec[1],first_ordinal=0,last_ordinal=0,intents={}}
  if spec[2] then
    phase_ordinal = phase_ordinal + 1
    local line = wire(phase_ordinal, 'Communication', phase_view, 21, phase_view,
      assert(phase_wire_policies[phase_ordinal]), phase_published)
    phase_bundle.intents[#phase_bundle.intents+1] = line
    row.first_ordinal, row.last_ordinal, row.intents = phase_ordinal, phase_ordinal, {json.encode(line)}
  end
  phase_bundle.native.cases[#phase_bundle.native.cases+1] = row
end
assert(phase_stage == 15 and phase_ordinal == 49 and ipc.verify_artifacts(phase_bundle))
assert(#ipc.expected_cases() == 458 and #ipc.expected_stages() == 157, 'default full inventory changed')
do
  local rows, wires = ipc.expected_cases('accessibility'), 0
  for _,spec in ipairs(rows) do if spec[2] then wires=wires+(type(spec[3])=='table' and #spec[3] or 1) end end
  assert(#rows==14 and wires==7 and #ipc.expected_stages('accessibility')==5,'accessible input inventory changed')
  rows[13][3][1][2]=0
  assert(ipc.expected_cases('accessibility')[13][3][1][2]==32767,'accessible direction declarations alias returned copies')
end
count = count + 3
local function reject_phase(label, mutate, needle)
  local sample = json.decode(json.encode(phase_bundle)); mutate(sample)
  local ok, err = pcall(ipc.verify_artifacts, sample)
  assert(not ok and tostring(err):find(needle,1,true), label .. ': ' .. tostring(err))
  count = count + 1
end
reject_phase('feature phase cannot claim full inventory', function(sample) sample.native.full_case_inventory = true end,
  'phase cannot claim a full case inventory')
reject_phase('feature native must echo the requested phase', function(sample) sample.native.phase = 'full' end,
  'native phase does not match requested inventory')
reject_phase('feature artifacts cannot satisfy default full gate', function(sample) sample.phase = nil end,
  'native phase does not match requested inventory')
reject_phase('feature stages preserve actual extra envelopes', function(sample) sample.history[2].published_sequence = PUBLISHED_BASE + 2 end,
  'published sequence does not match actual IPC envelopes')
reject_phase('feature cases cannot omit a real shortcut', function(sample) table.remove(sample.native.cases) end,
  'native case inventory mismatch')
local unknown_phase = ipc.run({ue_root='D:/UE5.8',phase='invented'})
assert(not unknown_phase.success and unknown_phase.error == 'unknown owned viewport phase')
count = count + 1

local called = false
local selection = ipc.expected_cases('selection')
assert(#selection == 47 and selection[1][1] == 'selection-ready' and selection[12][1] == 'selection-final-clear' and selection[17][1]=='combat-idle-projected')
local selection_wires=0
for _,spec in ipairs(selection) do if spec[2] then selection_wires=selection_wires+(type(spec[3])=='table' and #spec[3] or 1) end end
assert(selection_wires==38 and selection[9][2]=='AttackTarget' and selection[10][2]=='AttackMove')
assert(#ipc.expected_stages('selection')==14 and ipc.expected_stages('selection')[1]=='baseline')
selection[1][1]='mutated';assert(ipc.expected_cases('selection')[1][1]=='selection-ready')
count=count+4
local selection_bundle=copy()
selection_bundle.phase='selection';selection_bundle.native.phase='selection';selection_bundle.native.full_case_inventory=false
selection_bundle.history={ack(1,'baseline',11,3,21,3,0),ack(2,'combat_defaults',11,3,21,3,0),
  ack(3,'combat_cursor_projection',11,3,21,3,0),ack(4,'combat_idle_off_projection',11,3,21,3,0),
  ack(5,'input_point',11,3,21,3,0),ack(6,'input_keyboard',11,3,21,3,0),
  ack(7,'input_point',11,3,21,3,0),ack(8,'input_locked',11,3,21,3,0),
  ack(9,'owner_point_ready',11,3,21,3,0),ack(10,'owner_upgrade_point',11,3,21,3,0),ack(11,'owner_cast_ready',11,3,21,3,0),
  ack(12,'owner_support_team_cards',11,3,21,3,0),ack(13,'owner_support_team_cards_dead',11,3,21,3,0),ack(14,'baseline',11,3,21,3,0)}
selection_bundle.ack=selection_bundle.history[14]
selection_bundle.native.cases={};selection_bundle.intents={}
local selection_stage=0
for _,spec in ipairs(ipc.expected_cases('selection')) do
  local before=ipc.stages_around(spec[1]);selection_stage=selection_stage+#before
  local row={name=spec[1],first_ordinal=0,last_ordinal=0,intents={}}
  if spec[2] then
    local repeats=type(spec[3])=='table' and #spec[3] or 1
    row.first_ordinal=#selection_bundle.intents+1
    for index=1,repeats do
      local ordinal=#selection_bundle.intents+1
      local policy=spec[2]=='CombatPreferences' and {idle=spec[3]=='cursor'} or type(spec[3])=='table' and spec[3][index] or spec[3]
      if spec[2]=='AbilityCast' or spec[2]=='AbilityUpgrade' then
        local slot,shape=spec[3]:match('^card:([0-3]):?(%a*)$');slot=tonumber(slot)
        policy={index=slot,catalog=1,id=slot==3 and 'lumen_mend' or 'lumen_bolt',shape=shape}
      end
      local line=wire(ordinal,spec[2],3,21,3,policy,PUBLISHED_BASE+selection_stage)
      selection_bundle.intents[ordinal]=line
      row.last_ordinal=ordinal;row.intents[#row.intents+1]=json.encode(line)
    end
  end
  selection_bundle.native.cases[#selection_bundle.native.cases+1]=row
end
assert(ipc.verify_artifacts(selection_bundle))
local function reject_selection(mutate,needle)
  local sample=json.decode(json.encode(selection_bundle));mutate(sample)
  local ok,err=pcall(ipc.verify_artifacts,sample)
  assert(not ok and tostring(err):find(needle,1,true),tostring(err));count=count+1
end
reject_selection(function(sample) sample.native.full_case_inventory=true end,'phase cannot claim a full case inventory')
reject_selection(function(sample) sample.native.cases[2].first_ordinal=1 end,'recorded an unexpected intent range')
reject_selection(function(sample) sample.intents[1].intent.target_render_id=999 end,'AttackTarget render id mismatch')
reject_selection(function(sample) sample.intents[2].authority_ack=true end,'intent claimed authority')
reject_selection(function(sample) sample.intents[3].intent.schema_version=2 end,'combat schema mismatch')
reject_selection(function(sample) sample.intents[3].intent.acquisition=256 end,'combat cursor preference mismatch')
reject_selection(function(sample) sample.intents[4].intent.idle_auto_attack=true end,'combat idle preference mismatch')
reject_selection(function(sample) sample.intents[3].intent.x_raw=1 end,'combat preference aliased x_raw')
local function change_direction(field,value)
  return function(sample)
    sample.intents[6].intent[field]=value
    sample.native.cases[21].intents[1]=json.encode(sample.intents[6])
  end
end
reject_selection(change_direction('x',123),'direction phase/axes/press sequence differs')
reject_selection(change_direction('press_id','99'),'direction phase/axes/press sequence differs')
reject_selection(change_direction('source_render_id','12'),'direction source differs from published owner')
reject_selection(change_direction('phase',2),'direction phase/axes/press sequence differs')
reject_selection(change_direction('queued',false),'Direction carried queued')
local detached=ipc.expected_cases('selection');detached[21][3][1][2]=999
assert(ipc.expected_cases('selection')[21][3][1][2]==0,'direction policy nested arrays were shared')
count=count+1
reject_selection(function(sample)
  sample.intents[33].intent.ability_index=1
  sample.native.cases[38].intents[1]=json.encode(sample.intents[33])
end,'card ability shape/slot mismatch')
reject_selection(function(sample)
  sample.intents[34].intent.ability_index=1
  sample.native.cases[39].intents[1]=json.encode(sample.intents[34])
end,'card upgrade slot mismatch')
reject_selection(function(sample)
  sample.intents[35].intent.ability_index=0
  sample.native.cases[40].intents[1]=json.encode(sample.intents[35])
end,'card ability shape/slot mismatch')
reject_selection(function(sample)
  sample.intents[36].intent.target_render_id=999
  sample.native.cases[43].intents[1]=json.encode(sample.intents[36])
end,'unit AbilityCast did not use the disclosed target')
reject_selection(function(sample)
  sample.intents[37].intent.ability_index=0
  sample.native.cases[44].intents[1]=json.encode(sample.intents[37])
end,'card ability shape/slot mismatch')
reject_selection(function(sample)
  sample.intents[38].authority_ack=true
  sample.native.cases[45].intents[1]=json.encode(sample.intents[38])
end,'intent claimed authority')
count=count+1
local masked_ok, masked_err = pcall(ipc.open_dependent_artifacts, {success = false, failure = 'intents.jsonl could not be read'}, function()
  called = true
  error('missing control-history.jsonl', 0)
end)
assert(not called, 'a failed native report still loaded dependent artifacts')
assert(not masked_ok and tostring(masked_err):find('intents.jsonl could not be read', 1, true), tostring(masked_err))
local empty_ok, empty_err = pcall(ipc.open_dependent_artifacts, {success = false, failure = ''}, function()
  error('loader should not run', 0)
end)
assert(not empty_ok and tostring(empty_err):find('native IPC report failed', 1, true), tostring(empty_err))
assert(ipc.open_dependent_artifacts({success = true}, function() return 7 end) == 7)
count = count + 3
local workflow = path.read(path.join(b.root, 'scripts/moba_owned_viewport_ipc.lua'))
assert(workflow:find('if path.is_file(foundation_path)', 1, true), 'foundation summary is still a required input')
assert(workflow:find('workflow report was not written', 1, true), 'a failed workflow write does not fail the run')
assert(workflow:find('player profile was not restored', 1, true), 'profile restore is not gated on editor death')
assert(not workflow:find('HISTORICAL_UE_STAGE_HASH', 1, true), 'historical stage hash is still a required workflow input')
local driver = path.read(path.join(b.root, 'omfue/bridge/src/driver.rs'))
local rate_at = assert(driver:find('fn validated_presentation_ready_rate', 1, true))
local rate_end = assert(driver:find('fn wall_clock_us', rate_at, true))
assert(driver:sub(rate_at, rate_end):find('ready.content_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH', 1, true),
  'ready admission does not check the catalog data hash')
local jsonl = path.read(path.join(b.root, 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmOwnedViewportJsonlTests.cpp'))
assert(jsonl:find('Om.Runtime.JsonlObservation', 1, true) and not jsonl:find('OMOBA_OWNED_VIEWPORT_IPC_FILE', 1, true),
  'the JSONL parser test is not a separate disconnected-suite file')
count = count + 6

-- Retained runtime artifacts are independently checked through public verify_artifacts.
do
  local direction={kind='Direction',schema_version=1,source_render_id='18446744073709551615',press_id='18446744073709551614',phase=1,x=-32767,y=32767}
  assert(ipc.validate_owner_control_wire_intent(direction));count=count+1
  for _,patch in ipairs({{press_id='0'},{press_id='01'},{press_id='18446744073709551615'},
    {press_id='18446744073709551616'},{press_id=12},{source_render_id='-1'},{phase=257},{x=32768},
    {phase=3},{queued=false},{sealed_source=1},{schema_version=256}})do
    local bad={};for key,value in pairs(direction)do bad[key]=value end
    for key,value in pairs(patch)do bad[key]=value end
    assert(not pcall(ipc.validate_owner_control_wire_intent,bad));count=count+1
  end
  direction.phase=3;direction.x=0;direction.y=0;assert(ipc.validate_owner_control_wire_intent(direction));count=count+1
  assert(ipc.validate_owner_control_wire_intent({kind='InputScheme',schema_version=1,scheme=1}));count=count+1
  assert(not pcall(ipc.validate_owner_control_wire_intent,{kind='InputScheme',schema_version=1,scheme=256}));count=count+1
end
-- Unit tests remain self-contained and do not require ignored local editor-run files.
do
  local coverage=require('ue_owned_phase_coverage')
  local foundation=copy()
  foundation.phase='foundation';foundation.native.phase='foundation';foundation.native.full_case_inventory=false
  for index=#foundation.native.cases,295,-1 do table.remove(foundation.native.cases,index) end
  for index=#foundation.intents,121,-1 do table.remove(foundation.intents,index) end
  for index=#foundation.history,104,-1 do table.remove(foundation.history,index) end
  foundation.ack=foundation.history[#foundation.history]
  assert(#ipc.expected_cases('foundation')==294 and #ipc.expected_stages('foundation')==103)
  assert(ipc.verify_artifacts(foundation));count=count+1
  local tail=json.decode(json.encode(bundle):gsub('ipc%-contract','ipc-contract-tail'))
  tail.phase='tail';tail.native.phase='tail';tail.native.full_case_inventory=false
  tail.startup.published_sequence=tail.history[103].published_sequence
  tail.startup.source_disclosure_epoch=tail.history[103].source_disclosure_epoch
  tail.startup.target_disclosure_epoch=tail.history[103].target_disclosure_epoch
  tail.native.source_disclosure_epoch=tail.startup.source_disclosure_epoch
  tail.native.target_disclosure_epoch=tail.startup.target_disclosure_epoch
  for index=1,294 do table.remove(tail.native.cases,1) end
  for index=1,120 do table.remove(tail.intents,1) end
  for index=1,103 do table.remove(tail.history,1) end
  for index,line in ipairs(tail.intents) do line.observation_ordinal=index end
  for index,stage in ipairs(tail.history) do stage.sequence=index end
  for _,record in ipairs(tail.native.cases) do
    if record.first_ordinal>0 then
      record.first_ordinal=record.first_ordinal-120;record.last_ordinal=record.last_ordinal-120
      for index=1,#record.intents do record.intents[index]=json.encode(tail.intents[record.first_ordinal+index-1]) end
    end
  end
  tail.ack=tail.history[#tail.history]
  tail.native.accessible_input={}
  for index,spec in ipairs(coverage.expected_cursor_observations()) do
    tail.native.accessible_input[index]={case=spec[1],observation=spec[2],user=0,
      before_x=1,before_y=2,after_x=spec[3] and 3 or 1,after_y=2,expected_motion=spec[3]}
  end
  local samples={foundation,tail}
  local accepted=coverage.verify_union(samples)
  assert(accepted.cases==458 and accepted.intents==222 and accepted.stages==157 and accepted.cursor_observations==13)
  count=count+1
  local function reject_union(mutate,needle)
    local bad=json.decode(json.encode(samples));mutate(bad)
    local ok,err=pcall(coverage.verify_union,bad)
    assert(not ok and tostring(err):find(needle,1,true),tostring(err));count=count+1
  end
  reject_union(function(s) table.remove(s) end,'phase count mismatch')
  reject_union(function(s) s[1],s[2]=s[2],s[1] end,'phase order mismatch')
  reject_union(function(s) s[2].cleanup=nil end,'cleanup proof is missing')
  reject_union(function(s) s[2].native.full_case_inventory=true end,'phase cannot claim a full case inventory')
  reject_union(function(s) table.remove(s[1].native.cases) end,'native case inventory mismatch')
  reject_union(function(s) s[2].native.generated_content_hash='different';s[2].native.compiled_content_hash='different' end,'phase content or ABI identity drifted')
  reject_union(function(s) table.remove(s[2].native.accessible_input) end,'cursor observation count mismatch')
  reject_union(function(s) s[2].native.accessible_input[1].after_x=1 end,'cursor motion proof differs')
  reject_union(function(s) s[2].native.accessible_input[3]=s[2].native.accessible_input[1] end,'cursor case mismatch')
  reject_union(function(s) s[2].cleanup.listener_released=false end,'listener was not released')
  reject_union(function(s) s[2].history[1].published_sequence=0 end,'published sequence does not match actual IPC envelopes')
end
print('owned viewport IPC contract: ' .. count .. ' cases passed')
