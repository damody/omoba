-- Explicit managed-mana recipe; legacy launch defaults remain unchanged.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_archetype_single_player.lua'))()
plan.mana_enabled=true
plan.sustain.mana={recall_below_per_mille=200,leave_base_at_per_mille=850}
-- Preserve healing priority; only use the mana branch when the host's current
-- cost (including discounts) is lower than the declared immediate restoration.
for _,policy in ipairs(plan.ability_policies) do
    if policy.ability=='lumen_touch' then
        policy.intent={kind='self_recovery',below_hp_per_mille=600,
            below_mana_per_mille=500,restore_key='mana_restore'}
    end
end
return plan
