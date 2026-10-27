-- Owned viewport IPC workflow. The Rust publisher is a synthetic filtered feed.
-- A wire intent is local admission. It is not authority execution and not a positive ACK.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process, time, platform =
  b.lib('path'), b.lib('json'), b.lib('process'), b.lib('time'), b.lib('platform')
local fixture = require('moba_renderer_fixture')
local contract = require('ue_local_acceptance_contract')

local TEST = 'Om.Runtime.OwnedViewportIpc'
local OPERATION_MS = 860000
local GRACE_MS = 5000
local LIFETIME_SECONDS = 900
local CASES = {
  {'ground-rmb', 'MoveTo'},
  {'enemy-rmb', 'AttackTarget'},
  {'move-only-enemy', 'MoveTo'},
  {'attack-only-ground'},
  {'attack-only-ui'},
  {'armed-a-rmb', 'MoveTo'},
  {'indicator-q-rmb-release', 'MoveTo'},
  {'shift-a', 'AttackMove'},
  {'release-clears-hold', 'MoveTo'},
  {'text-focus-clears-hold', 'MoveTo'},
  {'modal-clears-hold', 'MoveTo'},
  {'window-focus-clears-hold', 'MoveTo'},
  {'source-death-clears-hold', 'MoveTo'},
  {'baseline-after-source-death'},
  {'source-death-clears-indicator'},
  {'baseline-after-indicator-death'},
  {'source-replacement-clears-hold', 'MoveTo'},
  {'source-removal-clears-hold', 'MoveTo'},
  {'baseline-restores-source'},
  {'epoch-attack', 'AttackTarget'},
  {'replaced-target-attack', 'AttackTarget'},
  {'baseline-canonical-pair'},
  {'armed-a-world-lmb', 'AttackMove'},
  {'armed-a-ui-lmb'},
  {'armed-a-esc'},
  {'repeat-identical', 'MoveTo'},
  {'repeat-changed-point', 'MoveTo'},
  {'repeat-large-delta', 'MoveTo'},
  {'repeat-release'},
  {'repeat-ui-focus', 'MoveTo'},
  {'repeat-profile-stops-hold', 'MoveTo'},
  {'dual-primary-rmb', 'MoveTo'},
  {'dual-secondary-key', 'MoveTo'},
  {'dual-identical-delta'},
  {'dual-changed-point', 'MoveTo'},
  {'dual-both-large-delta', 'MoveTo'},
  {'dual-release-primary'},
  {'dual-final-release'},
  {'hold-clock-armed', 'MoveTo'},
  {'hold-clock-join', 'MoveTo'},
  {'hold-clock-partial', 'MoveTo'},
  {'hold-clock-fresh', 'MoveTo'},
  {'cast-secondary-keeps-token'},
  {'cast-esc-cancels'},
  {'cast-right-cancels', 'MoveTo'},
  {'cast-focus-cancels'},
  {'cast-profile-cancels'},
  {'ability-cast-ready'},
  {'q-normal-unit', 'AbilityCast'},
  {'q-quick-unit', 'AbilityCast'},
  {'q-indicator-unit', 'AbilityCast'},
  {'w-normal-self', 'AbilityCast'},
  {'w-quick-self', 'AbilityCast'},
  {'w-indicator-self', 'AbilityCast'},
  {'e-normal-unit', 'AbilityCast'},
  {'e-quick-unit', 'AbilityCast'},
  {'e-indicator-unit', 'AbilityCast'},
  {'r-normal-self', 'AbilityCast'},
  {'r-quick-self', 'AbilityCast'},
  {'r-indicator-self', 'AbilityCast'},
  {'q-esc-cancels'},
  {'q-right-cancels', 'MoveTo'},
  {'q-ui-does-not-confirm'},
  {'q-focus-cancels'},
  {'q-modal-cancels'},
  {'q-stale-release'},
  {'shift-q-plain', 'AbilityCast'},
  {'shift-q-indicator', 'AbilityCast'},
  {'shift-q-modifier-up', 'AbilityCast'},
  {'alt-w-self', 'AbilityCast'},
  {'alt-q-refuses'},
  {'upgrade-no-points'},
  {'ability-upgrade-point'},
  {'upgrade-q', 'AbilityUpgrade'},
  {'ability-short-mana'},
  {'q-mana-refuses'},
  {'ability-cooldown'},
  {'q-cooldown-refuses'},
  {'ability-hide'},
  {'q-hide-refuses'},
  {'ability-far'},
  {'q-far-refuses'},
  {'ability-apprentice'},
  {'upgrade-apprentice-q', 'AbilityUpgrade'},
  {'upgrade-apprentice-e'},
  {'ability-point-ready'},
  {'r-normal-point', 'AbilityCast'},
  {'r-quick-point', 'AbilityCast'},
  {'r-indicator-point', 'AbilityCast'},
  {'ability-cast-ready-2'},
  {'q-dead-cancels'},
  {'ability-restore-dead'},
  {'ability-cast-ready-3'},
  {'q-replaced-cancels'},
  {'ability-restore-replaced'},
  {'ability-cast-ready-4'},
  {'q-removed-cancels'},
  {'ability-restore-removed'},
  {'ability-cast-ready-5'},
  {'q-epoch-cancels'},
  {'q-target-dead'},
  {'ability-support-ready'},
  {'r-aid-ally', 'AbilityCast'},
  {'alt-r-aid-self', 'AbilityCast', 'source'},
  {'ability-support-enemy'},
  {'r-aid-enemy-refuses'},
  {'ability-support-conceal'},
  {'r-aid-conceal-refuses'},
  {'ability-support-dead'},
  {'r-aid-dead-refuses'},
  {'ability-support-neutral'},
  {'r-aid-neutral-refuses'},
  {'minimap-point-ready'},
  {'minimap-move-off'},
  {'minimap-move-on', 'MoveTo'},
  {'minimap-cast-off'},
  {'minimap-normal-point', 'AbilityCast'},
  {'minimap-quick-point', 'AbilityCast'},
  {'minimap-indicator-point', 'AbilityCast'},
  {'minimap-unit-no-cast'},
  {'minimap-right-cancels'},
  {'summoner-ready'},
  {'d-normal-point', 'SummonerCast'},
  {'d-quick-point', 'SummonerCast'},
  {'d-indicator-point', 'SummonerCast'},
  {'f-normal-self', 'SummonerCast'},
  {'f-quick-self', 'SummonerCast'},
  {'f-indicator-self', 'SummonerCast'},
  {'d-esc-cancels'},
  {'d-right-cancels', 'MoveTo'},
  {'d-ui-does-not-confirm'},
  {'d-focus-cancels'},
  {'d-modal-cancels'},
  {'d-stale-release', 'SummonerCast'},
  {'d-secondary-once', 'SummonerCast'},
  {'alt-d-refuses'},
  {'summoner-cooldown'},
  {'d-cooldown-refuses'},
  {'f-cooled-d-still-heals', 'SummonerCast'},
  {'summoner-unavailable'},
  {'d-unequipped-refuses'},
  {'summoner-malformed'},
  {'d-malformed-refuses'},
  {'summoner-unsupported'},
  {'d-unsupported-refuses'},
  {'summoner-ready-2'},
  {'d-dead-cancels'},
  {'d-dead-refuses'},
  {'summoner-restore-dead'},
  {'summoner-ready-3'},
  {'d-replaced-cancels'},
  {'d-replaced-point', 'SummonerCast'},
  {'summoner-restore-replaced'},
  {'summoner-ready-4'},
  {'d-removed-cancels'},
  {'summoner-restore-removed'},
  {'summoner-ready-5'},
  {'d-profile-cancels'},
  {'display-hud-hide'},
  {'display-hud-show'},
  {'display-modal-suppresses'},
  {'display-text-suppresses'},
  {'display-secondary-health'},
  {'display-names-toggle'},
  {'display-settings-visible'},
  {'s-stop', 'Stop'},
  {'s-stop-repeat', 'Stop'},
  {'h-hold', 'HoldPosition'},
  {'h-hold-repeat', 'HoldPosition'},
  {'s-stop-after-hold', 'Stop'},
  {'stop-secondary', 'Stop'},
  {'hold-secondary', 'HoldPosition'},
  {'stop-text'},
  {'hold-modal'},
  {'recall-ready'},
  {'b-recall', 'Recall'},
  {'recall-channel'},
  {'b-channel-admits', 'Recall'},
  {'recall-unavailable'},
  {'b-unavailable'},
  {'recall-ready-2'},
  {'b-focus'},
  {'b-dead'},
  {'recall-restore'},
  {'item-ready'},
  {'item1-self', 'ItemUse', '0:self:5:moba_guard_charm'},
  {'item1-quick', 'ItemUse', '0:self:5:moba_guard_charm'},
  {'item1-indicator', 'ItemUse', '0:self:5:moba_guard_charm'},
  {'item2-normal-point', 'ItemUse', '1:point:10:moba_training_flare'},
  {'item2-quick-point', 'ItemUse', '1:point:10:moba_training_flare'},
  {'item2-indicator-point', 'ItemUse', '1:point:10:moba_training_flare'},
  {'item3-normal-unit', 'ItemUse', '2:unit:11:moba_training_hex'},
  {'item3-quick-unit', 'ItemUse', '2:unit:11:moba_training_hex'},
  {'item3-indicator-unit', 'ItemUse', '2:unit:11:moba_training_hex'},
  {'alt-item1-self', 'ItemUse', '0:self:5:moba_guard_charm'},
  {'alt-item2-refuses'},
  {'alt-item3-refuses'},
  {'item-shift-refuses'},
  {'item5-passive'},
  {'item-empty'},
  {'item-empty-press'},
  {'item-cooldown'},
  {'item-cooldown-press'},
  {'item-ready-2'},
  {'item-far'},
  {'item2-far-refuses'},
  {'item3-far-refuses'},
  {'item-ready-3'},
  {'item2-esc'},
  {'item2-right', 'MoveTo'},
  {'item2-ui'},
  {'item2-stale'},
  {'item2-replaced-self', 'ItemUse', '1:self:6:moba_stride_charm'},
  {'item-ready-4'},
  {'item2-profile'},
  {'item2-secondary', 'ItemUse', '1:point:10:moba_training_flare'},
  {'item-dead'},
  {'item-restore'},
  {'item-ready-5'},
  {'item6-self', 'ItemUse', '4:self:6:moba_stride_charm'},
  {'item6-quick', 'ItemUse', '4:self:6:moba_stride_charm'},
  {'item6-indicator', 'ItemUse', '4:self:6:moba_stride_charm'},
  {'alt-item6-self', 'ItemUse', '4:self:6:moba_stride_charm'},
  {'item7-self', 'ItemUse', '5:self:7:moba_mana_charm'},
  {'item7-quick', 'ItemUse', '5:self:7:moba_mana_charm'},
  {'item7-indicator', 'ItemUse', '5:self:7:moba_mana_charm'},
  {'alt-item7-self', 'ItemUse', '5:self:7:moba_mana_charm'},
  {'s-stop-held-repeat', 'Stop'},
  {'h-hold-held-repeat', 'HoldPosition'},
  {'stop-modified'},
  {'hold-modified'},
  {'stop-secondary-held-repeat', 'Stop'},
  {'hold-secondary-held-repeat', 'HoldPosition'},
  {'item2-held-focus'},
  {'item2-held-modal'},
  {'item2-held-dead'},
  {'item-alive-restore'},
  {'item-ready-6'},
  {'item2-held-replaced'},
  {'item-restore-replaced'},
  {'item-ready-7'},
  {'item2-held-removed'},
  {'item-restore-removed'},
  {'stop-text-recover', 'Stop', 'twice'},
  {'hold-modal-recover', 'HoldPosition', 'twice'},
  {'stop-outside-recover', 'Stop', 'twice'},
  {'hold-modifier-recover', 'HoldPosition', 'twice'},
  {'stop-refused-recover', 'Stop'},
  {'stop-source-held', 'Stop'},
  {'stop-source-restore'},
  {'stop-profile-replace', 'Stop', 'twice'},
  {'recall-label-rebind'},
  {'recall-dead-label'},
  {'item-card-ready'},
  {'item-card-map'},
  {'item-card-self', 'ItemUse', '0:self:5:moba_guard_charm'},
  {'item-card-point-quick', 'ItemUse', '1:point:10:moba_training_flare'},
  {'item-card-unit-indicator', 'ItemUse', '2:unit:11:moba_training_hex'},
  {'item-card-passive'},
  {'item-card-empty'},
  {'item-card-cooldown'},
  {'item-card-dead'},
  {'item-card-alive'},
  {'item-card-ready-2'},
  {'item-card-esc'},
  {'item-card-right', 'MoveTo'},
  {'item-card-held-dead'},
  {'item-card-restore-dead'},
  {'item-card-ready-3'},
  {'item-card-held-replaced'},
  {'item-card-restore-replaced'},
  {'item-card-ready-4'},
  {'item-card-held-removed'},
  {'item-card-restore-removed'},
  {'mode-moba-controls'},
  {'mode-td-controls'},
  {'mode-td-defeat'},
  {'mode-td-no-hero'},
  {'mode-moba-dead'},
  {'mode-unavailable'},
  {'mode-td-restore'},
  {'mode-td-select'},
  {'mode-td-select-absent'},
  {'mode-td-select-stale'},
  {'mode-epoch-td'},
  {'mode-td-select-replaced'},
  {'mode-td-select-current'},
  {'mode-chrome-quiet'},
  {'mode-td-remove-stale-gen'},
  {'mode-retire'},
  -- Baseline restores source 11 / target 21 / Moba at the current epoch. The shop stage then
  -- publishes the same six slots and gold 0 with protocol enabled. Browsing adds no wire.
  -- The buy policy names the authored item; the wire carries catalog_id only.
  {'shop-restore'},
  {'shop-open'},
  {'shop-recipe-browse'},
  {'shop-recipe-buy', 'ItemBuy', '4:moba_greatsword'},
  -- Appended after the shop prefix. Projections keep the current source actor.
  -- source-retire then replaces that actor without advancing the view floor.
  -- The status case forwards, then uncertain, then settled. Rejected is its own case.
  {'ward-ready-status'},
  {'ward-point', 'WardPlacement'},
  {'ward-esc'},
  {'ward-dead'},
  {'ward-quick', 'WardPlacement'},
  {'ward-indicator', 'WardPlacement'},
  {'ward-normal-release'},
  {'ward-source-retire', 'WardPlacement'},
  {'ward-status-uncertain-then-settled', 'WardPlacement'},
  {'ward-status-rejected'},
  -- Appended after the ward prefix. The current source stays 12. Gold stays 0.
  -- Browsing is not repeated. Each wire is one buy or one sell.
  {'shop-txn-ready'},
  {'shop-txn-buy', 'ItemBuy', '4:moba_greatsword'},
  {'shop-txn-admitted'},
  {'shop-txn-uncertain'},
  {'shop-txn-settled'},
  {'shop-txn-sell', 'ItemSell', '0'},
  {'shop-txn-rejected'},
  {'shop-txn-stale', 'ItemBuy', '4:moba_greatsword'},
  {'shop-txn-epoch'},
  -- No wire. Sixty-four admitted buys fill the pending table at the current epoch.
  {'shop-txn-full'},
  {'chat-ready'},
  {'chat-team-chinese', 'Communication', 'chat:1'},
  {'chat-all-chinese', 'Communication', 'chat:2'},
  {'chat-escape'},
  {'chat-focus-cancel'},
  {'chat-history-toggle'},
  {'interaction-1', 'Communication', 'interaction:1'},
  {'interaction-2', 'Communication', 'interaction:2'},
  {'interaction-3', 'Communication', 'interaction:3'},
  {'interaction-4', 'Communication', 'interaction:4'},
  {'interaction-5', 'Communication', 'interaction:5'},
  {'chat-scope-retire'},
  {'chat-unavailable'},
  {'chat-received-800'},
  {'chat-received-400'},
  {'chat-received-large'},
  {'chat-mute', 'Communication', 'mute:1'},
  {'chat-mute-scope-retire'},
  {'chat-roster-reconnected'},
  {'chat-unmute', 'Communication', 'mute:0'},
  {'chat-all-disabled'},
  {'chat-team-all-disabled', 'Communication', 'chat:1'},
  {'chat-received-all-disabled'},
  {'chat-all-reenabled', 'Communication', 'chat:2'},
  {'chat-text-controls', 'Communication', 'chat-text:1'},
  {'wheel-source-retire'}, {'wheel-ready'},
  {'ping-general','Communication','ping:9'}, {'ping-warning','Communication','ping:10'},
  {'ping-wheel-north','Communication','ping:1'}, {'emote-wheel-north','Communication','emote:101'},
  {'wheel-right-cancel'}, {'wheel-escape-cancel'}, {'wheel-foreign-release'}, {'wheel-text-guard'},
  {'wheel-focus-cancel'}, {'wheel-profile-cancel'}, {'wheel-retire-again'}, {'wheel-ready-again'},
  {'ping-alt-pointer','Communication','ping:9'}, {'ping-ctrl-pointer','Communication','ping:9'},
  {'ping-minimap-alt','Communication','ping:9'}, {'ping-minimap-ctrl','Communication','ping:9'},
  {'ping-wheel-400','Communication','ping:1'},
  {'emote-wheel-400','Communication','emote:101'},
  {'ping-wheel-large','Communication','ping:1'},
  {'emote-wheel-large','Communication','emote:101'},
  {'ping-wheel-sector-2','Communication','ping:2'},
  {'ping-wheel-sector-3','Communication','ping:3'},
  {'ping-wheel-sector-4','Communication','ping:4'},
  {'ping-wheel-sector-5','Communication','ping:5'},
  {'ping-wheel-sector-6','Communication','ping:6'},
  {'ping-wheel-sector-7','Communication','ping:7'},
  {'ping-wheel-sector-8','Communication','ping:8'},
  {'emote-wheel-sector-2','Communication','emote:102'},
  {'emote-wheel-sector-3','Communication','emote:103'},
  {'emote-wheel-sector-4','Communication','emote:104'},
  {'emote-wheel-sector-5','Communication','emote:105'},
  {'emote-wheel-sector-6','Communication','emote:106'},
  {'emote-wheel-sector-7','Communication','emote:107'},
  {'emote-wheel-sector-8','Communication','emote:108'},
  {'emote-center-cancel'},
  {'ping-repeat','Communication','ping:9'},
  {'signals-received-800'}, {'signals-received-400'}, {'signals-received-large'},
  {'signals-expired'}, {'signals-retired'}, {'combat-overflow-scroll'},
  {'hud-status-q','Communication','status:1:0:0'},
  {'hud-status-w','Communication','status:1:1:0'},
  {'hud-status-e','Communication','status:1:2:0'},
  {'hud-status-r','Communication','status:1:3:0'},
  {'hud-status-item-0','Communication','status:2:0:0'},
  {'hud-status-item-1','Communication','status:2:1:0'},
  {'hud-status-item-2','Communication','status:2:2:0'},
  {'hud-status-score-2','Communication','status:3:0:2'},
  {'objective-vote-north','Communication','objective:north_guardian'},
  {'objective-vote-south','Communication','objective:south_guardian'},
  {'hud-status-foreign-user'},
  {'hud-status-text-guard'},
  {'hud-status-retire'},
  {'hud-status-retired-click'},

}
local SELECTION_CASES = {
  {'selection-ready'}, {'selection-enemy'}, {'selection-escape'}, {'selection-self'},
  {'selection-ground'}, {'selection-keyboard'}, {'selection-old-mouse'}, {'selection-keyboard-clear'},
  {'selection-attack','AttackTarget'}, {'selection-armed-a','AttackMove'}, {'selection-400'}, {'selection-final-clear'},
  {'combat-ready'}, {'combat-cursor-submit','CombatPreferences','cursor'}, {'combat-cursor-projected'},
  {'combat-idle-submit','CombatPreferences','auto'}, {'combat-idle-projected'},
  {'input-point-ready'}, {'input-request-keyboard','InputScheme','1'}, {'input-keyboard-projected'},
  {'input-tap-w','Direction',{{1,0,32767,'1'},{3,0,0,'1'}}},
  {'input-diagonal','Direction',{{1,0,32767,'2'},{2,32767,32767,'2'},{2,32767,0,'2'},{3,0,0,'2'}}},
  {'input-opposed','Direction',{{1,0,32767,'3'},{3,0,0,'3'},{1,0,32767,'4'},{3,0,0,'4'}}},
  {'input-repeat','Direction',{{1,0,32767,'5'},{3,0,0,'5'}}},
  {'input-queue','Direction',{{1,0,32767,'6'},{3,0,0,'6'}}},
  {'input-rotate','Direction',{{1,-23170,23170,'7'},{3,0,0,'7'}}},
  {'input-rebind','Direction',{{1,0,32767,'8'},{3,0,0,'8'}}}, {'input-old-binding'},
  {'input-restore-binding','Direction',{{1,0,32767,'9'},{3,0,0,'9'}}},
  {'input-heartbeat','Direction',{{1,0,32767,'10'},{2,0,32767,'10'},{3,0,0,'10'}}},
  {'input-stall','Direction',{{1,0,32767,'11'},{2,0,32767,'11'},{3,0,0,'11'}}},
  {'input-modal'}, {'input-request-point','InputScheme','0'}, {'input-point-projected'}, {'input-locked'},
  {'ability-card-ready'}, {'ability-card-arm'}, {'ability-card-confirm','AbilityCast','card:3:point'},
  {'ability-card-upgrade','AbilityUpgrade','card:0'}, {'ability-card-self','AbilityCast','card:3:none'}, {'ability-card-modal'},
  {'ally-card-ready'}, {'ally-card-normal','AbilityCast','card:3:unit'}, {'ally-card-quick','AbilityCast','card:3:unit'},
  {'ally-card-indicator','AbilityCast','card:3:unit'}, {'ally-card-dead'}, {'ally-card-retire'},
}
local ACCESSIBILITY_CASES = {
  {'accessible-ready'}, {'accessible-keyboard-cursor'}, {'accessible-cursor-speed'},
  {'accessible-analog-cursor'}, {'accessible-cursor-focus'}, {'accessible-text-guard'},
  {'accessible-confirm-mouse-a','AttackMove'}, {'accessible-confirm-key-a','AttackMove'},
  {'accessible-confirm-shared-r','AbilityCast','card:3:point'},
  {'accessible-confirm-minimap','AbilityCast','card:3:point'}, {'accessible-source-retire'},
  {'accessible-direction-ready'},
  {'accessible-analog-direction','Direction',{{1,32767,0,'1'},{2,32767,0,'1'},{3,0,0,'1'}}},
  {'accessible-restored'},
}
local STAGE_AFTER = {
  ['accessible-source-retire'] = 'disclosure_epoch',
  ['source-death-clears-hold'] = 'source_death',
  ['baseline-after-source-death'] = 'baseline',
  ['source-death-clears-indicator'] = 'source_death',
  ['baseline-after-indicator-death'] = 'baseline',
  ['source-replacement-clears-hold'] = 'source_replacement',
  ['source-removal-clears-hold'] = 'source_removal',
  ['baseline-restores-source'] = 'baseline',
  ['baseline-canonical-pair'] = 'baseline',
  ['q-dead-cancels'] = 'source_death',
  ['q-replaced-cancels'] = 'source_replacement',
  ['q-removed-cancels'] = 'source_removal',
  ['q-epoch-cancels'] = 'disclosure_epoch',
  ['d-dead-cancels'] = 'source_death',
  ['d-replaced-cancels'] = 'source_replacement',
  ['d-removed-cancels'] = 'source_removal',
  ['item2-stale'] = 'owner_item_replaced',
  ['item2-held-dead'] = 'source_death',
  ['item2-held-replaced'] = 'source_replacement',
  ['item2-held-removed'] = 'source_removal',
  ['stop-source-held'] = 'source_death',
  ['recall-dead-label'] = 'source_death',
  ['item-card-held-dead'] = 'source_death',
  ['item-card-held-replaced'] = 'source_replacement',
  ['item-card-held-removed'] = 'source_removal',
  ['ward-source-retire'] = 'controlled_owner_ward_same_epoch_source',
  ['ward-status-uncertain-then-settled'] = {
    'controlled_owner_ward_forwarded',
    'controlled_owner_ward_uncertain_mapped',
    'controlled_owner_ward_settled_mapped',
  },
  ['shop-txn-stale'] = 'controlled_owner_shop_stale',
}
local STAGE_BEFORE = {
  ['accessible-ready'] = 'input_point',
  ['accessible-confirm-shared-r'] = 'owner_point_ready',
  ['accessible-direction-ready'] = 'input_keyboard',
  ['accessible-restored'] = 'input_point',
  ['ally-card-ready'] = 'owner_support_team_cards', ['ally-card-dead'] = 'owner_support_team_cards_dead', ['ally-card-retire'] = 'baseline',
  ['ability-card-ready'] = 'owner_point_ready', ['ability-card-upgrade'] = 'owner_upgrade_point', ['ability-card-self'] = 'owner_cast_ready',
  ['input-point-ready'] = 'input_point', ['input-keyboard-projected'] = 'input_keyboard',
  ['input-point-projected'] = 'input_point', ['input-locked'] = 'input_locked',
  ['combat-ready'] = 'combat_defaults',
  ['combat-cursor-projected'] = 'combat_cursor_projection',
  ['combat-idle-projected'] = 'combat_idle_off_projection',
  ['selection-ready'] = 'baseline',
  ['epoch-attack'] = 'disclosure_epoch',
  ['replaced-target-attack'] = 'replace_target',
  ['ability-cast-ready'] = 'owner_cast_ready',
  ['ability-upgrade-point'] = 'owner_upgrade_point',
  ['ability-short-mana'] = 'owner_short_mana',
  ['ability-cooldown'] = 'owner_cooldown',
  ['ability-hide'] = 'owner_hide_target',
  ['ability-far'] = 'owner_far_unit',
  ['ability-apprentice'] = 'owner_apprentice',
  ['ability-point-ready'] = 'owner_point_ready',
  ['ability-cast-ready-2'] = 'owner_cast_ready',
  ['ability-restore-dead'] = 'baseline',
  ['ability-cast-ready-3'] = 'owner_cast_ready',
  ['ability-restore-replaced'] = 'baseline',
  ['ability-cast-ready-4'] = 'owner_cast_ready',
  ['ability-restore-removed'] = 'baseline',
  ['ability-cast-ready-5'] = 'owner_cast_ready',
  ['q-target-dead'] = 'death',
  ['ability-support-ready'] = 'owner_support_ready',
  ['ability-support-enemy'] = 'owner_support_enemy',
  ['ability-support-conceal'] = 'owner_support_conceal',
  ['ability-support-dead'] = 'owner_support_dead',
  ['ability-support-neutral'] = 'owner_support_neutral',
  ['minimap-point-ready'] = 'owner_point_ready',
  ['summoner-ready'] = 'owner_summoner_ready',
  ['summoner-cooldown'] = 'owner_summoner_cooldown',
  ['summoner-unavailable'] = 'owner_summoner_unavailable',
  ['summoner-malformed'] = 'owner_summoner_malformed',
  ['summoner-unsupported'] = 'owner_summoner_unsupported',
  ['summoner-ready-2'] = 'owner_summoner_ready',
  ['summoner-restore-dead'] = 'baseline',
  ['summoner-ready-3'] = 'owner_summoner_ready',
  ['summoner-restore-replaced'] = 'baseline',
  ['summoner-ready-4'] = 'owner_summoner_ready',
  ['summoner-restore-removed'] = 'baseline',
  ['summoner-ready-5'] = 'owner_summoner_ready',
  ['recall-ready'] = 'owner_recall_ready',
  ['recall-channel'] = 'owner_recall_channel',
  ['recall-unavailable'] = 'owner_recall_unavailable',
  ['recall-ready-2'] = 'owner_recall_ready',
  ['b-dead'] = 'source_death',
  ['recall-restore'] = 'baseline',
  ['item-ready'] = 'owner_item_ready',
  ['item-empty'] = 'owner_item_empty',
  ['item-cooldown'] = 'owner_item_cooldown',
  ['item-ready-2'] = 'owner_item_ready',
  ['item-far'] = 'owner_item_far',
  ['item-ready-3'] = 'owner_item_ready',
  ['item-ready-4'] = 'owner_item_ready',
  ['item-dead'] = 'source_death',
  ['item-restore'] = 'baseline',
  ['item-ready-5'] = 'owner_item_ready',
  ['item-alive-restore'] = 'baseline',
  ['item-ready-6'] = 'owner_item_ready',
  ['item-restore-replaced'] = 'baseline',
  ['item-ready-7'] = 'owner_item_ready',
  ['item-restore-removed'] = 'baseline',
  ['stop-source-restore'] = 'baseline',
  ['recall-dead-label'] = 'owner_recall_channel',
  ['item-card-ready'] = 'owner_item_ready',
  ['item-card-empty'] = 'owner_item_empty',
  ['item-card-cooldown'] = 'owner_item_cooldown',
  ['item-card-dead'] = 'source_death',
  ['item-card-alive'] = 'baseline',
  ['item-card-ready-2'] = 'owner_item_ready',
  ['item-card-restore-dead'] = 'baseline',
  ['item-card-ready-3'] = 'owner_item_ready',
  ['item-card-restore-replaced'] = 'baseline',
  ['item-card-ready-4'] = 'owner_item_ready',
  ['item-card-restore-removed'] = 'baseline',
  ['mode-td-controls'] = 'mode_tower_defense',
  ['mode-td-defeat'] = 'mode_td_defeat',
  ['mode-td-no-hero'] = 'mode_td_no_hero',
  ['mode-moba-dead'] = 'mode_moba_dead',
  ['mode-unavailable'] = 'mode_unavailable',
  ['mode-td-restore'] = 'mode_tower_defense',
  ['mode-epoch-td'] = 'mode_td_epoch',
  ['shop-restore'] = 'baseline',
  ['shop-open'] = 'owner_shop_ready',
  ['ward-ready-status'] = 'controlled_owner_ward_projection',
  ['ward-dead'] = 'controlled_owner_ward_dead',
  ['ward-quick'] = 'controlled_owner_ward_projection',
  ['ward-indicator'] = 'controlled_owner_ward_projection',
  ['ward-normal-release'] = 'controlled_owner_ward_projection',
  ['ward-source-retire'] = 'controlled_owner_ward_projection',
  ['ward-status-uncertain-then-settled'] = 'controlled_owner_ward_projection',
  ['ward-status-rejected'] = 'controlled_owner_ward_rejected_mapped',
  ['shop-txn-ready'] = 'controlled_owner_shop_current',
  ['shop-txn-admitted'] = 'controlled_owner_shop_admitted',
  ['shop-txn-uncertain'] = 'controlled_owner_shop_uncertain',
  ['shop-txn-settled'] = 'controlled_owner_shop_settled',
  ['shop-txn-rejected'] = 'controlled_owner_shop_rejected',
  ['shop-txn-epoch'] = 'controlled_owner_shop_epoch',
  ['shop-txn-full'] = 'controlled_owner_shop_full',
  ['chat-ready'] = 'communication_ready',
  ['chat-scope-retire'] = 'communication_retire',
  ['chat-received-800'] = 'communication_received',
  ['chat-mute-scope-retire'] = 'communication_retire',
  ['chat-roster-reconnected'] = 'communication_ready',
  ['chat-received-all-disabled'] = 'communication_received',
  ['wheel-source-retire'] = 'communication_retire', ['wheel-ready'] = 'communication_ready',
  ['wheel-retire-again'] = 'communication_retire', ['wheel-ready-again'] = 'communication_ready',
  ['signals-received-800'] = 'communication_signals', ['signals-expired'] = 'communication_signals_expired',
  ['signals-retired'] = 'communication_retire',
  ['hud-status-q'] = 'communication_hud', ['hud-status-retire'] = 'communication_retire',
}

local M = {}

local function same_path(left, right)
  left, right = path.absolute(left), path.absolute(right)
  if platform.is_windows then return left:lower() == right:lower() end
  return left == right
end

local function is_entry()
  local entry = arg and arg[0]
  return type(entry) == 'string' and same_path(entry, source)
end

function M.expected_cases(phase)
  phase = phase or 'full'
  assert(phase == 'full' or phase == 'foundation' or phase == 'communication' or phase == 'selection' or phase == 'accessibility' or phase == 'tail', 'unknown owned viewport phase')
  if phase == 'foundation' then
    local copy = {}
    for _, spec in ipairs(M.expected_cases('full')) do
      if spec[1] == 'ward-ready-status' then return copy end
      copy[#copy + 1] = spec
    end
    error('owned foundation boundary is missing')
  end
  if phase == 'tail' then
    local copy, selected = {}, false
    for _,spec in ipairs(M.expected_cases('full'))do
      if spec[1]=='ward-ready-status' then selected=true end
      if selected then copy[#copy+1]=spec end
    end
    assert(#copy>0,'owned tail boundary is missing')
    return copy
  end
  if phase == 'selection' or phase == 'accessibility' then
    local copy = {}; for _,spec in ipairs(phase == 'selection' and SELECTION_CASES or ACCESSIBILITY_CASES) do copy[#copy+1]={spec[1],spec[2],type(spec[3])=='table' and json.decode(json.encode(spec[3])) or spec[3]} end
    return copy
  end
  local copy, selected = {}, phase == 'full'
  for _, spec in ipairs(CASES) do
    if spec[1] == 'chat-ready' then selected = true end
    if selected then copy[#copy + 1] = {spec[1], spec[2], spec[3]} end
  end
  if phase == 'full' then for _,spec in ipairs(SELECTION_CASES)do copy[#copy+1]={spec[1],spec[2],type(spec[3])=='table' and json.decode(json.encode(spec[3])) or spec[3]} end end
  if phase == 'full' then for _,spec in ipairs(ACCESSIBILITY_CASES)do copy[#copy+1]={spec[1],spec[2],type(spec[3])=='table' and json.decode(json.encode(spec[3])) or spec[3]} end end
  assert(#copy > 0, 'owned viewport phase is empty')
  return copy
end

function M.expected_case_names(phase)
  local names = {}
  for _, spec in ipairs(M.expected_cases(phase)) do names[#names + 1] = spec[1] end
  return names
end

local function integer(value, label)
  assert(math.type(value) == 'integer', label .. ' is not an integer')
  return value
end

local function lines_of(text)
  local out = {}
  for line in (text .. '\n'):gmatch('([^\n]*)\n') do
    line = line:gsub('\r$', '')
    if line ~= '' then out[#out + 1] = json.decode(line) end
  end
  return out
end

local function same_json(left, right)
  return json.encode(left) == json.encode(right)
end

-- A declaration is one stage name or an ordered list. Callers cannot share the returned table.
local function stages_of(name, which)
  local value = (which == 'before' and STAGE_BEFORE or STAGE_AFTER)[name]
  if value == nil then return {} end
  if type(value) == 'string' then
    assert(value ~= '', name .. ' has an empty ' .. which .. ' stage')
    return {value}
  end
  assert(type(value) == 'table', name .. ' stage declaration is not a string or an ordered list')
  assert(#value > 0, name .. ' has an empty ' .. which .. ' stage list')
  local stages = {}
  for index, stage in ipairs(value) do
    assert(type(stage) == 'string' and stage ~= '', name .. ' stage list is malformed')
    stages[index] = stage
  end
  assert(#stages == #value, name .. ' stage list has a hole')
  return stages
end

function M.stages_around(name)
  return stages_of(name, 'before'), stages_of(name, 'after')
end

function M.expected_stages(phase)
  local stages = {}
  for _, spec in ipairs(M.expected_cases(phase)) do
    for _, stage in ipairs(stages_of(spec[1], 'before')) do stages[#stages + 1] = stage end
    for _, stage in ipairs(stages_of(spec[1], 'after')) do stages[#stages + 1] = stage end
  end
  return stages
end

local function expected_stages(phase)
  return M.expected_stages(phase)
end

local function declared_intent_count(phase)
  local count = 0
  for _, spec in ipairs(M.expected_cases(phase)) do
    if spec[2] then count = count + (type(spec[3])=='table' and #spec[3] or spec[3] == 'twice' and 2 or 1) end
  end
  return count
end

local function check_history(startup, history, phase)
  local stages = expected_stages(phase)
  assert(#history == #stages, 'control history stage count mismatch')
  local source_epoch = integer(startup.source_disclosure_epoch, 'startup source epoch')
  local target_epoch = integer(startup.target_disclosure_epoch, 'startup target epoch')
  local published_base = integer(startup.published_sequence, 'startup published sequence')
  assert(published_base >= 0, 'startup published sequence is negative')
  local preceding_communication_frames = 0
  for index, stage in ipairs(stages) do
    local ack = history[index]
    assert(ack.schema == fixture.schemas.ack, 'control history schema mismatch')
    assert(ack.run_id == startup.run_id, 'control history run id mismatch')
    assert(ack.authority_ack == false, 'control history claimed authority')
    assert(ack.accepted == true, 'control history stage was not accepted')
    assert(ack.stage == stage, 'control history stage order mismatch')
    assert(integer(ack.sequence, 'control sequence') == index, 'control history sequence is not contiguous')
    -- Control ordinal and IPC sequence are different clocks. Each communication stage
    -- publishes one Snapshot followed by one separate CommunicationPresentation.
    assert(integer(ack.published_sequence, 'control published sequence') == published_base + index + preceding_communication_frames,
      'control history published sequence does not match actual IPC envelopes')
    if stage == 'communication_ready' or stage == 'communication_retire' or stage == 'communication_received' or stage == 'communication_signals' or stage == 'communication_signals_expired' or stage == 'communication_hud' then
      preceding_communication_frames = preceding_communication_frames + 1
    end
  end
  local replaced_epoch = source_epoch
  if phase == 'full' or phase == 'foundation' then
  local death_hold, baseline_death, death_cast, baseline_cast, replacement, removal, baseline, disclosure, replaced, final = table.unpack(history)
  assert(integer(death_hold.source_render_id, 'source death source') == 11, 'source death changed the owned render id')
  assert(integer(death_hold.source_disclosure_epoch, 'source death epoch') == source_epoch, 'source death bumped the owned epoch')
  assert(integer(death_hold.target_render_id, 'source death target') == 21, 'source death changed the target')
  assert(integer(death_hold.target_disclosure_epoch, 'source death target epoch') == target_epoch, 'source death changed the target epoch')
  assert(integer(baseline_death.source_render_id, 'post-death baseline source') == 11 and integer(baseline_death.target_render_id, 'post-death baseline target') == 21, 'post-death baseline did not restore 11 and 21')
  assert(integer(baseline_death.source_disclosure_epoch, 'post-death floor') == source_epoch, 'post-death baseline moved the epoch floor')
  assert(integer(baseline_death.target_disclosure_epoch, 'post-death target epoch') == source_epoch, 'post-death baseline did not refresh the target at the current floor')
  assert(integer(death_cast.source_render_id, 'indicator death source') == 11 and integer(death_cast.source_disclosure_epoch, 'indicator death epoch') == source_epoch, 'indicator source death changed the owned identity')
  assert(integer(death_cast.target_render_id, 'indicator death target') == 21 and integer(death_cast.target_disclosure_epoch, 'indicator death target epoch') == source_epoch, 'indicator source death changed the refreshed target')
  assert(integer(baseline_cast.source_disclosure_epoch, 'post-indicator floor') == source_epoch and integer(baseline_cast.target_disclosure_epoch, 'post-indicator target epoch') == source_epoch, 'post-indicator baseline moved the floor')
  assert(integer(replacement.source_render_id, 'replacement source') == 12, 'source replacement did not publish hero 12')
  assert(integer(replacement.source_disclosure_epoch, 'replacement source epoch') > source_epoch, 'source replacement did not advance the epoch')
  assert(integer(replacement.target_render_id, 'replacement target') == 21, 'source replacement changed the target')
  assert(integer(replacement.target_disclosure_epoch, 'replacement target epoch') == source_epoch, 'source replacement did not keep the refreshed target epoch')
  assert(removal.source_render_id == json.null and removal.source_disclosure_epoch == json.null, 'source removal did not publish a null source')
  assert(integer(removal.target_render_id, 'removal target') == 21, 'source removal changed the target')
  assert(integer(removal.target_disclosure_epoch, 'removal target epoch') == source_epoch, 'source removal changed the refreshed target epoch')
  local floor = integer(replacement.source_disclosure_epoch, 'replacement floor')
  assert(integer(baseline.source_render_id, 'baseline source') == 11 and integer(baseline.target_render_id, 'baseline target') == 21, 'baseline did not restore 11 and 21')
  assert(integer(baseline.source_disclosure_epoch, 'baseline source epoch') == floor, 'baseline moved the epoch floor')
  assert(integer(baseline.target_disclosure_epoch, 'baseline target epoch') == floor, 'baseline did not refresh the target at the current floor')
  assert(integer(disclosure.source_render_id, 'disclosure source') == 11 and integer(disclosure.target_render_id, 'disclosure target') == 21, 'disclosure changed a render id')
  local disclosed = integer(disclosure.source_disclosure_epoch, 'disclosure epoch')
  assert(disclosed > floor and integer(disclosure.target_disclosure_epoch, 'disclosure target epoch') == disclosed, 'disclosure did not advance both epochs together')
  assert(integer(replaced.source_render_id, 'replace source') == 11, 'replace_target changed the source id')
  assert(integer(replaced.source_disclosure_epoch, 'replace source epoch') == disclosed, 'replace_target changed the source epoch')
  replaced_epoch = integer(replaced.target_disclosure_epoch, 'replace target epoch')
  assert(integer(replaced.target_render_id, 'replace target') == 22 and replaced_epoch > disclosed, 'replace_target did not publish target 22 at a newer epoch')
  assert(integer(final.source_render_id, 'final source') == 11 and integer(final.target_render_id, 'final target') == 21, 'final baseline did not restore 11 and 21')
  assert(integer(final.source_disclosure_epoch, 'final source epoch') == replaced_epoch, 'final baseline did not use the current floor')
  assert(integer(final.target_disclosure_epoch, 'final target epoch') == replaced_epoch, 'final baseline did not refresh the target')
  end
  -- Owner stages disclose hero, mana, cooldown, or target span. They do not bump epochs.
  local keep_identity = {
    owner_support_team_cards = true, owner_support_team_cards_dead = true,
    input_point = true, input_keyboard = true, input_locked = true,
    combat_defaults = true, combat_cursor_projection = true, combat_idle_off_projection = true,
    owner_cast_ready = true, owner_upgrade_point = true, owner_short_mana = true,
    owner_cooldown = true, owner_hide_target = true, owner_far_unit = true,
    owner_apprentice = true, owner_point_ready = true,
    owner_support_ready = true, owner_support_enemy = true, owner_support_conceal = true,
    owner_support_dead = true, owner_support_neutral = true, source_death = true,
    death = true, hud_cooldown = true,
    owner_summoner_ready = true, owner_summoner_cooldown = true,
    owner_summoner_unavailable = true, owner_summoner_malformed = true,
    owner_summoner_unsupported = true,
    owner_recall_ready = true, owner_recall_channel = true, owner_recall_unavailable = true,
    owner_item_ready = true, owner_item_empty = true, owner_item_cooldown = true,
    owner_item_far = true, owner_item_replaced = true,
    mode_tower_defense = true, mode_td_defeat = true, mode_td_no_hero = true,
    mode_moba_dead = true, mode_unavailable = true,
    owner_shop_ready = true,
  }
  -- These projections keep the current actor token. They do not move the view floor or the target.
  local ward_preserve = {
    controlled_owner_ward_projection = true,
    controlled_owner_ward_dead = true,
    controlled_owner_ward_forwarded = true,
    controlled_owner_ward_uncertain_mapped = true,
    controlled_owner_ward_settled_mapped = true,
    controlled_owner_ward_rejected_mapped = true,
    controlled_owner_shop_current = true,
    controlled_owner_shop_admitted = true,
    controlled_owner_shop_uncertain = true,
    controlled_owner_shop_settled = true,
    controlled_owner_shop_rejected = true,
    controlled_owner_shop_stale = true,
    controlled_owner_shop_full = true,
  }
  local floor = replaced_epoch
  local source_id = 11
  local actor_generation = floor
  for index = (phase == 'full' or phase == 'foundation') and 11 or 1, #history do
    local ack = history[index]
    local stage = stages[index]
    if stage == 'baseline' or stage == 'communication_ready' or stage == 'communication_received' or stage == 'communication_signals' or stage == 'communication_signals_expired' or stage == 'communication_hud' then
      assert(integer(ack.source_render_id, 'tail baseline source') == 11, 'ability baseline did not restore source 11')
      assert(integer(ack.target_render_id, 'tail baseline target') == 21, 'ability baseline did not restore target 21')
      assert(integer(ack.source_disclosure_epoch, 'tail baseline source epoch') == floor, 'ability baseline moved the epoch floor')
      assert(integer(ack.target_disclosure_epoch, 'tail baseline target epoch') == floor, 'ability baseline did not refresh the target at the current floor')
      source_id = 11
      actor_generation = floor
    elseif stage == 'communication_retire' then
      local advanced = integer(ack.source_disclosure_epoch, 'communication retired epoch')
      assert(advanced > floor and ack.source_render_id == 11 and ack.target_render_id == 21
        and ack.target_disclosure_epoch == advanced, 'communication retirement scope mismatch')
      floor, source_id, actor_generation = advanced, 11, advanced
    elseif stage == 'source_replacement' then
      assert(integer(ack.source_render_id, 'tail replacement source') == 12, 'ability source replacement did not publish hero 12')
      local advanced = integer(ack.source_disclosure_epoch, 'tail replacement epoch')
      assert(advanced > floor, 'ability source replacement did not advance the epoch')
      assert(integer(ack.target_render_id, 'tail replacement target') == 21, 'ability source replacement changed the target')
      assert(integer(ack.target_disclosure_epoch, 'tail replacement target epoch') == floor, 'ability source replacement did not keep the target epoch')
      floor = advanced
      source_id = 12
      actor_generation = floor
    elseif stage == 'source_removal' then
      assert(ack.source_render_id == json.null and ack.source_disclosure_epoch == json.null, 'ability source removal did not publish a null source')
      assert(integer(ack.target_render_id, 'tail removal target') == 21, 'ability source removal changed the target')
      assert(integer(ack.target_disclosure_epoch, 'tail removal target epoch') == floor, 'ability source removal changed the target epoch')
      source_id = nil
      actor_generation = nil
    elseif stage == 'disclosure_epoch' or stage == 'mode_td_epoch' or stage == 'communication_retire' then
      assert(integer(ack.source_render_id, 'tail disclosure source') == 11, 'ability disclosure changed the source id')
      assert(integer(ack.target_render_id, 'tail disclosure target') == 21, 'ability disclosure changed the target id')
      local disclosed = integer(ack.source_disclosure_epoch, 'tail disclosure epoch')
      assert(disclosed > floor and integer(ack.target_disclosure_epoch, 'tail disclosure target epoch') == disclosed, 'ability disclosure did not advance both epochs together')
      floor = disclosed
      source_id = 11
      actor_generation = floor
    elseif stage == 'controlled_owner_ward_same_epoch_source' then
      assert(source_id == 11, 'same-view replacement did not start from the living canonical source')
      assert(integer(ack.source_render_id, 'same-view source') == 12, 'same-view replacement did not publish hero 12')
      local generation = integer(ack.source_disclosure_epoch, 'same-view actor generation')
      assert(generation ~= floor, 'same-view replacement used the view floor as the actor generation')
      assert(generation == floor + 64, 'same-view replacement actor generation mismatch')
      assert(integer(ack.target_render_id, 'same-view target') == 21, 'same-view replacement changed the target')
      assert(integer(ack.target_disclosure_epoch, 'same-view target epoch') == floor, 'same-view replacement changed the target epoch')
      source_id = 12
      actor_generation = generation
    elseif stage == 'controlled_owner_shop_epoch' then
      assert(source_id == 12, 'shop epoch advance did not keep hero 12')
      assert(integer(ack.source_render_id, 'shop epoch source') == 12, 'shop epoch advance changed the source id')
      assert(integer(ack.target_render_id, 'shop epoch target') == 21, 'shop epoch advance changed the target')
      local disclosed = integer(ack.source_disclosure_epoch, 'shop epoch source epoch')
      assert(disclosed > floor and integer(ack.target_disclosure_epoch, 'shop epoch target epoch') == disclosed, 'shop epoch advance did not move both epochs together')
      floor = disclosed
      source_id = 12
      actor_generation = disclosed
    elseif ward_preserve[stage] then
      assert(source_id ~= nil, stage .. ' ran without a current source')
      assert(integer(ack.source_render_id, stage .. ' source') == source_id, stage .. ' changed the current source')
      assert(integer(ack.source_disclosure_epoch, stage .. ' actor generation') == actor_generation, stage .. ' changed the actor generation')
      assert(integer(ack.target_render_id, stage .. ' target') == 21, stage .. ' changed the target')
      assert(integer(ack.target_disclosure_epoch, stage .. ' target epoch') == floor, stage .. ' changed the view floor or the target epoch')
    else
      assert(keep_identity[stage], 'ability tail stage is not classified: ' .. tostring(stage))
      assert(integer(ack.source_render_id, stage .. ' source') == 11, stage .. ' changed the owned render id')
      assert(integer(ack.source_disclosure_epoch, stage .. ' source epoch') == floor, stage .. ' bumped the owned epoch')
      assert(integer(ack.target_render_id, stage .. ' target') == 21, stage .. ' changed the target')
      assert(integer(ack.target_disclosure_epoch, stage .. ' target epoch') == floor, stage .. ' changed the target epoch')
    end
  end
end

-- Lua patterns have no alternation. Capture one token, then whitelist the three legal policies.
function M.parse_item_policy(text)
  local slot_text, policy, catalog_text, item_id = tostring(text):match('^(%d+):(%a+):(%d+):([%w_]+)$')
  if not slot_text then
    return nil, 'ItemUse case policy is missing'
  end
  if policy ~= 'self' and policy ~= 'point' and policy ~= 'unit' then
    return nil, 'ItemUse case policy is unknown'
  end
  return slot_text, policy, catalog_text, item_id
end

-- Catalog id plus the authored item id. The wire itself has catalog_id only.
function M.parse_shop_buy_policy(text)
  local catalog_text, item_id = tostring(text):match('^(%d+):([%w_]+)$')
  if not catalog_text then
    return nil, 'ItemBuy case policy is missing'
  end
  return catalog_text, item_id
end

local FIXED32_MIN = -2147483648
local function positive_u64_string(value, allow_max)
  assert(type(value)=='string' and value:match('^[1-9]%d*$') and #value<=20, 'owner control identity must be canonical positive decimal')
  local limit=allow_max and '18446744073709551615' or '18446744073709551614'
  assert(#value<20 or value<=limit, 'owner control identity overflow/reserved')
end

function M.validate_owner_control_wire_intent(intent)
  assert(type(intent)=='table', 'owner control intent is missing')
  local allowed={kind=true,schema_version=true}
  assert(integer(intent.schema_version,'owner control schema')==1, 'owner control schema mismatch')
  if intent.kind=='InputScheme' then
    allowed.scheme=true
    local scheme=integer(intent.scheme,'input scheme');assert(scheme==0 or scheme==1,'input scheme outside 0..1')
  else
    assert(intent.kind=='Direction','owner control kind mismatch')
    for _,key in ipairs({'source_render_id','press_id','phase','x','y'})do allowed[key]=true end
    positive_u64_string(intent.source_render_id,true);positive_u64_string(intent.press_id,false)
    local phase=integer(intent.phase,'direction phase');assert(phase>=1 and phase<=3,'direction phase outside 1..3')
    local x,y=integer(intent.x,'direction x'),integer(intent.y,'direction y')
    assert(x>=-32767 and x<=32767 and y>=-32767 and y<=32767,'direction axes overflow')
    assert((x==0 and y==0)==(phase==3),'direction zero axes/phase mismatch')
  end
  for key in pairs(intent)do assert(allowed[key],'owner control aliased '..tostring(key))end
  return true
end

local FIXED32_MAX = 2147483647

local function fixed32(value, label)
  local raw = integer(value, label)
  assert(raw >= FIXED32_MIN and raw <= FIXED32_MAX, label .. ' is outside fixed32')
  return raw
end

local function check_intent(line, startup, kind, view, target_id, target_epoch, unit_who, published, source_id, actor_generation)
  assert(line.schema == 'omoba-renderer-fixture-intent/1', 'intent schema mismatch')
  assert(line.run_id == startup.run_id, 'intent run id mismatch')
  assert(line.authority_ack == false, 'intent claimed authority')
  assert(integer(line.player_id, 'intent player') == 1, 'intent player is not 1')
  assert(line.content_hash == startup.content_hash, 'intent content hash mismatch')
  assert(line.moba_map_id == startup.moba_map_id, 'intent map mismatch')
  assert(type(line.intent) == 'table' and line.intent.kind == kind, 'unexpected wire intent kind')
  local ability = kind == 'AbilityCast' or kind == 'AbilityUpgrade'
  if ability then
    assert(line.intent.queued == nil, 'ability wire intent carried queued')
    local slot = integer(line.intent.ability_index, kind .. ' slot')
    assert(slot >= 0 and slot <= 3, kind .. ' slot is outside QWER')
    assert(integer(line.intent.catalog_id, kind .. ' catalog') > 0, kind .. ' catalog is not positive')
    assert(type(line.intent.ability_id) == 'string' and line.intent.ability_id ~= '', kind .. ' ability id is empty')
  elseif kind == 'SummonerCast' then
    assert(line.intent.queued == nil, 'summoner wire intent carried queued')
    local slot = integer(line.intent.slot, 'SummonerCast slot')
    assert(slot == 0 or slot == 1, 'SummonerCast slot is outside D/F')
    assert(line.intent.ability_index == nil and line.intent.ability_id == nil, 'SummonerCast aliased a QWER ability')
    assert(integer(line.intent.catalog_id, 'SummonerCast catalog') > 0, 'SummonerCast catalog is not positive')
    assert(type(line.intent.spell_id) == 'string' and line.intent.spell_id ~= '', 'SummonerCast spell id is empty')
  elseif kind == 'Stop' or kind == 'Recall' or kind == 'ItemUse' or kind == 'ItemBuy' or kind == 'ItemSell' then
    assert(line.intent.queued == nil, kind .. ' carried queued')
  elseif kind == 'Communication' or kind == 'CombatPreferences' or kind == 'InputScheme' or kind == 'Direction' then
    assert(line.intent.queued == nil, kind .. ' carried queued')
  elseif kind == 'WardPlacement' then
    assert(line.intent.queued == nil, 'WardPlacement carried queued')
  elseif kind == 'HoldPosition' then
    assert(line.intent.queued == false, 'HoldPosition was queued or omitted queued')
  else
    assert(line.intent.queued == false, 'wire intent was queued')
  end
  assert(integer(line.disclosure_epoch, 'intent view epoch') == view, 'wire intent view epoch mismatch')
  assert(integer(line.entity_epoch_floor, 'intent view floor') == view, 'wire intent confused the view floor')
  assert(integer(line.published_sequence, 'intent published sequence') == published, 'wire intent published sequence mismatch')
  assert(integer(line.target_render_id, 'intent record target') == target_id, 'wire intent record target mismatch')
  assert(integer(line.target_disclosure_epoch, 'intent record target epoch') == target_epoch, 'wire intent record target epoch mismatch')
  if kind == 'Communication' then
    local intent = line.intent
    assert(integer(line.ipc_sequence, 'communication IPC sequence') >= published, 'communication IPC precedes its snapshot')
    assert(integer(intent.source_generation, 'communication source') > 0 and integer(intent.presentation_epoch, 'communication presentation') > 0, 'communication scope invalid')
    for _, field in ipairs({'x_raw','y_raw','target_kind','ability_index','ability_id','catalog_id','spell_id','slot','item_slot','item_id','explicit_x_raw','explicit_y_raw','view_epoch'}) do
      assert(intent[field] == nil, 'Communication aliased ' .. field)
    end
    local hud_kind,hud_index,hud_player = tostring(unit_who):match('^status:(%d):(%d):(%d+)$')
    local objective = tostring(unit_who):match('^objective:([%w_]+)$')
    for _, field in ipairs({'target_render_id','target_disclosure_epoch'}) do
      assert(integer(intent[field], 'Communication ' .. field) == 0, 'Communication carried ' .. field)
    end
    assert(integer(intent.hud_kind,'HUD kind') == tonumber(hud_kind or 0) and integer(intent.hud_index,'HUD index') == tonumber(hud_index or 0)
      and integer(intent.hud_player_id,'HUD player') == tonumber(hud_player or 0), 'Communication HUD selector differs')
    local ping = tostring(unit_who):match('^ping:(%d+)$')
    if ping then
      local semantic = integer(tonumber(ping), 'ping policy')
      assert(semantic >= 1 and semantic <= 10 and intent.has_point == true
        and integer(intent.semantic, 'ping semantic') == semantic, 'Communication ping semantic differs')
      for _, field in ipairs({'point_x_raw','point_y_raw'}) do
        local value = integer(intent[field], 'ping ' .. field)
        assert(value >= -2147483648 and value <= 2147483647, 'Communication ping point outside fixed32')
      end
    else
      assert(intent.has_point == false, 'Communication carried point')
      for _, field in ipairs({'semantic','point_x_raw','point_y_raw'}) do
        assert(integer(intent[field], 'Communication ' .. field) == 0, 'Communication carried ' .. field)
      end
    end
    local mute = tostring(unit_who):match('^mute:([01])$')
    assert(type(intent.muted) == 'boolean', 'Communication muted must be boolean')
    assert(integer(intent.mute_sender_id, 'mute sender') == (mute and 2 or 0), 'Communication mute sender differs')
    assert(intent.muted == (mute == '1'), 'Communication desired mute differs')
    assert(intent.objective_id == (objective or ''), 'Communication carried or malformed objective id')
    local chat = tostring(unit_who):match('^chat:([12])$') or tostring(unit_who):match('^chat%-text:([12])$')
    local interaction = tostring(unit_who):match('^interaction:([1-5])$')
    local emote = tostring(unit_who):match('^emote:(10[1-8])$')
    if hud_kind then
      assert(intent.communication_kind == 6 and intent.chat_channel == 0 and intent.text == '' and intent.content_id == 0, 'Communication status shape differs')
    elseif objective then
      assert(intent.communication_kind == 3 and intent.chat_channel == 0 and intent.text == '' and intent.content_id == 0, 'Communication objective shape differs')
    elseif ping then
      assert(intent.communication_kind == 1 and intent.chat_channel == 0 and intent.text == '' and intent.content_id == 0, 'Communication ping shape differs')
    elseif emote then
      assert(intent.communication_kind == 5 and intent.content_id == tonumber(emote) and intent.chat_channel == 0 and intent.text == '', 'Communication emote mismatch')
    elseif mute then
      assert(intent.communication_kind == 4 and intent.chat_channel == 0 and intent.text == '' and intent.content_id == 0, 'Communication mute shape differs')
    elseif chat then
      assert(intent.communication_kind == 2 and intent.chat_channel == tonumber(chat) and intent.text == (unit_who == 'chat-text:1' and 'QWER A 集合' or '集合')
        and intent.content_id == 0, 'Communication Chinese chat mismatch')
    else
      assert(interaction and intent.communication_kind == 5 and intent.content_id == tonumber(interaction)
        and intent.chat_channel == 0 and intent.text == '', 'Communication interaction mismatch')
    end
  elseif kind == 'AttackTarget' then
    assert(integer(line.intent.target_render_id, 'attack target') == target_id, 'AttackTarget render id mismatch')
  elseif kind == 'AbilityUpgrade' then
    local card_slot=tostring(unit_who):match('^card:([0-3])$')
    if card_slot then assert(line.intent.ability_index==tonumber(card_slot),'card upgrade slot mismatch') end
    assert(line.intent.x_raw == nil and line.intent.y_raw == nil, 'ability upgrade carried a point')
    assert(line.intent.target_kind == nil and line.intent.target_render_id == nil, 'ability upgrade carried a target')
  elseif kind == 'AbilityCast' then
    integer(line.intent.x_raw, 'AbilityCast x_raw')
    integer(line.intent.y_raw, 'AbilityCast y_raw')
    local shape = line.intent.target_kind
    local card_slot,card_shape=tostring(unit_who):match('^card:([0-3]):(%a+)$')
    if card_slot then assert(line.intent.ability_index==tonumber(card_slot) and shape==card_shape,'card ability shape/slot mismatch') end
    assert(shape == 'none' or shape == 'unit' or shape == 'point', 'AbilityCast target kind is not none, unit, or point')
    local cast_target = integer(line.intent.target_render_id, 'AbilityCast target')
    if shape == 'unit' then
      local expected = target_id
      if unit_who == 'source' then
        expected = integer(startup.source_render_id, 'startup source')
      elseif unit_who ~= nil and not card_slot then
        error('unit AbilityCast target policy is not envelope or source')
      end
      local detail = unit_who == 'source' and 'unit AbilityCast did not use the disclosed source at the origin point' or 'unit AbilityCast did not use the disclosed target at the origin point'
      assert(cast_target == expected and line.intent.x_raw == 0 and line.intent.y_raw == 0, detail)
    elseif shape == 'none' then
      assert(cast_target == 0 and line.intent.x_raw == 0 and line.intent.y_raw == 0, 'none AbilityCast carried a target or a point')
    else
      assert(cast_target == 0, 'point AbilityCast carried a unit target')
    end
  elseif kind == 'SummonerCast' then
    local shape = line.intent.target_kind
    assert(shape == 'none' or shape == 'point', 'SummonerCast target kind is not none or point')
    assert(line.intent.target_render_id == nil, 'SummonerCast carried a unit target')
    if shape == 'none' then
      assert(line.intent.x_raw == json.null and line.intent.y_raw == json.null, 'self SummonerCast carried a point')
    else
      integer(line.intent.x_raw, 'SummonerCast x_raw')
      integer(line.intent.y_raw, 'SummonerCast y_raw')
    end
  elseif kind == 'Stop' or kind == 'Recall' or kind == 'HoldPosition' then
    assert(line.intent.x_raw == nil and line.intent.y_raw == nil, kind .. ' carried a point')
    assert(line.intent.item_slot == nil and line.intent.explicit_x_raw == nil, kind .. ' carried an item target')
  elseif kind == 'ItemUse' then
    local slot_text, policy, catalog_text, item_id = M.parse_item_policy(unit_who)
    assert(slot_text, policy)
    local slot = integer(tonumber(slot_text), 'item slot')
    local catalog = integer(tonumber(catalog_text), 'item catalog')
    assert(integer(line.intent.item_slot, 'item_slot') == slot, 'ItemUse slot mismatch')
    assert(integer(line.intent.catalog_id, 'item catalog') == catalog, 'ItemUse catalog mismatch')
    assert(line.intent.item_id == item_id, 'ItemUse item id mismatch')
    assert(line.intent.ability_index == nil and line.intent.ability_id == nil and line.intent.spell_id == nil, 'ItemUse aliased an ability or summoner')
    integer(line.intent.x_raw, 'ItemUse x_raw')
    integer(line.intent.y_raw, 'ItemUse y_raw')
    local cast_target = integer(line.intent.target_render_id, 'ItemUse target')
    if policy == 'point' then
      assert(cast_target == 0, 'point ItemUse carried a unit')
      assert(integer(line.intent.explicit_x_raw, 'ItemUse explicit x') == line.intent.x_raw, 'point ItemUse explicit x does not match')
      assert(integer(line.intent.explicit_y_raw, 'ItemUse explicit y') == line.intent.y_raw, 'point ItemUse explicit y does not match')
    elseif policy == 'unit' then
      assert(cast_target == target_id and line.intent.x_raw == 0 and line.intent.y_raw == 0, 'unit ItemUse did not use the disclosed target at the origin')
      assert(line.intent.explicit_x_raw == json.null and line.intent.explicit_y_raw == json.null, 'unit ItemUse carried an explicit point')
    else
      assert(cast_target == 0 and line.intent.x_raw == 0 and line.intent.y_raw == 0, 'self ItemUse carried a target or a point')
      assert(line.intent.explicit_x_raw == json.null and line.intent.explicit_y_raw == json.null, 'self ItemUse carried an explicit point')
    end
  elseif kind == 'ItemBuy' then
    local catalog_text, item_id = M.parse_shop_buy_policy(unit_who)
    assert(catalog_text, item_id)
    local catalog = integer(tonumber(catalog_text), 'ItemBuy policy catalog')
    assert(catalog == 4 and item_id == 'moba_greatsword', 'ItemBuy policy is not the authored greatsword identity')
    assert(integer(line.intent.catalog_id, 'ItemBuy catalog') == catalog, 'ItemBuy catalog mismatch')
    assert(line.intent.item_id == nil and line.intent.item_slot == nil, 'ItemBuy carried an item id or a slot')
    assert(line.intent.x_raw == nil and line.intent.y_raw == nil, 'ItemBuy carried a point')
    assert(line.intent.ability_index == nil and line.intent.ability_id == nil and line.intent.spell_id == nil, 'ItemBuy aliased an ability')
  elseif kind == 'ItemSell' then
    local slot = integer(tonumber(unit_who), 'ItemSell slot')
    assert(slot == 0, 'ItemSell slot must be 0')
    assert(integer(line.intent.item_slot, 'ItemSell slot') == 0, 'ItemSell slot mismatch')
    assert(line.intent.catalog_id == nil and line.intent.item_id == nil, 'ItemSell carried a catalog or an item id')
    assert(line.intent.x_raw == nil and line.intent.y_raw == nil, 'ItemSell carried a point')
    assert(line.intent.ability_index == nil and line.intent.ability_id == nil and line.intent.spell_id == nil, 'ItemSell aliased an ability')
  elseif kind == 'WardPlacement' then
    assert(line.intent.slot ~= nil and line.intent.x_raw ~= nil and line.intent.y_raw ~= nil, 'WardPlacement is missing slot or point')
    local function ward_integer(value)
      if value == json.null or math.type(value) ~= 'integer' then error('WardPlacement field is malformed', 0) end
      return value
    end
    local slot = ward_integer(line.intent.slot)
    assert(slot == 0, 'WardPlacement slot must be 0')
    fixed32(ward_integer(line.intent.x_raw), 'WardPlacement x_raw')
    fixed32(ward_integer(line.intent.y_raw), 'WardPlacement y_raw')
    local forbidden = {
      'target_render_id', 'target_kind', 'ability_index', 'ability_id', 'catalog_id', 'spell_id',
      'item_slot', 'item_id', 'explicit_x_raw', 'explicit_y_raw',
    }
    for _, field in ipairs(forbidden) do
      assert(line.intent[field] == nil, 'WardPlacement aliased ' .. field)
    end
    assert(source_id ~= nil, 'WardPlacement has no current source')
    assert(integer(line.source_render_id, 'WardPlacement source') == source_id, 'WardPlacement source identity mismatch')
    assert(integer(line.source_disclosure_epoch, 'WardPlacement actor generation') == actor_generation, 'WardPlacement confused actor generation with the view floor')
  elseif kind == 'InputScheme' or kind == 'Direction' then
    M.validate_owner_control_wire_intent(line.intent)
    if kind == 'InputScheme' then
      assert(line.intent.scheme == tonumber(unit_who), 'owner mode wire differs from requested scheme')
    else
      assert(type(unit_who)=='table' and #unit_who==4, 'direction sequence policy missing')
      assert(line.intent.phase==unit_who[1] and line.intent.x==unit_who[2] and line.intent.y==unit_who[3]
        and line.intent.press_id==unit_who[4], 'direction phase/axes/press sequence differs')
      assert(line.intent.source_render_id==tostring(source_id), 'direction source differs from published owner')
    end
  elseif kind == 'CombatPreferences' then
    assert(integer(line.intent.schema_version, 'combat schema') == 1, 'combat schema mismatch')
    assert(integer(line.intent.acquisition, 'combat acquisition') == 1, 'combat cursor preference mismatch')
    assert(line.intent.idle_auto_attack == (unit_who == 'cursor'), 'combat idle preference mismatch')
    for _,field in ipairs({'queued','x_raw','y_raw','slot','ability_index','ability_id','catalog_id','item_slot','target_render_id'})do
      assert(line.intent[field] == nil, 'combat preference aliased ' .. field)
    end
  else
    integer(line.intent.x_raw, kind .. ' x_raw')
    integer(line.intent.y_raw, kind .. ' y_raw')
  end
end

function M.verify_artifacts(bundle)
  assert(type(bundle) == 'table', 'artifact bundle is missing')
  local phase = bundle.phase or 'full'
  local selected_cases = M.expected_cases(phase)
  local startup, native = bundle.startup, bundle.native
  assert(startup.schema == fixture.schemas.startup, 'startup schema mismatch')
  assert(startup.authority == false and startup.critical_input_result == 'never', 'startup claimed authority or a critical result')
  assert(startup.moba_map_id == 'three_lane_training', 'startup map is not three_lane_training')
  assert(integer(startup.source_render_id, 'startup source') == 11, 'startup source is not 11')
  assert(integer(startup.target_render_id, 'startup target') == 21, 'startup target is not 21')
  assert(type(startup.content_hash) == 'string' and startup.content_hash ~= '', 'startup content hash is empty')
  assert(type(startup.content_catalog_identity_hash) == 'string' and startup.content_catalog_identity_hash ~= '', 'startup catalog identity is empty')
  assert(type(startup.moba_map_catalog_hash) == 'string' and startup.moba_map_catalog_hash ~= '', 'startup map catalog is empty')
  assert(native.schema == 'omoba-owned-viewport-ipc-native/1', 'native IPC schema mismatch')
  assert(native.success == true, 'native IPC report failed: ' .. tostring(native.failure))
  assert((native.phase or 'full') == phase, 'native phase does not match requested inventory')
  if phase ~= 'full' or native.full_case_inventory ~= nil then
    assert(native.full_case_inventory == (phase == 'full'), 'phase cannot claim a full case inventory')
  end
  assert(native.authority_ack == false, 'native IPC report claimed authority')
  assert(native.failure == '', 'native IPC success still carries a failure')
  assert(native.run_id == startup.run_id, 'native report run id mismatch')
  assert(native.content_hash == startup.content_hash, 'native report content hash does not echo startup')
  assert(native.content_catalog_identity_hash == startup.content_catalog_identity_hash, 'native report catalog identity does not echo startup')
  assert(native.moba_map_id == startup.moba_map_id, 'native report map does not echo startup')
  assert(native.moba_map_catalog_hash == startup.moba_map_catalog_hash, 'native report map catalog does not echo startup')
  assert(native.catalog_compatible == true, 'native catalog was not compatible')
  assert(integer(native.configured_player_id, 'configured player') == 1, 'configured player is not 1')
  assert(native.address == startup.address, 'native report address does not echo startup')
  assert(integer(native.source_render_id, 'native source') == 11 and integer(native.target_render_id, 'native target') == 21, 'native report render ids drifted')
  assert(integer(native.source_disclosure_epoch, 'native source epoch') == integer(startup.source_disclosure_epoch, 'startup source epoch'), 'native source epoch does not echo startup')
  assert(integer(native.target_disclosure_epoch, 'native target epoch') == integer(startup.target_disclosure_epoch, 'startup target epoch'), 'native target epoch does not echo startup')
  assert(type(native.generated_content_hash) == 'string' and native.generated_content_hash ~= '', 'generated content hash is empty')
  assert(native.compiled_content_hash == native.generated_content_hash, 'compiled template hash does not match the bridge generated hash')
  assert(type(native.class_surface_signature) == 'string' and native.class_surface_signature ~= '', 'class surface signature is empty')
  assert(native.compiled_class_surface == native.class_surface_signature, 'compiled class surface does not match the bridge class surface')
  assert(native.catalog_data_hash == startup.content_hash, 'catalog data hash does not match startup content hash')
  assert(integer(native.lockstep_step_fps, 'lockstep step fps') > 0, 'lockstep step fps is not a positive admitted rate')
  check_history(startup, bundle.history, phase)
  assert(same_json(bundle.history[#bundle.history], bundle.ack), 'last control history line does not match the publisher ack')
  local intents = bundle.intents
  assert(type(intents) == 'table', 'intent log is missing')
  local ordinal, cursor = 0, 1
  local view = integer(startup.source_disclosure_epoch, 'startup source epoch')
  local target_epoch = integer(startup.target_disclosure_epoch, 'startup target epoch')
  local target_id = 21
  local source_id = integer(startup.source_render_id, 'startup source')
  local actor_generation = view
  local published = integer(startup.published_sequence, 'startup published sequence')
  local stage_index = 0
  local function apply(stage)
    stage_index = stage_index + 1
    local ack = bundle.history[stage_index]
    assert(type(ack) == 'table' and ack.stage == stage, 'intent walk reached a different control stage')
    if stage == 'source_replacement' then
      view = integer(ack.source_disclosure_epoch, 'replacement view')
      source_id = 12
      actor_generation = view
    elseif stage == 'baseline' or stage == 'input_point' or stage == 'input_keyboard' or stage == 'input_locked' or stage == 'communication_ready' or stage == 'communication_received' or stage == 'communication_signals' or stage == 'communication_signals_expired' or stage == 'communication_hud' then
      view = integer(ack.source_disclosure_epoch, 'baseline view')
      target_epoch = integer(ack.target_disclosure_epoch, 'baseline target epoch')
      target_id = 21
      source_id = 11
      actor_generation = view
    elseif stage == 'disclosure_epoch' or stage == 'mode_td_epoch' or stage == 'communication_retire' then
      view = integer(ack.source_disclosure_epoch, 'disclosure view')
      target_epoch = view
      target_id = 21
      source_id = 11
      actor_generation = view
    elseif stage == 'replace_target' then
      view = integer(ack.target_disclosure_epoch, 'replaced view')
      target_epoch = view
      target_id = 22
    elseif stage == 'source_removal' then
      source_id = nil
      actor_generation = nil
    elseif stage == 'controlled_owner_ward_same_epoch_source' then
      source_id = integer(ack.source_render_id, 'same-view source')
      actor_generation = integer(ack.source_disclosure_epoch, 'same-view actor generation')
    elseif stage == 'controlled_owner_shop_epoch' then
      view = integer(ack.source_disclosure_epoch, 'shop epoch view')
      target_epoch = integer(ack.target_disclosure_epoch, 'shop epoch target epoch')
      target_id = 21
      source_id = 12
      actor_generation = view
    end
    published = integer(ack.published_sequence, 'control published sequence')
  end
  assert(type(native.cases) == 'table' and #native.cases == #selected_cases, 'native case inventory mismatch')
  for index, spec in ipairs(selected_cases) do
    local record = native.cases[index]
    assert(record.name == spec[1], 'native case order mismatch')
    for _, stage in ipairs(stages_of(spec[1], 'before')) do apply(stage) end
    if spec[2] then
      local repeats = type(spec[3])=='table' and #spec[3] or spec[3] == 'twice' and 2 or 1
      local policy = spec[3] ~= 'twice' and spec[3] or nil
      local first = ordinal + 1
      assert(type(record.intents) == 'table' and #record.intents == repeats, spec[1] .. ' did not record its raw intent')
      for repeat_index = 1, repeats do
        local line = intents[cursor]
        assert(line, 'missing wire intent for ' .. spec[1])
        assert(integer(line.observation_ordinal, 'observation ordinal') == cursor, 'intent observation ordinal is not contiguous')
        check_intent(line, startup, spec[2], view, target_id, target_epoch, type(spec[3])=='table' and spec[3][repeat_index] or policy, published, source_id, actor_generation)
        local recorded = record.intents[repeat_index]
        assert(type(recorded) == 'string' and recorded ~= '', spec[1] .. ' did not record its raw intent')
        local decoded_ok, decoded = pcall(json.decode, recorded)
        assert(decoded_ok, spec[1] .. ' recorded raw intent is not JSON')
        assert(same_json(decoded, line), spec[1] .. ' recorded raw intent does not match the wire line')
        cursor = cursor + 1
        ordinal = ordinal + 1
      end
      assert(integer(record.first_ordinal, 'case first ordinal') == first, spec[1] .. ' first ordinal mismatch')
      assert(integer(record.last_ordinal, 'case last ordinal') == ordinal, spec[1] .. ' last ordinal mismatch')
    else
      assert(integer(record.first_ordinal, 'empty first ordinal') == 0 and integer(record.last_ordinal, 'empty last ordinal') == 0, spec[1] .. ' recorded an unexpected intent range')
      assert(type(record.intents) == 'table' and #record.intents == 0, spec[1] .. ' recorded a raw intent')
    end
    for _, stage in ipairs(stages_of(spec[1], 'after')) do apply(stage) end
  end
  assert(cursor == #intents + 1, 'intent log contains lines outside the expected case sequence')
  assert(ordinal == declared_intent_count(phase) and stage_index == #bundle.history, 'wire intents or control stages do not match the declared cases')
  assert(stage_index == #expected_stages(phase), 'control walk did not apply every declared stage')
  local ue = contract.native_json(bundle.ue_index, {TEST})
  assert(ue.count == 1 and ue.failed == 0 and ue.not_run == 0, 'Unreal automation index did not accept the IPC test')
  if bundle.cleanup then
    assert(bundle.cleanup.schema == fixture.schemas.cleanup, 'cleanup schema mismatch')
    assert(bundle.cleanup.run_id == startup.run_id, 'cleanup run id mismatch')
    assert(bundle.cleanup.authority == false, 'cleanup claimed authority')
    assert(bundle.cleanup.listener_released == true, 'listener was not released')
  end
  return true
end

function M.open_dependent_artifacts(native, loader)
  if type(native) ~= 'table' or native.success ~= true then
    local failure = type(native) == 'table' and native.failure or nil
    if type(failure) ~= 'string' or failure == '' then failure = 'native IPC report failed' end
    error(failure, 0)
  end
  return loader()
end

local function plain(text)
  return tostring(text or ''):gsub('[%z\1-\31]', ' ')
end

local function remaining_ms(started, deadline)
  local now = time.monotonic_ms()
  assert(now >= started, 'workflow clock moved backwards')
  local left = deadline - now
  if left < 0 then return 0 end
  return left
end

local function atomic_stop(dest)
  if path.is_file(dest) then return end
  local tmp = dest .. '.partial'
  local file = assert(io.open(tmp, 'wb'))
  assert(file:write('stop\n'))
  assert(file:flush())
  assert(file:close())
  os.remove(dest)
  assert(os.rename(tmp, dest), 'shutdown file was not published')
end

local function retire(label, identity, shutdown_file)
  local result = {label = label, ok = false, retired = false}
  if not identity then
    result.ok = true
    result.retired = true
    result.detail = 'not started'
    return result
  end
  local ok, err = xpcall(function()
    if shutdown_file then atomic_stop(shutdown_file) end
    if shutdown_file and process.wait_owned(identity, GRACE_MS) then
      -- The publisher observed the shutdown file inside the cleanup allowance.
    elseif process.owned_alive(identity) then
      process.stop_owned(identity)
      assert(process.wait_owned(identity, GRACE_MS), label .. ' remained after stop_owned')
    end
    assert(not process.owned_alive(identity), label .. ' original lifetime was not retired')
  end, debug.traceback)
  result.ok = ok
  result.error = ok and nil or plain(err)
  local alive_ok, alive = pcall(process.owned_alive, identity)
  result.retired = alive_ok and alive == false
  if not result.retired and result.ok then
    result.ok = false
    result.error = label .. ' retirement was not independently observed'
  end
  return result
end

function M.run(options)
  local report = {
    executed = true, success = false, not_run = 0, failed = 1, tests = {}, cleanups = {},
    hash_drift = false, foundation_summary = 'absent',
    publisher_retired = false, editor_retired = false,
  }
  local function finish_tests()
    report.tests = {{
      name = TEST,
      state = report.success and 'Success' or 'Fail',
      errors = report.success and 0 or 1,
      warnings = 0,
    }}
  end
  if type(options) ~= 'table' or type(options.ue_root) ~= 'string' or options.ue_root == '' then
    report.error = 'ue_root is required'
    finish_tests()
    return report
  end
  local phase = options.phase or 'full'
  if phase ~= 'full' and phase ~= 'foundation' and phase ~= 'communication' and phase ~= 'selection' and phase ~= 'accessibility' and phase ~= 'tail' then
    report.error = 'unknown owned viewport phase'; finish_tests(); return report
  end
  report.phase, report.full_case_inventory = phase, phase == 'full'
  local run_id = options.run_id or os.date('!%Y%m%dT%H%M%SZ')
  if not fixture.valid_run_id(run_id) then
    report.error = 'invalid owned viewport run id'
    finish_tests()
    return report
  end
  local state = {}
  -- The editor keeps player 1's controls inside this run's evidence (isolate_controls below); the
  -- player's own Saved/Controls is never captured or rewritten, so a key the user changes meanwhile
  -- survives the run (E141).
  local profile = path.join(b.root, 'omfue', 'Saved', 'Automation', 'OwnedViewportIpc', run_id, 'controls', '1', 'profile.json')
  local profile_existed = path.is_file(profile)
  local profile_bytes = profile_existed and path.read(profile, true) or nil
  local workflow_path
  local ok, err = xpcall(function()
    local foundation_path = path.join(b.root, '.ai-collab', 'renderer-fixture-foundation-accepted', 'summary.json')
    if path.is_file(foundation_path) then
      local foundation = json.read(foundation_path)
      report.foundation_content_hash = foundation.content_hash
      report.foundation_map_catalog_hash = foundation.map_catalog_hash
      report.foundation_summary = 'present'
    end
    local parent = path.join(b.root, 'omfue', 'Saved', 'Automation', 'OwnedViewportIpc')
    path.mkdir_p(parent)
    local evidence = path.join(parent, run_id)
    assert(not path.exists(evidence), 'evidence directory must be a fresh path')
    -- Unreal runs from a binaries snapshot, never live omfue, so the user's own build can
    -- relink the DLLs while this runs (E116).
    local project = require('ue_run_snapshot').prepare(b.root).project_dir
    local editor = path.join(path.absolute(options.ue_root), 'Engine', 'Binaries', 'Win64', 'UnrealEditor-Cmd.exe')
    assert(path.is_file(editor), 'UnrealEditor-Cmd.exe is missing')
    assert(path.is_file(path.join(project, 'om.uproject')), 'om.uproject is missing')
    workflow_path = path.join(parent, run_id .. '.workflow.json')
    local native_report = path.join(parent, run_id .. '.native.json')
    local ue_report = path.join(parent, run_id .. '.ue-report')
    local abslog = path.join(parent, run_id .. '.ue.log')
    path.mkdir_p(ue_report)
    fixture.build()
    local started = time.monotonic_ms()
    local deadline = started + OPERATION_MS
    local _, owned = fixture.spawn_fixture(run_id, evidence, LIFETIME_SECONDS, path.join(evidence, 'shutdown.signal'), path.join(parent, run_id .. '.ownership.json'))
    state.publisher = owned
    state.shutdown = path.join(evidence, 'shutdown.signal')
    state.evidence = evidence
    state.run_id = run_id
    local startup_budget = remaining_ms(started, deadline)
    assert(startup_budget > 0, 'publisher startup exceeded the shared deadline')
    local startup = fixture.wait_startup(owned, evidence, run_id, math.min(startup_budget, 120000))
    assert(startup.address:match('^127%.0%.0%.1:9$') == nil, 'startup port is the placeholder port')
    assert(startup.moba_map_id == 'three_lane_training', 'startup map mismatch')
    assert(integer(startup.source_render_id, 'startup source') == 11 and integer(startup.target_render_id, 'startup target') == 21, 'startup render ids are not canonical')
    assert(type(startup.content_catalog_identity_hash) == 'string' and startup.content_catalog_identity_hash ~= '', 'startup catalog identity is empty')
    assert(type(startup.moba_map_catalog_hash) == 'string' and startup.moba_map_catalog_hash ~= '', 'startup map catalog is empty')
    report.live_content_hash = startup.content_hash
    report.live_catalog_identity_hash = startup.content_catalog_identity_hash
    report.live_map_catalog_hash = startup.moba_map_catalog_hash
    report.live_address = startup.address
    -- The renderer's bridge rejects a runtime whose content hash differs from its own and the
    -- run then times out with no frame (E130). The fixture is rebuilt for every run; the
    -- bridge only when the frontend is built, so refuse a stale bridge here with the reason.
    local bridge = path.join(project, require('ue_bridge_stage').DLL)
    local bridge_content = path.is_file(bridge) and require('run_artifacts').content_hash(bridge)
    report.bridge_content_hash = bridge_content or nil
    assert(not bridge_content or bridge_content == startup.content_hash:lower(),
      ('renderer bridge content %s differs from the fixture content %s: the bridge would reject the fixture '..
       'and no frame would arrive. Build the frontend (build_ue_moba) or run a shadow build that matches '..
       '(stage_ue_shadow.lua + OM_UE_RUN_BINARIES): %s'):format(bridge_content or '?', startup.content_hash, bridge))
    if report.foundation_summary == 'present'
        and type(report.foundation_content_hash) == 'string'
        and type(report.foundation_map_catalog_hash) == 'string' then
      report.hash_drift = startup.content_hash ~= report.foundation_content_hash
        or startup.moba_map_catalog_hash ~= report.foundation_map_catalog_hash
    end
    local left = remaining_ms(started, deadline)
    assert(left > 0, 'no time remained for the owned editor')
    local _, editor_owned = process.spawn_owned(editor, {
      path.join(project, 'om.uproject'),
      '-unattended', '-nop4', '-NoSound', '-NoLiveCoding', '-d3d12',
      require('ue_desktop').args()[1], -- off the user's desktop unless OM_OWN_DESKTOP=1 (E114)
      require('ue_desktop').isolate_controls({}, path.join(evidence, 'controls'))[1], -- never the player's profile (E141)
      '-om-team=1', '-om-native-content', '-om-renderer-fixture',
      options.record_video and '-om-owned-video' or '-om-owned-video-disabled',
      '-om-owned-phase=' .. phase,
      '-om-renderer-fixture-evidence=' .. evidence,
      '-om-renderer-fixture-run=' .. run_id,
      '-om-renderer-fixture-report=' .. native_report,
      '-ExecCmds=Automation RunTests ' .. TEST,
      '-TestExit=Automation Test Queue Empty',
      '-ReportExportPath=' .. ue_report,
      '-abslog=' .. abslog,
    }, {
      cwd = project,
      stdout = path.join(parent, run_id .. '.ue.stdout.log'),
      stderr = path.join(parent, run_id .. '.ue.stderr.log'),
    })
    state.editor = editor_owned
    assert(process.wait_owned(editor_owned, left), 'owned editor exceeded the shared deadline')
    local native = json.read(native_report)
    local loaded = M.open_dependent_artifacts(native, function()
      return {
        intents = lines_of(path.read(path.join(evidence, 'intents.jsonl'), true)),
        history = lines_of(path.read(path.join(evidence, 'control-history.jsonl'), true)),
        ack = json.read(path.join(evidence, 'control.ack.json')),
        ue_index = path.read(path.join(ue_report, 'index.json'), true),
      }
    end)
    if options.record_video then
      report.video = require('owned_window_video').verify(evidence, native)
      json.write(path.join(evidence, 'owned-window-video.json'), report.video, false)
    end
    M.verify_artifacts({
      phase = phase,
      startup = startup,
      native = native,
      intents = loaded.intents,
      history = loaded.history,
      ack = loaded.ack,
      ue_index = loaded.ue_index,
    })
    local function assert_display_png(name)
      local bytes = assert(path.read(path.join(evidence, name), true), 'missing owned display capture: ' .. name)
      assert(bytes:sub(1, 8) == '\137PNG\13\10\26\10', name .. ' is not a png')
    end
    if phase == 'full' or phase == 'foundation' then
      assert_display_png('display-hud-visible.png')
      assert_display_png('display-hud-hidden.png')
      assert_display_png('display-settings-visible.png')
    end
    if phase == 'full' or phase == 'selection' or phase == 'tail' then
      assert_display_png('selection-enemy-800.png'); assert_display_png('selection-enemy-400.png')
    end
    if phase == 'full' or phase == 'accessibility' or phase == 'tail' then assert_display_png('accessible-controls.png') end
    if phase == 'full' or phase == 'communication' or phase == 'tail' then
    assert_display_png('chat-team-draft.png')
    assert_display_png('chat-all-draft.png')
    assert_display_png('chat-received-800.png')
    assert_display_png('chat-received-400.png')
    assert_display_png('chat-received-large.png')
    assert_display_png('chat-roster-reconnected.png')
    assert_display_png('chat-all-disabled.png')
    assert_display_png('chat-received-all-disabled.png')
    assert_display_png('chat-text-controls.png')
    assert_display_png('hud-public-score.png'); assert_display_png('objective-vote-north.png'); assert_display_png('objective-vote-south.png')
    for _,name in ipairs({'signals-received-800','signals-received-400','signals-received-large','signals-expired','signals-retired','combat-overflow-before','combat-overflow-after'}) do assert_display_png(name..'.png') end
    for _, name in ipairs({'ping-wheel-800','emote-wheel-800','ping-wheel-400','emote-wheel-400','ping-wheel-large','emote-wheel-large','ping-alt-pointer','ping-ctrl-pointer','ping-minimap-alt','ping-minimap-ctrl'}) do
      assert_display_png(name .. '.png')
    end
    end
  end, debug.traceback)
  local publisher_cleanup = retire('publisher', state.publisher, state.evidence and state.shutdown or nil)
  local editor_cleanup = retire('editor', state.editor, nil)
  report.cleanups = {publisher_cleanup, editor_cleanup}
  report.publisher_retired = publisher_cleanup.retired == true
  report.editor_retired = editor_cleanup.retired == true
  local profile_ok, profile_err = xpcall(function()
    if not state.editor then return end
    local alive_ok, alive = pcall(process.owned_alive, state.editor)
    if not alive_ok or alive ~= false then
      error('editor lifetime is still alive; player profile was not restored', 0)
    end
    if profile_existed then path.write(profile, profile_bytes, true, true)
    elseif path.is_file(profile) then assert(os.remove(profile), 'modified player-1 profile was not removed') end
  end, debug.traceback)
  report.cleanups[#report.cleanups + 1] = {label = 'profile', ok = profile_ok, error = profile_ok and nil or plain(profile_err)}
  if ok and state.evidence then
    local cleanup_ok, cleanup_err = xpcall(function()
      local cleanup = fixture.read_cleanup(state.evidence, state.run_id)
      assert(cleanup and cleanup.listener_released == true, 'listener_released is not true')
      assert(cleanup.authority == false, 'cleanup claimed authority')
    end, debug.traceback)
    if not cleanup_ok then
      ok = false
      err = cleanup_err
    end
  end
  if ok and not (publisher_cleanup.ok and editor_cleanup.ok and profile_ok and report.publisher_retired and report.editor_retired) then
    ok = false
    err = publisher_cleanup.error or editor_cleanup.error or profile_err or 'owned lifetimes were not retired'
  end
  report.success = ok == true
  report.failed = report.success and 0 or 1
  report.not_run = 0
  report.error = report.success and nil or plain(err)
  finish_tests()
  if workflow_path then
    local wrote, write_err = pcall(json.write, workflow_path, report, true)
    if not wrote then
      local preserved = report.error
      report.success = false
      report.failed = 1
      report.not_run = 0
      local detail = 'workflow report was not written: ' .. plain(write_err)
      report.error = preserved and (preserved .. '; ' .. detail) or detail
      finish_tests()
      pcall(json.write, workflow_path, report, true)
    end
  end
  return report
end

if is_entry() then
  if arg[1] == '--help' or arg[1] == nil then
    print([[Usage: tools\lua\lua.exe scripts\moba_owned_viewport_ipc.lua --ue-root PATH [--run-id ID] [--phase full|foundation|communication|selection|accessibility|tail]
Builds the compiled-content fixture, starts it on loopback, and runs Om.Runtime.OwnedViewportIpc.
A wire intent is local admission only. This entry does not run full local acceptance.]])
    if arg[1] == nil then os.exit(2) end
  else
    process.supervise_workflow(source, arg)
    local ue_root, run_id, phase, record_video
    local index = 1
    while index <= #arg do
      if arg[index] == '--ue-root' then index = index + 1; ue_root = assert(arg[index], '--ue-root requires a path')
      elseif arg[index] == '--run-id' then index = index + 1; run_id = assert(arg[index], '--run-id requires a value')
      elseif arg[index] == '--phase' then index = index + 1; phase = assert(arg[index], '--phase requires a value')
      elseif arg[index] == '--record-video' then record_video = true
      else error('unknown argument: ' .. tostring(arg[index])) end
      index = index + 1
    end
    local result = M.run({ue_root = ue_root, run_id = run_id, phase = phase, record_video = record_video})
    if not result.success then error(result.error or 'owned viewport IPC failed', 0) end
  end
end

return M
