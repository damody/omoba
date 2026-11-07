-- Coarse ability-family tagging (plan D2). Rules only record evidence that is
-- literally present in KV fields or the zh-TW tooltip; nothing is inferred
-- from hero names or numbering. An ability with no evidence is `unclassified`.
local M = {}

-- Families follow README §5 (技能能力交付矩陣) rows.
M.FAMILIES = {
  { id = "damage_heal_shield", label = "指定傷害／治療、資源、護盾" },
  { id = "passive_attack_modifier", label = "普攻強化、被動、暴擊、吸血" },
  { id = "shape_displacement", label = "圓形、扇形、直線、位移" },
  { id = "channel_dot_area", label = "持續引導、DoT、地面區域" },
  { id = "projectile", label = "飛行直線／追蹤投射物" },
  { id = "multi_stage_toggle", label = "多段／重施放／蓄力／切換" },
  { id = "summon", label = "召喚物" },
  { id = "stealth_detect_control", label = "隱形／偵測／控制" },
  { id = "push_pull_knockup", label = "推拉、擊飛、衝鋒、牆體互動" },
  { id = "clone_multiform", label = "分身、多形態／多技能組" },
  { id = "copy_rewind_terrain", label = "複製／偷技能、回溯、任意地形改造" },
}

local TAG_FAMILY = {
  unit_target = "damage_heal_shield", damage = "damage_heal_shield", heal = "damage_heal_shield",
  shield = "damage_heal_shield", mana = "damage_heal_shield",
  passive = "passive_attack_modifier", attack_modifier = "passive_attack_modifier",
  crit = "passive_attack_modifier", lifesteal = "passive_attack_modifier",
  circle = "shape_displacement", cone = "shape_displacement", line = "shape_displacement",
  directional = "shape_displacement", displacement = "shape_displacement",
  channel = "channel_dot_area", dot = "channel_dot_area", ground_area = "channel_dot_area",
  aura = "channel_dot_area",
  projectile_linear = "projectile", projectile_tracking = "projectile", projectile_text = "projectile",
  toggle = "multi_stage_toggle", recast = "multi_stage_toggle", charge_up = "multi_stage_toggle",
  mode_switch = "multi_stage_toggle",
  summon = "summon",
  stealth = "stealth_detect_control", detect = "stealth_detect_control", stun = "stealth_detect_control",
  root = "stealth_detect_control", silence = "stealth_detect_control", slow = "stealth_detect_control",
  disarm = "stealth_detect_control", fear_taunt = "stealth_detect_control", freeze = "stealth_detect_control",
  sleep = "stealth_detect_control", hex = "stealth_detect_control", mute = "stealth_detect_control",
  blind = "stealth_detect_control",
  knockback = "push_pull_knockup", pull = "push_pull_knockup", knockup = "push_pull_knockup",
  charge = "push_pull_knockup", wall = "push_pull_knockup",
  illusion = "clone_multiform", transform = "clone_multiform", ability_swap = "clone_multiform",
  steal = "copy_rewind_terrain", rewind = "copy_rewind_terrain", terrain = "copy_rewind_terrain",
}
M.TAG_FAMILY = TAG_FAMILY

local BEHAVIOR_TAGS = {
  PASSIVE = "passive", TOGGLE = "toggle", CHANNELLED = "channel", UNIT_TARGET = "unit_target",
  AOE = "circle", DIRECTIONAL = "directional", AUTOCAST = "attack_modifier", ATTACK = "attack_modifier",
}

-- KV block keys (actions, events, modifier keys) that are direct evidence.
local KEY_TAGS = {
  Damage = "damage", Heal = "heal", LinearProjectile = "projectile_linear",
  TrackingProjectile = "projectile_tracking", Stun = "stun", Knockback = "knockback",
  SpawnUnit = "summon", CreateThinker = "ground_area", CleaveAttack = "attack_modifier",
  Orb = "attack_modifier", OnAttackLanded = "attack_modifier", OnAttackStart = "attack_modifier",
  OnAttack = "attack_modifier", OnToggleOn = "toggle", OnChannelFinish = "channel",
  Blink = "displacement", ApplyMotionController = "displacement", Aura = "aura",
  Lifesteal = "lifesteal", SpendMana = "mana", Purge = "damage",
}

local MODIFIER_TAGS = {
  MODIFIER_STATE_INVISIBLE = "stealth", MODIFIER_PROPERTY_INVISIBILITY_LEVEL = "stealth",
  MODIFIER_STATE_STUNNED = "stun", MODIFIER_STATE_ROOTED = "root", MODIFIER_STATE_SILENCED = "silence",
  MODIFIER_STATE_DISARMED = "disarm", MODIFIER_STATE_HEXED = "hex", MODIFIER_STATE_FROZEN = "freeze",
  MODIFIER_STATE_MUTED = "mute", MODIFIER_STATE_BLIND = "blind",
  MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE = "crit", MODIFIER_PROPERTY_ATTACK_RANGE_BONUS = "attack_modifier",
  MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE = "attack_modifier",
  MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE = "attack_modifier",
  MODIFIER_PROPERTY_MODEL_CHANGE = "transform",
}

-- zh-TW tooltip keywords. Patterns run on the description with markup removed.
local TEXT_RULES = {
  { "summon", { "召喚", "召出", "喚出", "召來" } },
  { "stealth", { "隱形", "隱身", "潛行" } },
  { "detect", { "偵測", "真視", "顯形", "反隱" } },
  { "stun", { "暈眩", "昏迷", "擊暈" } },
  { "root", { "定身", "禁錮", "纏繞", "束縛" } },
  { "silence", { "沉默", "封印" } },
  { "slow", { "緩速", "減速", { "降低", "移動速度" } } },
  { "disarm", { "繳械" } },
  { "fear_taunt", { "恐懼", "嘲諷" } },
  { "freeze", { "冰凍", "凍結" } },
  { "sleep", { "睡眠", "沉睡" } },
  { "knockback", { "擊退", "推開", "彈開" } },
  { "pull", { "拉回", "拉近", "拉向", "吸引", "拉到", "鉤" } },
  { "knockup", { "擊飛", "浮空", "擊起" } },
  { "charge", { "衝鋒", "衝刺", "突進", "衝向", "衝撞" } },
  { "wall", { "牆" } },
  { "displacement", { "跳躍", "瞬移", "閃爍", "位移", "傳送", "瞬間移動", "閃現", "互換位置" } },
  { "heal", { "治療", { "回復", "生命" }, { "恢復", "生命" }, "補血" } },
  { "shield", { "護盾", { "吸收", "傷害" }, "抵擋" } },
  { "dot", { { "每秒", "傷害" }, { "持續", "傷害" } } },
  { "channel", { "持續施法", "引導", "吟唱", "施法期間" } },
  { "circle", { "範圍", "周圍", "半徑", "附近" } },
  { "line", { "直線", "一條線" } },
  { "cone", { "扇形", "錐形" } },
  { "projectile_text", { "投擲", "射出", "發射", "飛出", "彈道", "擲出", "丟出" } },
  { "recast", { "再次施放", "再次使用", "第二段", "第二次", "第三次", "連續施放" } },
  { "mode_switch", { "切換", "模式", "姿態" } },
  { "charge_up", { "蓄力", "集氣" } },
  { "illusion", { "分身", "幻象", "替身" } },
  { "transform", { "變身", "化身", "形態" } },
  { "steal", { "複製", "偷取", "偷來", "竊取" } },
  { "rewind", { "回溯", "時間倒流" } },
  { "terrain", { "地形", "樹木" } },
  { "passive", { "被動" } },
  { "crit", { "暴擊", "爆擊", "致命一擊" } },
  { "lifesteal", { "吸血", "吸取生命" } },
  { "attack_modifier", { "攻擊時", "每次攻擊", "普通攻擊", "普攻", "攻擊命中", { "增加", "攻擊" }, { "提升", "攻擊" }, { "提高", "攻擊" } } },
}

function M.plain_text(s)
  if not s then return nil end
  return (s:gsub("<[^>]->", ""):gsub("\\n", " "):gsub("%s+", " "))
end

-- Splits on full-width/ASCII sentence and clause punctuation. Byte classes
-- cannot hold multi-byte characters, so separators are matched as strings.
local SEPARATORS = { "。", "，", "；", "！", "？", "、", ",", ";" }
function M.clauses(text)
  local marked = text
  for _, sep in ipairs(SEPARATORS) do marked = marked:gsub(sep:gsub("%p", "%%%0"), "\0") end
  local out = {}
  for part in marked:gmatch("[^\0]+") do out[#out + 1] = part end
  return out
end

-- pattern: plain string, or { first, second } that must appear in order
-- within one clause. Returns the matched excerpt or nil.
function M.match_clause(clause, pattern)
  if type(pattern) == "string" then
    return clause:find(pattern, 1, true) and pattern or nil
  end
  local s, e = clause:find(pattern[1], 1, true)
  if not s then return nil end
  local s2, e2 = clause:find(pattern[2], e + 1, true)
  if not s2 then return nil end
  return clause:sub(s, e2)
end

-- facts: { behavior_flags = {...}, block_keys = set, modifier_keys = set, text = string|nil }
function M.classify(facts)
  local tags, order = {}, {}
  local function add(tag, source, match)
    local t = tags[tag]
    if not t then t = { tag = tag, family = TAG_FAMILY[tag], evidence = {} }; tags[tag] = t; order[#order + 1] = tag end
    for _, e in ipairs(t.evidence) do if e.source == source and e.match == match then return end end
    t.evidence[#t.evidence + 1] = { source = source, match = match }
  end
  for _, flag in ipairs(facts.behavior_flags or {}) do
    local tag = BEHAVIOR_TAGS[flag]
    if tag then add(tag, "kv_behavior", "DOTA_ABILITY_BEHAVIOR_" .. flag) end
  end
  -- Sorted so evidence order (and the generated catalog) is byte-stable.
  local function sorted(set)
    local out = {}
    for k in pairs(set or {}) do out[#out + 1] = k end
    table.sort(out)
    return out
  end
  for _, key in ipairs(sorted(facts.block_keys)) do
    if KEY_TAGS[key] then add(KEY_TAGS[key], "kv_block", key) end
  end
  for _, key in ipairs(sorted(facts.modifier_keys)) do
    if MODIFIER_TAGS[key] then add(MODIFIER_TAGS[key], "kv_modifier", key) end
  end
  if facts.has_radius_target then add("circle", "kv_block", "Target.Radius") end
  if facts.dot_modifier then add("dot", "kv_block", "Modifier.ThinkInterval+Damage") end
  if facts.text and facts.text ~= "" then
    local clauses = M.clauses(facts.text)
    for _, rule in ipairs(TEXT_RULES) do
      local hit
      for _, pattern in ipairs(rule[2]) do
        for _, clause in ipairs(clauses) do
          hit = M.match_clause(clause, pattern)
          if hit then break end
        end
        if hit then break end
      end
      if hit then add(rule[1], "tooltip_text", hit) end
    end
  end
  table.sort(order)
  local tag_list, families, seen = {}, {}, {}
  local kv_hit, text_hit = false, false
  for _, tag in ipairs(order) do
    local t = tags[tag]
    tag_list[#tag_list + 1] = t
    for _, e in ipairs(t.evidence) do
      if e.source == "tooltip_text" then text_hit = true else kv_hit = true end
    end
    if not seen[t.family] then seen[t.family] = true; families[#families + 1] = t.family end
  end
  table.sort(families)
  local status = #families > 0 and "classified" or "unclassified"
  local basis = (kv_hit and text_hit) and "kv+text" or kv_hit and "kv_only" or text_hit and "text_only" or "none"
  return { status = status, basis = basis, families = families, tags = tag_list }
end

return M
