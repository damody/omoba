# 來源資料：C15 玉子

由 `scripts/hero_source_index.lua export` 從 source-catalog 產生；內容只有來源事實，沒有分析。

- 倉庫：https://github.com/damody/nobugg_official
- commit：`ca6ee8602473f6015a397cc121dac6baeb70b2e5`
- 陣營：C_Neutral；主檔：`npc/C_Neutral/C15.txt`；單位檔：`npc/C_Neutral/C15_2.txt`
- 攻擊：DOTA_UNIT_CAP_RANGED_ATTACK，射程 700；主屬性 DOTA_ATTRIBUTE_AGILITY

## 單位（npc/C_Neutral/C15_2.txt:4，raw_sha256 `2300ffd2ed29c92ea085789d2e05d34c854c555a66677b5da3b2f3708e744527`）

```text
"C15"
	{
		// General
		// ----------------------------------------------------------------
		"override_hero"	"npc_dota_hero_mirana"
		//"Level"	"5"
		"BoundsHullName"	"DOTA_HULL_SIZE_HERO"
		"HealthBarOffset"	"180"
		"Model"	"models/heroes/mirana/mirana.vmdl"
		"Portrait"	"vgui/hud/heroportraits/portrait_mirana"
		"IdleExpression"	"scenes/mirana/mirana_exp_idle_01.vcd"
		"SoundSet"	"Hero_Mirana"
		"PickSound"	"mirana_mir_spawn_01"
		"BanSound"	"mirana_mir_anger_01"
		"ModelScale"	"1.0"
		//"Model"	"models/c15/c15.vmdl"
		//"DisableWearables"	"1"
		"HeroGlowColor"	"220 194 248"
		"CMEnabled"	"1"
		"url"	"Mirana"
		"LastHitChallengeRival"	"npc_dota_hero_luna"
		"HeroSelectSoundEffect"	"Hero_Mirana.Pick"
		// Abilities
		// ---------------
		"Ability1"	"C15W"
		"Ability2"	"C15E"
		"Ability3"	"C15R"
		"Ability4"	"magical_resistance"
		"Ability5"	"C15_precache"
		"Ability6"	"C15T"
		"Ability7"	""
		// Armor
		// ----------------------------------------------------------------
		"ArmorPhysical"	"3"
		"MagicalResistance"	"0"
		// Attack
		// ----------------------------------------------------------------
		"AttackCapabilities"	"DOTA_UNIT_CAP_RANGED_ATTACK"
		"AttackDamageMin"	"95.0"
		"AttackDamageMax"	"95.0"
		"AttackDamageType"	"DAMAGE_TYPE_ArmorPhysical"
		"AttackRate"	"1.80"
		"AttackAnimationPoint"	"0.20"
		"AttackAcquisitionRange"	"700"
		"AttackRange"	"700"
		"ProjectileSpeed"	"1800"
		// Attributes
		// ----------------------------------------------------------------
		"AttributePrimary"	"DOTA_ATTRIBUTE_AGILITY"
		"AttributeBaseStrength"	"28"
		"AttributeStrengthGain"	"1.60"
		"AttributeBaseAgility"	"30"
		"AttributeAgilityGain"	"2.90"
		"AttributeBaseIntelligence"	"29"
		"AttributeIntelligenceGain"	"2.40"		
		// Bounty
		// ----------------------------------------------------------------
		"BountyGoldMin"	"0"
		"BountyGoldMax"	"0"
		// Gold and Lumber
		// ----------------------------------------------------------------
		"GoldCost"	"0"
		"LumberCost"	"0"
		// Movement
		// ----------------------------------------------------------------
		"MovementCapabilities"	"DOTA_UNIT_CAP_MOVE_GROUND"
		"MovementSpeed"	"350"
		"MovementTurnRate"	"360"
		// Status
		// ----------------------------------------------------------------
		"StatusHealth"	"550"
		"StatusHealthRegen"	"0.25"
		"StatusMana"	"0"
		"StatusManaRegen"	"0.01"
		"StatusStartingMana"	"100"
		// Vision
		// ----------------------------------------------------------------
		"VisionDaytimeRange"	"1600"
		"VisionNighttimeRange"	"1000"
		// Team
		// ----------------------------------------------------------------
		"TeamName"	"DOTA_TEAM_NEUTRALS"
		"CombatClassAttack"	"DOTA_COMBAT_CLASS_ATTACK_HERO"
		"CombatClassDefend"	"DOTA_COMBAT_CLASS_DEFEND_HERO"
		"UnitRelationShipClass"	"DOTA_NPC_UNIT_RELATIONSHIP_TYPE_DEFAULT"
		"Ability8"	""
		"Ability9"	""
		"Ability10"	""
		"Ability11"	""
		"Ability12"	""
		"Ability13"	""
		"Ability14"	""
		"Ability15"	""
		"Ability16"	""
	}
```

## 技能

### C15W：亂射（單位技能欄 1）

- 位置：`npc/C_Neutral/C15.txt:17`，raw_sha256 `1a63382a2d1e6c0af3ef28c96dc9a6584c0f4eabfc0c768c8383b42f08ea0f75`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_C15W_Description，raw_sha256 `0fc005ff6d84a852e2be251aaf05d6e8d1e92d691e206a5120fae87f632ab214`）：

  > 對著範圍內的每個目標射箭，每隻箭矢可以造成(75/150/225/300)傷害並擊暈(0.6/1.2/1.8/2.4)秒。 

```text
"C15W"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"mirana_starfall"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_AOE | DOTA_ABILITY_BEHAVIOR_IGNORE_BACKSWING"
		"AbilityUnitTargetTeam"			"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetType"			"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitDamageType"			"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_NO"
		"FightRecapLevel"	"1"

		"AbilityCastAnimation" "ACT_DOTA_ATTACK"
		// Damage.
		// -------------------------------------------------------------------------------------------------------------
		"AbilityDamage"	"75 150 225 300"
		// Casting
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastPoint" "0.3" //stun
		// Time
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCooldown"	"18 15 12 9"
		// Cost
		// -------------------------------------------------------------------------------------------------------------
		"AbilityManaCost"	"140 160 180 200"
		"AbilityCastRange"	"700"

		"AOERadius"	"%radius"
		// Precache
		// -------------------------------------------------------------------------------------------------------------
		"precache"
		{
			"particle"	"particles/econ/items/drow/drow_bow_monarch/drow_frost_arrow_monarch.vpcf"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_mirana.vsndevts"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C15W.vo1"
			}
			"ActOnTargets"
			{
				"Target"
				{
					"Center"  	"POINT"
					"Radius" 	"%radius"
					"Teams" 	"DOTA_UNIT_TARGET_TEAM_ENEMY"
					"Types" 	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
				}
				"Action"    
				{
					"TrackingProjectile"
					{
						"Target"           	"TARGET"
						"EffectName"		"particles/econ/items/drow/drow_bow_monarch/drow_frost_arrow_monarch.vpcf"
						"Dodgeable"			"0"
						"ProvidesVision"	"0"
						"VisionRadius"		"0"
						"MoveSpeed"        	"1500"
						"SourceAttachment"	"DOTA_PROJECTILE_ATTACHMENT_ATTACK_1"
					}
				}
			}
			"FireSound"
			{
				"EffectName"	"Ability.Starfall"
				"Target"	"CASTER"
			}
		}
		"OnProjectileHitUnit"
		{
			"DeleteOnHit"	"1"
			"Stun"
			{
				"Target"	"TARGET"
				"Duration"	"%stun_duration"
			}
			"Damage"
			{
				"Target"	"TARGET"
				"Damage"	"%AbilityDamage"
				"Type"	"DAMAGE_TYPE_MAGICAL"
			}
			"FireSound"
			{
				"Target"	"TARGET"
				"EffectName"	"Ability.StarfallImpact"
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"radius"	"275"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"stun_duration"	"0.6 1.2 1.8 2.4"
			}
		}
	}
```

### C15E：爆裂箭（單位技能欄 2）

- 位置：`npc/C_Neutral/C15.txt:124`，raw_sha256 `1fa0f6dc992a8de957b1d337d3038085dca0ef44f0dd571ba8d2da977879a1ca`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_C15E_Description，raw_sha256 `6ff8b4607459b54bee0a51d61bba7fe9917680f702d7480ba9881d4c8e512e04`）：

  > 對指定的方向射出一支爆裂箭，爆裂箭將會在經過的路徑上留下一排寬度170的火焰，碰到火焰的敵人每秒受到(100/120/140/160)的傷害，火焰存在12秒。 
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15E`

```text
"C15E"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"clinkz_searing_arrows"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_DIRECTIONAL | DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_IGNORE_BACKSWING"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_NO"
		"FightRecapLevel"	"1"
		"AbilityCastPoint"          				"0.2"
		"AbilityCastAnimation"	"ACT_DOTA_CAST_ABILITY_2"
		// Time
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCooldown"	"22 20 18 16"
		// Cost
		// -------------------------------------------------------------------------------------------------------------
		"AbilityManaCost"	"150 170 190 210"
		// Casting
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastRange"	"1350"
		// Precache
		// -------------------------------------------------------------------------------------------------------------
		"precache"
		{
			"particle"	"particles/c15t/c15t.vpcf"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_mirana.vsndevts"
			"particle"	"particles/c15t/c15t.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C15E.vo1"
			}
			"FireSound"
			{
				"EffectName"	"Hero_Mirana.ArrowCast"
				"Target"	"CASTER"
			}
			"LinearProjectile"
			{
				"Target"	"POINT"
				"EffectName"	"particles/c15t/c15t.vpcf"
				"MoveSpeed"	"%arrow_speed"
				"StartRadius"	"0"
				"StartPosition"	"attach_origin"
				"EndRadius"	"0"
				"FixedDistance"	"%arrow_range"
				"TargetTeams"	"DOTA_UNIT_TARGET_TEAM_NONE"
				"TargetTypes"	"DOTA_UNIT_TARGET_TREE"
				"HasFrontalCone"	"0"
				"ProvidesVision"	"0"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
				"Function"	"new_C15E"
			}
		}
		"Modifiers"
		{
			"modifier_C15E_2"
			{
				"ThinkInterval"	"0.1"
				"Duration"	"15"
				"IsDebuff"	"0"
				"EffectAttachType"	"attach_origin"
				"States"
				{
					"MODIFIER_STATE_INVULNERABLE"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"OnIntervalThink"
				{
					"ActOnTargets"
					{
						"Target"
						{
							"Types"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
							"Teams"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
							"Center"	"TARGET"
							"Radius"	"%arrow_width"
						}
						"Action"
						{
							"ApplyModifier"
							{
								"Target"	"TARGET"
								"ModifierName"	"modifier_C15E_3"
							}
						}
					}
				}
				"EffectName"	"particles/b02e5/b02e5.vpcf"
			}
			"modifier_C15E_3"
			{
				"ThinkInterval"	"0.25"
				"Duration"	"1.01"
				"IsDebuff"	"0"
				"OnIntervalThink"
				{
					"Damage"
					{
						"Target"	"TARGET"
						"Type"	"DAMAGE_TYPE_MAGICAL"
						"Damage"	"%dmg"
					}
				}
				"IsHidden"	"1"
			}
		}
		// Special
		// -------------------------------------------------------------------------------------------------------------
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"spell_duration"	"0.6"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"arrow_speed"	"1666"
			}
			"03"
			{
				"var_type"	"FIELD_INTEGER"
				"arrow_width"	"170"
			}
			"04"
			{
				"var_type"	"FIELD_INTEGER"
				"arrow_range"	"2000"
			}
			"05"
			{
				"var_type"	"FIELD_INTEGER"
				"dmg"	"25 30 35 40"
			}
			"06"
			{
				"var_type"	"FIELD_FLOAT"
				"duration"	"12"
			}
		}
	}
```

### C15R：精準之光（單位技能欄 3）

- 位置：`npc/C_Neutral/C15.txt:278`，raw_sha256 `11aeebd1f4d71698e9cef4af2783de1af39bb87d0250d5dc0048c8da23f85712`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_C15R_Description，raw_sha256 `e16e88df3e6581393504492d5fdc100729e80ca84a85e360fbcf6e2fa7e05f18`）：

  > 增加(10/18/26/34)% 攻擊傷害以及(5/10/15/20)靈活。 

```text
"C15R"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"mirana_invis"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_PASSIVE"
		"Modifiers"
		{
			"modifier_C15R"
			{
				"Passive"	"1"
				"Properties"
				{
					"MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE"	"%per_atk"
					"MODIFIER_PROPERTY_STATS_AGILITY_BONUS"	"%agi"
				}
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"per_atk"	"10 18 26 34"
			}
			"02"
			{
				"var_type"	"FIELD_INTEGER"
				"agi"	"5 10 15 20"
			}
		}
	}
```

### C15_precache（單位技能欄 5）

- 位置：`npc/C_Neutral/C15.txt:4`，raw_sha256 `7b3de2f3fd4dfefc707c66cff9db05d344ee54aab20e80e64a4130df422c3d4b`
- 版本：current
- tooltip：無

```text
"C15_precache"
	{
		"BaseClass"             		"ability_datadriven"
		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_HIDDEN"
		"precache"
		{
			"soundfile"	"soundevents/c15.vsndevts"
		}
	}
```

### C15T：百箭齊發（單位技能欄 6）

- 位置：`npc/C_Neutral/C15.txt:310`，raw_sha256 `75150edbb63e39290eb1b5368f84a6b3db60a2ab930df94faf9bf44f8d4e5748`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_C15T_Description，raw_sha256 `a0c9c36f0398c0be5e85368e77cdb83f2471f22205cf331622056668a589e106`）：

  > (被動)每攻擊8下將產生一個致命箭矢來攻擊目標。(啟動)震暈對手(1/1.5/2)秒，同時射出10發致命箭矢攻擊目標，若是指定的目標死亡，所有箭矢將隨機找尋附近敵人進行攻擊，每隻箭矢都擁有(50/75/100)點攻擊力傷害，持續10秒。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T`, `scripts/vscripts/heroes/C_Neutral/C15.lua#C15T_OnAttackLanded`, `scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T_end`, `scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T_attack`

```text
"C15T"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"drow_ranger_marksmanship"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_UNIT_TARGET | DOTA_ABILITY_BEHAVIOR_IGNORE_BACKSWING"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetFlags"	"DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_PHYSICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_YES"
		"AbilityType"	"DOTA_ABILITY_TYPE_ULTIMATE"
		"RequiredLevel"	"6"
		"LevelsBetweenUpgrades"	"6"
		"AbilityCastPoint"          				"0.3"
		// Time
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCooldown"	"140 115 90"
		// Cost
		// -------------------------------------------------------------------------------------------------------------
		"AbilityManaCost"	"150"
		// Casting
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastRange"	"700"
		// Precache
		// -------------------------------------------------------------------------------------------------------------
		"precache"
		{
			"particle"	"particles/base_attacks/ranged_tower_good_explosion.vpcf"
			"soundfile"	"sounds/weapons/hero/sven/attack02.vsnd"
			"soundfile"	"sounds/weapons/hero/necrolyte/attack1.vsnd"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_disruptor.vsndevts"
			"particle"	"particles/c15t7/c15t7.vpcf"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_legion_commander.vsndevts"
			"particle"	"particles/generic_gameplay/generic_stunned.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C15T.vo1"
			}
			"FireSound"
			{
				"EffectName"	"Hero_Disruptor.PreAttack"
				"Target"	"CASTER"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
				"Function"	"new_C15T"
			}
		}
		"Modifiers"
		{
			"modifier_C15T_trigger"
			{
				"Passive" 			"1"
				"IsDebuff"			"0"
				"OnAttackLanded"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
						"Function"		"C15T_OnAttackLanded"
					}
				}
			}
			"modifier_C15T"
			{
				"EffectName"	"particles/c15t7/c15t7.vpcf"
				"States"
				{
					"MODIFIER_STATE_INVULNERABLE"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"Properties"
				{
					"MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE"	"%attack_dmg"
				}
				"Duration"	"%life_duration"
				"OnDestroy"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
						"Function"	"new_C15T_end"
					}
				}
				"OnAttack"
				{
					"RunScript"
					{
						"Function"	"new_C15T_attack"
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
					}
					"FireSound"
					{
						"Target"	"TARGET"
						"EffectName"	"Hero_LegionCommander.Attack"
					}
				}
			}
			"modifier_C15T_2"
			{
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"IsPurgable"	"1"
				"IsDebuff"		"1"
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"Duration"	"%stun_duration"
				"OnCreated"
				{
					"Damage"
					{
						"Target"	"TARGET"
						"Type"	"DAMAGE_TYPE_MAGICAL"
						"Damage"	"5"
					}
				}
			}
		}
		// Special
		// -------------------------------------------------------------------------------------------------------------
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"spell_duration"	"0.6"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"stun_duration"	"1 1.5 2"
			}
			"03"
			{
				"var_type"	"FIELD_FLOAT"
				"life_duration"	"10"
			}
			"04"
			{
				"var_type"	"FIELD_INTEGER"
				"attack_dmg"	"50 100 150"
			}
			"05"
			{
				"var_type"	"FIELD_INTEGER"
				"attack_speed"	"1"
			}
		}
	}
```

### C15W_old：三連矢（不在單位技能清單）

- 位置：`npc/C_Neutral/C15.txt:471`，raw_sha256 `135cba46db73baed11baf6243f9402f6269ec97e93ffde298ae40f887da1f7c6`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_C15W_old_Description，raw_sha256 `deb54a0b1e655b4a4fb9e20b437f6a28615191e7f25acd743a511ebc313adced`）：

  > 對著目標射出三支箭矢，每支箭矢可以造成(50/100/150/200)傷害並且擊暈對手(0.5/1.0/1.5/2.0)秒。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/C_Neutral/C15.lua#C15W_old`

```text
"C15W_old"
	{
		"BaseClass"             	"ability_datadriven"
		"AbilityTextureName"		"custom/B33W"
		"AbilityBehavior"			"DOTA_ABILITY_BEHAVIOR_UNIT_TARGET"
		"AbilityUnitTargetTeam"		"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetType"		"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitDamageType"		"DAMAGE_TYPE_MAGICAL"
		"FightRecapLevel"	"1"
		"AbilityDamage"	"50 100 150 200"
		// Casting
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastPoint" "0.25" //stun
		"AbilityCastAnimation"	"ACT_DOTA_ATTACK"
		// Time
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCooldown"	"30 27 24 20"
		// Cost
		// -------------------------------------------------------------------------------------------------------------
		"AbilityManaCost"	"100 120 140 160"
		"AbilityCastRange"	"800"
		// Precache
		// -------------------------------------------------------------------------------------------------------------
		"precache"
		{
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_mirana.vsndevts"
			"particle"  "particles/units/heroes/hero_mirana/mirana_base_attack.vpcf"
			"particle"	"particles/units/heroes/hero_mirana/mirana_starfall_circle.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C15W_old.vo1"
			}
			"FireSound"
			{
				"EffectName"	"Ability.Starfall"
				"Target"		"CASTER"
			}
			"FireEffect"
			{
				"EffectName"		"particles/units/heroes/hero_mirana/mirana_starfall_circle.vpcf"
				"EffectAttachType"	"attach_origin"
				"Target"			"CASTER"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
				"Function"		"C15W_old"
			}
		}
		"OnProjectileHitUnit"
		{
			"DeleteOnHit"	"1"
			"Stun"
			{
				"Target"	"TARGET"
				"Duration"	"%stun_duration"
			}
			"Damage"
			{
				"Target"	"TARGET"
				"Damage"	"%AbilityDamage"
				"Type"	"DAMAGE_TYPE_MAGICAL"
			}
			"FireSound"
			{
				"Target"	"TARGET"
				"EffectName"	"Ability.StarfallImpact"
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"stun_duration"	"0.5 1 1.5 2"
			}
		}
	}
```

### C15E_old：烈焰箭（不在單位技能清單）

- 位置：`npc/C_Neutral/C15.txt:554`，raw_sha256 `35b6d50e4ea2e95953babf15faf127c0678f03e78acde0e3b1f2474ce0bd101a`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_C15E_old_Description，raw_sha256 `cd1d73eb5bcf593ceb3af3f9cd743fdfacc3397d811b84cf4838e7e14721d82e`）：

  > 攻擊時增加(60/100/140/180)點火焰傷害，對建築物傷害20% ，每次攻擊消耗(8/10/12/14)點法力。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/C_Neutral/C15.lua#C15E_old_orb_fire`, `scripts/vscripts/heroes/C_Neutral/C15.lua#C15E_old_orb_apply_damage`

```text
"C15E_old"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"clinkz_searing_arrows"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_NO_TARGET | DOTA_ABILITY_BEHAVIOR_TOGGLE"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC | DOTA_UNIT_TARGET_BUILDING"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_MAGICAL"
		"precache"
		{
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_medusa.vsndevts"
			"particle"	"particles/units/heroes/hero_clinkz/clinkz_searing_arrow.vpcf"
		}
		"OnToggleOn"
		{
			"ApplyModifier"
			{
				"ModifierName"	"modifier_C15E_old"
				"Target"	"CASTER"
			}
		}
		"OnToggleOff"
		{
			"RemoveModifier"
			{
				"Target"	"CASTER"
				"ModifierName"	"modifier_C15E_old"
			}
		}
		"Modifiers"
		{
			"modifier_C15E_old"
			{
				"Passive"	"0"
				"IsHidden"	"0"
				"Orb"
				{
					"Priority"	"DOTA_ORB_PRIORITY_ABILITY"
					"CastAttack"	"0"
					"ProjectileName"	"particles/units/heroes/hero_clinkz/clinkz_searing_arrow.vpcf"
				}
				"OnAttackStart"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
						"Function"	"C15E_old_orb_fire"
					}
				}
				"OnAttackLanded"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
						"Function"	"C15E_old_orb_apply_damage"
						"Target"	"TARGET"
					}
					"FireSound"
					{
						"Target"	"TARGET"
						"EffectName"	"Hero_Medusa.AttackSplit"
					}
				}
			}
		}
		// Special
		// -------------------------------------------------------------------------------------------------------------
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"damage_bonus"	"60 100 140 180"
			}
			"02"
			{
				"var_type"	"FIELD_INTEGER"
				"mana_per_attack"	"14 16 18 20"
			}
		}
	}
```

### C15R_old：精準之光（不在單位技能清單）

- 位置：`npc/C_Neutral/C15.txt:638`，raw_sha256 `94e88df193d32c282e18c10f97badfb3232bdc8753f15a770c81d4846215a01a`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_C15R_old_Description，raw_sha256 `70a3f713cf8f0b3cc98078609b3c75adafd23c2535541d44c40c9514a652653e`）：

  > 增加(20/30/40/50)% 攻擊力。

```text
"C15R_old"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"mirana_invis"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_PASSIVE"
		"Modifiers"
		{
			"modifier_C15R"
			{
				"Passive"	"1"
				"Properties"
				{
					"MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE"	"%per_atk"
				}
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"per_atk"	"20 30 40 50"
			}
		}
	}
```

### C15T_old：百箭齊發（不在單位技能清單）

- 位置：`npc/C_Neutral/C15.txt:664`，raw_sha256 `74915f22092412c9781eb0912720e4e8a3cbcde77e3e75bce7c11e97e97fc3ab`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_C15T_old_Description，raw_sha256 `8cf5dca0e9a98a76253501724d4fbf9e2fd7899c010975d898493eaf6e07d024`）：

  > 震暈對手(1/1.5/2)秒，同時射出10發致命的箭矢攻擊目標，若是指定的目標死亡，所有箭矢將隨機找尋附近敵人進行攻擊，每隻箭矢都擁有(90/120/160)點攻擊力傷害，持續10秒。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T`, `scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T_end`, `scripts/vscripts/heroes/C_Neutral/C15.lua#new_C15T_attack`

```text
"C15T_old"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"drow_ranger_marksmanship"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_UNIT_TARGET | DOTA_ABILITY_BEHAVIOR_IGNORE_BACKSWING"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetFlags"	"DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_PHYSICAL"
		"AbilityType"	"DOTA_ABILITY_TYPE_ULTIMATE"
		"RequiredLevel"	"6"
		"LevelsBetweenUpgrades"	"6"
		"AbilityCastPoint"          				"0.35"
		// Time
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCooldown"	"140 115 90"
		// Cost
		// -------------------------------------------------------------------------------------------------------------
		"AbilityManaCost"	"250 300 350"
		// Casting
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastRange"	"700"
		// Precache
		// -------------------------------------------------------------------------------------------------------------
		"precache"
		{
			"particle"	"particles/base_attacks/ranged_tower_good_explosion.vpcf"
			"soundfile"	"sounds/weapons/hero/sven/attack02.vsnd"
			"soundfile"	"sounds/weapons/hero/necrolyte/attack1.vsnd"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_disruptor.vsndevts"
			"particle"	"particles/c15t7/c15t7.vpcf"
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_legion_commander.vsndevts"
			"particle"	"particles/generic_gameplay/generic_stunned.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C15T_old.vo1"
			}
			"FireSound"
			{
				"EffectName"	"Hero_Disruptor.PreAttack"
				"Target"	"CASTER"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
				"Function"	"new_C15T"
			}
		}
		"Modifiers"
		{
			"modifier_C15T"
			{
				"EffectName"	"particles/c15t7/c15t7.vpcf"
				"Properties"
				{
					"MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE"	"%attack_dmg"
				}
				"States"
				{
					"MODIFIER_STATE_INVULNERABLE"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"Duration"	"%life_duration"
				"OnDestroy"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
						"Function"	"new_C15T_end"
					}
				}
				"OnAttack"
				{
					"RunScript"
					{
						"Function"	"new_C15T_attack"
						"ScriptFile"	"scripts/vscripts/heroes/C_Neutral/C15.lua"
					}
					"FireSound"
					{
						"Target"	"TARGET"
						"EffectName"	"Hero_LegionCommander.Attack"
					}
				}
			}
			"modifier_C15T_2"
			{
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"IsPurgable"	"1"
				"IsDebuff"		"1"
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"Duration"	"%stun_duration"
				"OnCreated"
				{
					"Damage"
					{
						"Target"	"TARGET"
						"Type"	"DAMAGE_TYPE_MAGICAL"
						"Damage"	"5"
					}
				}
			}
		}
		// Special
		// -------------------------------------------------------------------------------------------------------------
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"stun_duration"	"1 1.5 2"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"life_duration"	"10"
			}
			"03"
			{
				"var_type"	"FIELD_INTEGER"
				"attack_dmg"	"90 120 160"
			}
		}
	}
```
