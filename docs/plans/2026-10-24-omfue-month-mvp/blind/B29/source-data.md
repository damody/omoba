# 來源資料：B29 村上義清

由 `scripts/hero_source_index.lua export` 從 source-catalog 產生；內容只有來源事實，沒有分析。

- 倉庫：https://github.com/damody/nobugg_official
- commit：`ca6ee8602473f6015a397cc121dac6baeb70b2e5`
- 陣營：B_Unified；主檔：`npc/B_Unified/B29.txt`；單位檔：`npc/B_Unified/B29_2.txt`
- 攻擊：DOTA_UNIT_CAP_MELEE_ATTACK，射程 128；主屬性 DOTA_ATTRIBUTE_STRENGTH

## 單位（npc/B_Unified/B29_2.txt:4，raw_sha256 `ec0f848b47b4343322fea589dd383dfb6c70fb5ae0f5a9d589e8927f23cab62b`）

```text
"B29"
	{
	 //General
	 //--------------------------------------------------------------
		"ModelScale"	"1"
		//"Level"	"1"
		"BoundsHullName"	"DOTA_HULL_SIZE_HERO"
		"HealthBarOffset"	"180"
		"override_hero"	"npc_dota_hero_magnataur"
	 	//Abilities
		//--------------------------------------------------------------

		"Ability1"	"B29W"
		"Ability2"	"B29E"
		"Ability3"	"B29R"
		"Ability4"	"magical_resistance"
		"Ability5"	"B29_precache"
		"Ability6"	"B29T"
		"Ability7"	""
		//Armor
		// --------------------------------------------------------------
		"ArmorPhysical"	"5"
		"MagicalResistance"	"0"
		//Attack
		// --------------------------------------------------------------
		"AttackCapabilities"	"DOTA_UNIT_CAP_MELEE_ATTACK"
		"AttackDamageMin"	"120.0"
		"AttackDamageMax"	"120.0"
		"AttackDamageType"	"DAMAGE_TYPE_ArmorPhysical"
		"AttackRate"	"1.8"
		"AttackAnimationPoint"	"0.3"
		"AttackAcquisitionRange"	"700"
		"AttackRange"	"128"
		//Attributes
		// --------------------------------------------------------------
		"AttributePrimary"	"DOTA_ATTRIBUTE_STRENGTH"
		"AttributeBaseStrength"	"35"
		"AttributeStrengthGain"	"2.7"
		"AttributeBaseAgility"	"30"
		"AttributeAgilityGain"	"2.7"
		"AttributeBaseIntelligence"	"24"
		"AttributeIntelligenceGain"	"2.2"
		//Bounty
		// --------------------------------------------------------------
		"BountyGoldMin"	"300"
		"BountyGoldMax"	"300"
		//Gold and Lumber
		// --------------------------------------------------------------
		"GoldCost"	"0"
		"LumberCost"	"0"
		// Movement
		// --------------------------------------------------------------
		"MovementCapabilities"	"DOTA_UNIT_CAP_MOVE_GROUND"
		"MovementSpeed"	"350"
		"MovementTurnRate"	"360"
		// Status
		// --------------------------------------------------------------
		"StatusHealth"	"550"
		"StatusHealthRegen"	"0.25"
		"StatusMana"	"0"
		"StatusManaRegen"	"0.01"
		"StatusStartingMana"	"100"
		// Vision
		// --------------------------------------------------------------
		"VisionDaytimeRange"	"1600"
		"VisionNighttimeRange"	"1000"
		// Team
		// --------------------------------------------------------------
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
		"AbilityLayout"	"5"
		
	}
```

## 技能

### B29W：動地斧（單位技能欄 1）

- 位置：`npc/B_Unified/B29.txt:27`，raw_sha256 `646dd144ff79dfb163cc2e859e380e866cc1cbd4a6f31c8cfc4b9c9a82d1e60c`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_B29W_Description，raw_sha256 `bb66ed19db1a9a8fecc3f40cbe18a3866fbf738ead61e72a245ef323e4177323`）：

  > 猛擊地面，對附近500的敵軍造成(150/250/350/450)傷害並且震昏(1.0/1.3/1.6/1.9)秒。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29W_OnSpellStart`

```text
"B29W"
	{
		"BaseClass"             		"ability_datadriven"
		"AbilityTextureName"			"custom/icon_128x128_25"
		"AbilityType"					"DOTA_ABILITY_TYPE_BASIC"
		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_NO_TARGET"
		"AbilityUnitTargetType"			"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"			"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitDamageType"			"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_NO"
		"AbilityCastPoint" "0.3" //stun
		"AbilityCooldown"				"14"
		"AbilityManaCost"				"100 120 140 160"
		"AbilityCastAnimation"			"ACT_DOTA_CAST_ABILITY_2"
		"AOERadius" 					"500"
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"		"FIELD_INTEGER"
				"damage"	"150 250 350 450"
			}
			"02"
			{
				"var_type"		"FIELD_FLOAT"
				"stun"			"1 1.3 1.6 1.9"
			}
		}


		"OnSpellStart"
		{
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
				"Function"		"B29W_OnSpellStart"
			}
		}

		"Modifiers"
		{
			"modifier_B29W"
			{
				"Duration"	"%stun"
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"OverrideAnimation"	"ACT_DOTA_DISABLED"
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"IsBuff"	"0"
				"IsDebuff"	"1"
			}
		}
	}
```

### B29E：震地斧（單位技能欄 2）

- 位置：`npc/B_Unified/B29.txt:86`，raw_sha256 `0453974eb5f925dba90f2967473d01f0ffed90bd658c9ffb7fc1ee37f5c198d0`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_B29E_Description，raw_sha256 `6e4ef2665d396ccca0219f93c8a454ff061f5962fa66681e86b551a29f7de9a6`）：

  > 向指定方向跳躍最多300距離並猛擊地面，對附近500的敵軍造成200傷害並減緩(18/22/26/30)% 移動速度 ，並且在接下來8秒內獲得破軍靈光，使附近敵軍護甲降低(4/7/10/13)。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29E_OnSpellStart`, `scripts/vscripts/library/common/dummy.lua#Slow`, `scripts/vscripts/library/common/dummy.lua#ReturnSpeed`

```text
"B29E"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"			"ability_datadriven"
		"AbilityTextureName"	"custom/icon_128x128_98"
		"AbilityType"					"DOTA_ABILITY_TYPE_BASIC"

		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_DIRECTIONAL | DOTA_ABILITY_BEHAVIOR_AOE"
		"AbilityUnitTargetTeam"			"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetType"			"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitDamageType"			"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_NO"

		"AbilityCooldown"				"20 18 16 14"
		"AbilityManaCost"				"105 125 145 165"
		"AbilityDamage"					"200"

		"AOERadius"						"500"
		"AbilityCastPoint"				"0.15"

		"AbilitySpecial"
		{
			"01"
			{
				"var_type"		"FIELD_INTEGER"
				"armor_reduction"	"-4 -7 -10 -13"
			}
			"02"
			{
				"var_type"		"FIELD_INTEGER"
				"debuff_movement_speed"	"-18 -22 -26 -30"
			}
			"03"
			{
				"var_type"	"FIELD_FLOAT"
				"radius"	"800"
			}
			"04"
			{
				"var_type"	"FIELD_INTEGER"
				"max_blink_range"	"300"
			}
		}

		"OnAbilityPhaseStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"B29E_old.vo1"
			}
		}

		"OnSpellStart"
		{
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
				"Function"		"B29E_OnSpellStart"
				"Target"	"POINT"
				"MaxBlinkRange"	"%max_blink_range"
			}
			"DelayedAction"
			{
				"Delay"		"0.25"
				"Action"
				{
					"FireEffect"
					{
						"Target"	"CASTER"
						"EffectName"	"particles/b26w3/b26w3.vpcf"
						"EffectAttachType"	"follow_hitloc"
					}
				}
			}
			
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"Hero_Brewmaster.ThunderClap"
			}
			"ApplyModifier"
			{
				"Target"
				{
				    "Center" "CASTER"
				    "Flags" "DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES | DOTA_UNIT_TARGET_FLAG_INVULNERABLE"
				}
				"ModifierName"	"modifier_B29E_aura"					
			}
		}

		"Modifiers"
		{
			"modifier_B29E_debuff"
			{
				"IsDebuff"	"1"
				"Duration"	"8"
				"OnCreated"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/library/common/dummy.lua"
						"Function"	"Slow"
						"name"	"B29E"
						"ms_slow" "%debuff_movement_speed"
					}
				}
				"OnDestroy"
				{
					"RunScript"
					{
						"ScriptFile"	"scripts/vscripts/library/common/dummy.lua"
						"Function"	"ReturnSpeed"
						"name"	"B29E"
					}
				}
				"EffectName"	"particles/generic_gameplay/move_slow.vpcf"
				"EffectAttachType"	"follow_overhead"
			}
			"modifier_B29E_aura" 																//零氣
			{
				"Passive"			"0"
				"IsHidden"			"0"
				"IsDebuff"			"0"
				"EffectName"		"particles/a01/a01t.vpcf"
				"EffectAttachType"	"follow_origin"
				"Duration"	"8"
				"ThinkInterval"  "0.5"
				"OnCreated"
				{
					"ActOnTargets"
					{
						"Target"
						{
							"Center"  	"CASTER"
							"Radius" 	"%radius"
							"Teams" 	"DOTA_UNIT_TARGET_TEAM_ENEMY"
							"Types" 	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
						}
					
						"Action"    
						{
							"ApplyModifier"
							{
								"ModifierName"	"modifier_B29E"
								"Target" 		"TARGET"
								"Duration"		"1"
							}
						}
					}
				}
				"OnIntervalThink"
				{
					"ActOnTargets"
					{
						"Target"
						{
							"Center"  	"CASTER"
							"Radius" 	"%radius"
							"Teams" 	"DOTA_UNIT_TARGET_TEAM_ENEMY"
							"Types" 	"DOTA_UNIT_TARGET_HERO"
						}
					
						"Action"    
						{
							"ApplyModifier"
							{
								"ModifierName"	"modifier_B29E"
								"Target" 		"TARGET"
								"Duration"		"1"
							}
						}
					}
					"ActOnTargets"
					{
						"Target"
						{
							"Center"  	"CASTER"
							"Radius" 	"%radius"
							"Teams" 	"DOTA_UNIT_TARGET_TEAM_ENEMY"
							"Types" 	"DOTA_UNIT_TARGET_BASIC"
						}
					
						"Action"    
						{
							"ApplyModifier"
							{
								"ModifierName"	"modifier_B29E"
								"Target" 		"TARGET"
								"Duration"		"1"
							}
						}
					}																
				}
			}
			"modifier_B29E"
			{
				"Passive"	"0"
				"EffectName"		"particles/a01/a01t_ring.vpcf"
				"EffectAttachType"	"follow_origin"
				"Properties"
				{					
					"MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" "%armor_reduction"
				}
			}
			"modifier_B29E2"
			{
				"Passive"	"0"
				"EffectName"		"particles/a01/a01t_ring.vpcf"
				"EffectAttachType"	"follow_origin"
				"Properties"
				{					
					"MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" "%armor_reduction"				//-------------零氣
				}
			}
		}
	}
```

### B29R：士氣提升（單位技能欄 3）

- 位置：`npc/B_Unified/B29.txt:306`，raw_sha256 `506375dfc061a33a2a6d40ee6de50b34c1c26cb5673f5b1c5347b27814b7e93f`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_B29R_Description，raw_sha256 `f01392a1a8c220f87f1120a0ff7de5e9ff86217f85ead5b29ef2540fcf13fd10`）：

  > 增加身旁友軍(7/14/21/28)% 攻擊傷害。永久增加自身(4/8/12/16)力量。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29R_OnSpellStart`

```text
"B29R"			//可能要給小兵光環特效 但是暗處gank會被人看到特效
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/icon_128x128_56"
		"AbilityBehavior"	" DOTA_ABILITY_BEHAVIOR_PASSIVE"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_FRIENDLY"
		"AOERadius"	"900"
		"precache"
		{
			"particle"	"particles/b29/b29r.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"Hero_AbyssalUnderlord.Firestorm.Start"
				//sounds\ambient\horn_radiant.vsnd
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
				"Function"		"B29R_OnSpellStart"
			}
			"ApplyModifier"
			{
				"Target"	"CASTER"
				"ModifierName"	"modifier_B29R"
			}			
		}
		"Modifiers"
		{				

			"modifier_B29R_passive"	//靈氣
			{
				"Passive"			"0"
				"IsHidden"			"0"
				"IsDebuff"			"0"
				"EffectName"	"particles/b29/b29r.vpcf"
				"EffectAttachType"	"follow_origin"
				"Properties"
				{
					"MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE"	"%bonus_atk_percent"
				}
			}
			"modifier_B29R_aura"
			{
				"IsHidden"	"1"
				//"Attributes"	"MODIFIER_ATTRIBUTE_MULTIPLE"
				"Passive"	"1"
				"Aura"          	"modifier_B29R_passive"
				"Aura_Radius"   	"600"
				"Aura_Teams"    	"DOTA_UNIT_TARGET_TEAM_FRIENDLY"
				"Aura_Types"    	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
				"Aura_Flags"    	"DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES"
				"Aura_ApplyToCaster" "1"
			}
			"modifier_B29R_str" //+str
			{
				"Passive"			"1"			
				"Properties"
				{
				"MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" "%add_str"
				}
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"					"FIELD_INTEGER"
				"bonus_atk_percent"			"7 14 21 28"
			}
			"02"
			{
				"var_type"			"FIELD_INTEGER"
				"add_str"			"4 8 12 16"
			}
		}
	}
```

### B29_precache（單位技能欄 5）

- 位置：`npc/B_Unified/B29.txt:4`，raw_sha256 `48adc3dfa1ad35e168094746a24880970acdc6ff906bd758a4be024ec3e1a2d5`
- 版本：current
- tooltip：無

```text
"B29_precache"
	{
		"BaseClass"             		"ability_datadriven"
		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_HIDDEN"
		"precache"
	{
			"particle"  "particles/b03w/b03w.vpcf" //w
			"particle"  "particles/b03w/b03w2.vpcf" //w
			"particle"  "particles/b03e/b03e.vpcf" //e
			"particle"  "particles/a17/a17tecon/items/sniper/sniper_charlie/sniper_assassinate_impact_blood_charlie.vpcf" //r
			"particle"  "particles/b03t/b03t.vpcf" //t
			"particle"  "particles/b03t/b03t2.vpcf" //t
			"particle"  "particles/b03w_old/b03w_old.vpcf" //w_old
			"particle"  "particles/b03t_old/b03t_oldfallback_mid.vpcf" //t_old
			"particle"  "particles/b29/b29tattack.vpcf" //t_old
			"soundfile" "soundevents/game_sounds_heroes/game_sounds_chaos_knight.vsndevts" //t
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_leshrac.vsndevts" //w
			"soundfile"	"soundevents/game_sounds_heroes/game_sounds_elder_titan.vsndevts"
		}
	}
```

### B29T：大地震撼（單位技能欄 6）

- 位置：`npc/B_Unified/B29.txt:389`，raw_sha256 `d4d33dc77fa7fae20a4bdbca468a0cf9e0f8363c40e30e5cd98ad8df78df1836`
- 版本：current
- tooltip（DOTA_Tooltip_Ability_B29T_Description，raw_sha256 `3018f995653bd8eba0a3057be9dbb4be57c3294cc6ca1f36350465d8ebf59508`）：

  > 在指定的範圍內每0.1秒打出一個震波，每個震波將會使範圍340的敵軍受到60傷害並暈眩(0.2/0.3/0.4)秒。最多可持續(50/60/70)波 。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/b29.lua#B29T_OnAbilityPhaseStart`, `scripts/vscripts/heroes/B_Unified/b29.lua#B29T`

```text
"B29T"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/B03/B03R"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_AOE | DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_CHANNELLED"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_YES"
		"AbilityType"	"DOTA_ABILITY_TYPE_ULTIMATE"
		"HotKeyOverride"	"T"
		"MaxLevel"	"3"
		"RequiredLevel"	"6"
		"LevelsBetweenUpgrades"	"6"
		"AbilityCastPoint"	"0.3"
		"AbilityCooldown"	"100"
		"AbilityManaCost"	"200 225 250"
		"AbilityCastRange"	"1100"
		"AbilityDamage"		"60"
		"AbilityChannelTime"	"%B29T_Skill_Duration"
		"AOERadius"	"%B29T_Radius"
		"precache"
		{
			"soundfile"	"soundevents/C01T.vsndevts"
			"particle"	"particles/generic_gameplay/generic_stunned.vpcf"
		}
		"OnAbilityPhaseStart"
		{
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/b29.lua"
				"Function"	"B29T_OnAbilityPhaseStart"
			}
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C01T.vo1"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/b29.lua"
				"Function"	"B29T"
				"Target"	"POINT"
			}
		}
		"Modifiers"
		{
			"modifier_B29T"
			{
				"IsHidden"	"0"
				"Attributes"	"MODIFIER_ATTRIBUTE_MULTIPLE"
				"Duration"	"%B29T_Duration"
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"IsDebuff"	"1"
				"OverrideAnimation"	"ACT_DOTA_DISABLED"
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_Radius"	"700 800 900"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_Duration"	"0.2 0.3 0.4"
			}
			"03"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_Skill_Duration"	"4.5 5.2 5.9"
			}
			"04"
			{
				"var_type"	"FIELD_INTEGER"
				"B29T_Amount"	"50 60 70"
			}
		}
	}
```

### B29D_old：武王丸（不在單位技能清單）

- 位置：`npc/B_Unified/B29.txt:481`，raw_sha256 `0794372c23fb0641a34ac1a761beaba572cad2fb0409363cdced01675867ce36`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_B29D_old_Description，raw_sha256 `726ddcaeecbf211e38bb1162c443b0c067de9a4d7163cb546fe7aef692371d3f`）：

  > 增加40點攻擊力，30%攻擊速度， 持續30秒。

```text
"B29D_old"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/zhangliao_cizhenkuijia"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_IMMEDIATE | DOTA_ABILITY_BEHAVIOR_NO_TARGET"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO"
		"FightRecapLevel"	"0"
		"AbilityType"	"DOTA_ABILITY_TYPE_BASIC"
		"HotKeyOverride"	"D"
		"MaxLevel"	"1"
		"AbilityCooldown"	"40"
		"AbilityManaCost"	"50"
		// Stats
		// -------------------------------------------------------------------------------------------------------------
		"AbilityCastRange"	"0"
		"AbilityCastRangeBuffer"	"250"
		"precache"
		{
			"particle"	"particles/econ/courier/courier_greevil_red/courier_greevil_red_ambient_3.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"A06W.vo1"
			}
			"ApplyModifier"
			{
				"Target"
				{
				    "Center" "CASTER"
				    "Flags" "DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES | DOTA_UNIT_TARGET_FLAG_INVULNERABLE"
				}
				"ModifierName"	"modifier_B29D_old"
				"Duration"	"%B29D_old_Duration"
			}
		}
		"Modifiers"
		{
			"modifier_B29D_old"
			{
				"IsBuff"	"1"
				"Properties"
				{
					"MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT"	"%B29D_old_speed"
					"MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE"	"%atk"
				}
				"States"	{}
				"Attributes"	"MODIFIER_ATTRIBUTE_MULTIPLE"
				"IsPurgable"	"1"
				"EffectName" "particles/econ/courier/courier_greevil_red/courier_greevil_red_ambient_3.vpcf"
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"B29D_old_speed"	"30"
			}
			"02"
			{
				"var_type"	"FIELD_INTEGER"
				"atk"	"40"
			}
			"03"
			{
				"var_type"	"FIELD_INTEGER"
				"B29D_old_Duration"	"30"
			}
		}
	}
```

### B29W_old：動地斧（不在單位技能清單）

- 位置：`npc/B_Unified/B29.txt:556`，raw_sha256 `67b930f01052c6de6ce0494a615d46b14c447125d5bebbaabb52cb78112992fd`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_B29W_old_Description，raw_sha256 `98dec60690c47bae31e4841b6dd8691b873f806337e37c3ec3cc81310ab58a51`）：

  > 猛擊地面，對附近400範圍的敵軍造成(150/250/350/450)傷害並且震昏(1.0/1.5/2.0/2.5)秒。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29W_old_OnSpellStart`

```text
"B29W_old"	//-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------
	{
		"BaseClass"             		"ability_datadriven"
		"AbilityTextureName"			"custom/icon_128x128_25"
		"AbilityType"					"DOTA_ABILITY_TYPE_BASIC"
		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_NO_TARGET"
		"AbilityUnitTargetType"			"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"			"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitDamageType"			"DAMAGE_TYPE_MAGICAL"
		"AbilityCastPoint" "0.25" //stun
		"AbilityCooldown"				"16 14 12 10"
		"AbilityManaCost"				"100 120 140 160"
		"AbilityCastAnimation"			"ACT_DOTA_CAST_ABILITY_2"
		"AOERadius" 					"400"
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"		"FIELD_INTEGER"
				"damage"	"150 250 350 450"
			}
			"02"
			{
				"var_type"		"FIELD_FLOAT"
				"stun"			"1 1.5 2 2.5"
			}
		}


		"OnSpellStart"
		{
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
				"Function"		"B29W_old_OnSpellStart"
			}
		}

		"Modifiers"
		{
			"modifier_B29W_old"
			{
				"Duration"	"%stun"
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"OverrideAnimation"	"ACT_DOTA_DISABLED"
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"IsBuff"	"0"
				"IsDebuff"	"1"
			}
		}
	}
```

### B29E_old：震地斧（不在單位技能清單）

- 位置：`npc/B_Unified/B29.txt:612`，raw_sha256 `f678c8427ea0ba2b0a00a79a92c9aebb0e34329d35362eaece56846e589aeeec`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_B29E_old_Description，raw_sha256 `1bd929e6e8073265726afc26613ab5a8785813bb0d96bdfe9b118682445790b9`）：

  > 敲擊地面，對附近400範圍的敵軍造成(90/180/270/360)傷害，並減緩(20/30/40/50)% 攻擊速度及移動速度。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29E_old_OnSpellStart`

```text
"B29E_old"
	{
		// General
		// -------------------------------------------------------------------------------------------------------------
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/icon_128x128_98"
		"AbilityType"					"DOTA_ABILITY_TYPE_BASIC"

		"AbilityBehavior"				"DOTA_ABILITY_BEHAVIOR_NO_TARGET | DOTA_ABILITY_BEHAVIOR_AOE"
		"AbilityUnitTargetTeam"			"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitTargetType"			"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitDamageType"			"DAMAGE_TYPE_MAGICAL"

		"AbilityCooldown"				"14 12 10 8"
		"AbilityManaCost"				"100 120 140 160"
		"AbilityDamage"					"90 180 270 360"

		"AbilityCastRange"				"400"
		"AOERadius"						"400"
		"AbilityCastPoint"				"0.1"

		"AbilitySpecial"
		{
			"01"
			{
				"var_type"		"FIELD_INTEGER"
				"debuff_attack_speed"	"-20 -30 -40 -50"
			}
			"02"
			{
				"var_type"		"FIELD_INTEGER"
				"debuff_movement_speed"	"-20 -30 -40 -50"
			}
		}

		"OnAbilityPhaseStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C03E.vo1"
			}
		}

		"OnSpellStart"
		{
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
				"Function"		"B29E_old_OnSpellStart"
			}
			"FireEffect"
			{
				"Target"	"CASTER"
				"EffectName"	"particles/b26w3/b26w3.vpcf"
				"EffectAttachType"	"follow_hitloc"
			}
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"Hero_Brewmaster.ThunderClap"
			}
		}

		"Modifiers"
		{
			"modifier_B29E_old_debuff"
			{
				"IsDebuff"	"1"
				"Duration"	"6"
				"Properties"
				{
					"MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" "%debuff_attack_speed"
				    "MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE" "%debuff_movement_speed" 
				}
				"EffectName"	"particles/generic_gameplay/move_slow.vpcf"
				"EffectAttachType"	"follow_overhead"
			}
		}	
	}
```

### B29R_old：士氣提升（不在單位技能清單）

- 位置：`npc/B_Unified/B29.txt:693`，raw_sha256 `39d512407719581d227d7b90872d1d9e696273c3e767b6ee9fa5bb16aaffc612`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_B29R_old_Description，raw_sha256 `b8d5f65067870373d6ea8a066ed012f62a6f20650a8cd6b702240cad5bde6d3a`）：

  > 使900範圍內的友軍擁有(1.1/1.2/1.3/1.4)倍爆擊。
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/B29.lua#B29R_OnAttackStart`

```text
"B29R_old"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/juggernaut_blade_dance2"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_PASSIVE"
		"HotKeyOverride"	"R"
		"AbilityCastRange"	"900"
		"Modifiers"
		{
			"modifier_A18R_old_aura"
			{
				"Passive"	"1"
				"IsHidden"	"1"
				"Aura"          	"modifier_A18R_old"
				"Aura_Radius"   	"900"
				"Aura_Teams"    	"DOTA_UNIT_TARGET_TEAM_FRIENDLY"
				"Aura_Types"    	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
				"Aura_Flags"    	"DOTA_UNIT_TARGET_FLAG_NONE"
				"Aura_ApplyToCaster" "1"
				"EffectName"		"particles/a18/a18r_old.vpcf"
				"EffectAttachType"	"follow_origin"
			}
			"modifier_A18R_old"
			{
				"Passive"	"0"
				"IsHidden"	"0"
				"Attributes"		"MODIFIER_ATTRIBUTE_IGNORE_INVULNERABLE"
				"EffectName"		"particles/a18/a18r_old_ring.vpcf"
				"EffectAttachType"	"follow_origin"
				"OnAttackStart"
				{
					"RunScript"
					{
						"Target"	"CASTER"
						"ScriptFile"	"scripts/vscripts/heroes/B_Unified/B29.lua"
						"Function"	"B29R_OnAttackStart"
					}
				}
			}
			"modifier_A18R_critical_strike"
			{
				"Passive"	"0"
				"IsHidden"	"1"
				"IsBuff"	"0"
				"Attributes"		"MODIFIER_ATTRIBUTE_IGNORE_INVULNERABLE"
				"Properties"
				{
					"MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE"	"%crit_bonus"
				}
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_INTEGER"
				"crit_bonus"	"110 120 130 140"
			}
			"02"
			{
				"var_type"	"FIELD_INTEGER"
				"crit_chance" "100" // %
			}
		}
	}
```

### B29T_old：大地震撼（不在單位技能清單）

- 位置：`npc/B_Unified/B29.txt:759`，raw_sha256 `f1311972a3e6bfa198b7d6c5099a9e17104bdf4520f782dbeee97f607b4efbb2`
- 版本：old
- tooltip（DOTA_Tooltip_Ability_B29T_old_Description，raw_sha256 `b8346f0527f0f1cb89783934e0767e3281801040762fe62fc82ee51b5d0647b7`）：

  > 在範圍700內每(0.1/0.08/0.06)秒打出兩波動地跺，每個震波對範圍240內所有敵人45點損傷，並擊暈0.2秒。需持續施法，最多可持續(65/90/115)波。並且獲得大地之力而增加力量(25/35/45)點持續100秒
- 引用但來源 repo 沒有的 vscript：`scripts/vscripts/heroes/B_Unified/b29.lua#B29T_old`

```text
"B29T_old"
	{
		"BaseClass"	"ability_datadriven"
		"AbilityTextureName"	"custom/B03/B03R"
		"AbilityBehavior"	"DOTA_ABILITY_BEHAVIOR_AOE | DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_CHANNELLED"
		"AbilityUnitTargetType"	"DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC"
		"AbilityUnitTargetTeam"	"DOTA_UNIT_TARGET_TEAM_ENEMY"
		"AbilityUnitDamageType"	"DAMAGE_TYPE_MAGICAL"
		"SpellImmunityType"	"SPELL_IMMUNITY_ENEMIES_YES"
		"AbilityType"	"DOTA_ABILITY_TYPE_ULTIMATE"
		"HotKeyOverride"	"T"
		"MaxLevel"	"3"
		"RequiredLevel"	"6"
		"LevelsBetweenUpgrades"	"6"
		"AbilityCastPoint"	"0"
		"AbilityCooldown"	"100"
		"AbilityManaCost"	"250 300 350"
		"AbilityCastRange"	"1100"
		"AbilityDamage"		"45"
		"AbilityChannelTime"	"%B29T_old_Skill_Duration"
		"AOERadius"	"%B29T_old_Radius"
		"precache"
		{
			"soundfile"	"soundevents/C01T.vsndevts"
			"particle"	"particles/generic_gameplay/generic_stunned.vpcf"
		}
		"OnSpellStart"
		{
			"FireSound"
			{
				"Target"	"CASTER"
				"EffectName"	"C01T.vo1"
			}
			"RunScript"
			{
				"ScriptFile"	"scripts/vscripts/heroes/B_Unified/b29.lua"
				"Function"	"B29T_old"
				"Target"	"POINT"
			}
			"ApplyModifier"
			{
				"Target"	"CASTER"
				"ModifierName"	"modifier_B29T_old_str"
			}
		}
		"Modifiers"
		{
			"modifier_B29T_old"
			{
				"IsHidden"	"0"
				"Attributes"	"MODIFIER_ATTRIBUTE_MULTIPLE"
				"Duration"	"%B29T_old_Duration"
				"States"
				{
					"MODIFIER_STATE_STUNNED"	"MODIFIER_STATE_VALUE_ENABLED"
				}
				"EffectName"	"particles/generic_gameplay/generic_stunned.vpcf"
				"EffectAttachType"	"follow_overhead"
				"IsDebuff"	"1"
				"OverrideAnimation"	"ACT_DOTA_DISABLED"
			}
			"modifier_B29T_old_str"
			{
				"IsHidden"	"0"
				"Attributes"	"MODIFIER_ATTRIBUTE_MULTIPLE"
				"Duration"	"100"
				"Properties"
				{
				"MODIFIER_PROPERTY_STATS_STRENGTH_BONUS"	"%B29T_old_str"
				}
				"IsBuff"	"1"
			}
		}
		"AbilitySpecial"
		{
			"01"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_old_Radius"	"700"
			}
			"02"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_old_Duration"	"0.2"
			}
			"03"
			{
				"var_type"	"FIELD_FLOAT"
				"B29T_old_Skill_Duration"	"4.5 5.2 5.9"
			}
			"04"
			{
				"var_type"	"FIELD_INTEGER"
				"B29T_old_Amount"	"50 60 70"
			}
			"05"
			{
				"var_type"	"FIELD_INTEGER"
				"B29T_old_str"	"25 35 45"
			}
			"06"
			{
				"var_type"	"FIELD_FLOAT"
				"interval"	"0.1 0.08 0.06"
			}
		}
	}
```
