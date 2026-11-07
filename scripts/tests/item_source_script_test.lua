local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local analyzer = require('item_source.script')

-- E169: source item scripts are read as text; the analysis says what a function does.
local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end
local text = [[
-- a comment with function and end inside: if x then end
function Shock( keys )
	local caster = keys.caster
	local particle = ParticleManager:CreateParticle("particles/a.vpcf",PATTACH_POINT,caster)
	ParticleManager:SetParticleControl(particle,0, caster:GetAbsOrigin())
	EmitSoundOnLocationWithCaster(caster:GetAbsOrigin(),"end function",caster)
	for itemSlot = 0,8 do
		local item = caster:GetItemInSlot(itemSlot)
		if item ~= nil and ((item:GetName() == "item_fire_scroll") or (item:GetName() == "item_recipe_x") or
				(item:GetName() == "item_flame_book")) then
			item:StartCooldown(ability:GetCooldown(-1))
		end
	end
end

function Buff( keys )
	if keys.caster then
		ability:ApplyDataDrivenModifier(keys.caster, keys.caster, "modifier_x", {})
	end
end
]]
local shock = analyzer.analyze(text, 'Shock')
ok(shock.found and shock.line == 2, 'finds the function after a comment')
ok(shock.presentation_only, 'particles, sounds and the cooldown loop are presentation: ' .. table.concat(shock.calls, ','))
ok(#shock.cooldown_items == 2 and shock.cooldown_items[1] == 'fire_scroll' and shock.cooldown_items[2] == 'flame_book',
  'cooldown items in order, recipes left out')
local buff = analyzer.analyze(text, 'Buff')
ok(buff.found and not buff.presentation_only and buff.calls[1] == 'ApplyDataDrivenModifier', 'a game effect is reported')
ok(not analyzer.analyze(text, 'Missing').found, 'a missing function is not found')
-- E176: target effects in their source shape become data; any other effect call refuses.
local effects = [[
function Shock( keys )
	local caster = keys.caster
	local target = keys.target
	local ability = keys.ability
	if not target:IsBuilding() then
		AMHC:Damage(caster,target,150,AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )
		ability:ApplyDataDrivenModifier(caster, target,"modifier_stunned",{duration=0.05})
	end
end
function Other( keys )
	keys.ability:ApplyDataDrivenModifier(keys.caster, keys.caster, "modifier_self", nil)
end
]]
local star = analyzer.analyze(effects, 'Shock')
ok(#star.calls == 0 and not star.presentation_only and #star.effects == 2, 'damage and stun are explained effects')
ok(star.effects[1].kind == 'apply_modifier' and star.effects[1].modifier == 'modifier_stunned' and star.effects[1].duration == 0.05
  and star.effects[1].non_building, 'modifier, duration and the building guard')
ok(star.effects[2].kind == 'damage' and star.effects[2].amount == 150 and star.effects[2].damage_type == 'magical', 'magical damage')
local other = analyzer.analyze(effects, 'Other')
ok(#other.effects == 0 and other.calls[1] == 'ApplyDataDrivenModifier', 'a modifier on someone else is not a target effect')
-- E178: a point area loop becomes data.
local napalm = [[
function Shock( keys )
	local caster = keys.caster
	local point = keys.target_points[1]
	local ability = keys.ability
	local SEARCH_RADIUS = 300
	GridNav:DestroyTreesAroundPoint(point, SEARCH_RADIUS, false)
	local direUnits = FindUnitsInRadius(caster:GetTeamNumber(), point, nil, SEARCH_RADIUS,
		DOTA_UNIT_TARGET_TEAM_ENEMY, DOTA_UNIT_TARGET_ALL, DOTA_UNIT_TARGET_FLAG_NONE, FIND_ANY_ORDER, false)
	local dmg = 300
	for _,it in pairs(direUnits) do
		if (not(it:IsBuilding())) then
			AMHC:Damage(caster,it,dmg,AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )
			ability:ApplyDataDrivenModifier(caster, it,"modifier_stunned", {duration = 0.1})
		end
	end
end
]]
local area = analyzer.analyze(napalm, 'Shock')
ok(#area.calls == 0 and area.area and area.area.radius == 300 and area.area.damage == 300 and area.area.damage_type == 'magical'
  and area.area.stun == 0.1 and area.area.skip_buildings, 'radius, damage, type, stun and building guard from the loop')
local ally = napalm:gsub('DOTA_UNIT_TARGET_TEAM_ENEMY', 'DOTA_UNIT_TARGET_TEAM_FRIENDLY')
ok(not analyzer.analyze(ally, 'Shock').area, 'a friendly area is not this pattern')
-- E188: a review holds only while its verbatim fragments (spaces ignored) are in the function.
local review = { requires = { 'AMHC:Damage(caster,it,dmg,AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )' } }
ok(analyzer.review_matches(review, napalm:gsub('AMHC:Damage%(', 'AMHC:Damage(  ')), 'whitespace is ignored')
local matches, missing = analyzer.review_matches(review, napalm:gsub('it,dmg,AMHC', 'it,2*dmg,AMHC'))
ok(not matches and missing == review.requires[1], 'a changed expression drops the review')
for key, reviewed in pairs(analyzer.REVIEWED_EFFECTS) do
  ok(reviewed.kind == 'magic_immunity' or (reviewed.requires and #reviewed.requires > 0), key .. ' names the fragments it relies on')
end
print(('item source script: %d checks passed'):format(checks))
