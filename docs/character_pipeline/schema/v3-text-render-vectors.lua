-- Shared test vectors for rendering v3 text templates (v3.md §9, value-single-source §3.1).
--
-- The website generator (scripts/site/text_render.lua, checked by
-- scripts/tests/site_text_render_test.lua) and UE's TooltipRenderer must turn every
-- `template` + input below into exactly `expect.runs`. When a rule changes, change it
-- here first; both implementations follow.
--
-- Input
--   values  name -> { rank values..., unit, damage_type?, per? }. These are catalog
--           values (v3-content-model-proposal.md §2): exact decimals or fractions as
--           strings, the displayed number; pct is a percent number ("25" is 25%).
--   owners  owner id -> values, for {owner.name}.
--   rules   nested table, for {rules.a.b}.  names  id -> name, for {@id}.
--   rank    current ability rank (nil: none).  hero_level  for per = "hero_level".
-- Output
--   runs    { t = text, b = true (current rank), dt = damage type }; a missing field
--           means false/nil. Adjacent runs with the same b and dt are merged; empty
--           runs are dropped.
--   error   the template must be rejected (the text says why; not compared).
--
-- Rules
--   1. A number is shown as its exact value, rounded half away from zero to 2
--      decimals, trailing zeros removed, never "-0". A pct placeholder gets one "%"
--      after the whole value ("5/6/8/11%"), unstyled except for the damage type.
--   2. Default format `all`: every rank joined by "/". If every rank shows the same
--      text, show it once. With a rank, that rank (clamped to 1..count) is bold.
--   3. `current`: only the current rank, not bold; without a rank, same as `all`.
--      `max`: the last rank. `abs`: absolute value. `int`: round to an integer
--      (half away from zero) before display. `signed`: "+" before values above 0.
--   4. per = "hero_level": the bold index follows hero_level, not rank.
--   5. A value with damage_type gives the whole placeholder (and its "/") dt.
--   6. {rules.a.b}: the rules number with rule 1, no unit. {@id}: the name.
--   7. Literal text is kept as is, including newlines.
return {
  version = 2,
  cases = {
    { name = "single_value", lang = "zh_TW", template = "造成 {damage} 傷害。",
      values = { damage = { "100", unit = "dmg" } },
      expect = { runs = { { t = "造成 100 傷害。" } } } },

    { name = "ranks_without_current_rank", lang = "zh_TW", template = "造成 {damage} 魔法傷害",
      values = { damage = { "350", "520", "690", "860", unit = "dmg", damage_type = "magical" } },
      expect = { runs = { { t = "造成 " }, { t = "350/520/690/860", dt = "magical" }, { t = " 魔法傷害" } } } },

    { name = "current_rank_bold", lang = "zh_TW", template = "造成 {damage} 魔法傷害", rank = 2,
      values = { damage = { "350", "520", "690", "860", unit = "dmg", damage_type = "magical" } },
      expect = { runs = { { t = "造成 " }, { t = "350/", dt = "magical" }, { t = "520", b = true, dt = "magical" },
        { t = "/690/860", dt = "magical" }, { t = " 魔法傷害" } } } },

    { name = "rank_clamped_to_count", lang = "zh_TW", template = "暈眩 {stun} 秒", rank = 9,
      values = { stun = { "1", "1.5", "2", "2.5", unit = "s" } },
      expect = { runs = { { t = "暈眩 1/1.5/2/" }, { t = "2.5", b = true }, { t = " 秒" } } } },

    { name = "equal_ranks_collapse", lang = "zh_TW", template = "持續 {duration} 秒", rank = 2,
      values = { duration = { "3", "3", "3", "3", unit = "s" } },
      expect = { runs = { { t = "持續 3 秒" } } } },

    { name = "single_value_ignores_rank", lang = "en", template = "lasts {duration} seconds", rank = 3,
      values = { duration = { "16", unit = "s" } },
      expect = { runs = { { t = "lasts 16 seconds" } } } },

    { name = "decimal_rounding", lang = "zh_TW", template = "{a} {b} {c} {d} {e} {f} {g}",
      values = { a = { "0.35", unit = "s" }, b = { "1/3", unit = "s" }, c = { "2/3", unit = "s" },
        d = { "0.125", unit = "s" }, e = { "-0.125", unit = "number" }, f = { "0.004", unit = "s" },
        g = { "-0.004", unit = "number" } },
      expect = { runs = { { t = "0.35 0.33 0.67 0.13 -0.13 0 0" } } } },

    { name = "pct_is_a_percent_number", lang = "zh_TW", template = "{chance} 機率造成 {mult} 傷害；{p} 與 {half}",
      values = { chance = { "25", unit = "pct" }, mult = { "185", unit = "pct" }, p = { "1900/101", unit = "pct" },
        half = { "7.5", unit = "pct" } },
      expect = { runs = { { t = "25% 機率造成 185% 傷害；18.81% 與 7.5%" } } } },

    { name = "pct_ranks_one_suffix", lang = "zh_TW", template = "{bonus}",
      values = { bonus = { "5", "6", "8", "11", unit = "pct" } },
      expect = { runs = { { t = "5/6/8/11%" } } } },

    { name = "pct_ranks_first_bold", lang = "zh_TW", template = "{bonus}", rank = 1,
      values = { bonus = { "7.5", "15", "22.5", "30", unit = "pct" } },
      expect = { runs = { { t = "7.5", b = true }, { t = "/15/22.5/30%" } } } },

    { name = "pct_last_rank_bold_suffix_plain", lang = "zh_TW", template = "{bonus}", rank = 4,
      values = { bonus = { "5", "6", "8", "11", unit = "pct", damage_type = "magical" } },
      expect = { runs = { { t = "5/6/8/", dt = "magical" }, { t = "11", b = true, dt = "magical" }, { t = "%", dt = "magical" } } } },

    { name = "pct_current_rank", lang = "zh_TW", template = "{bonus:current}", rank = 2,
      values = { bonus = { "5", "6", "8", "11", unit = "pct" } },
      expect = { runs = { { t = "6%" } } } },

    { name = "format_abs", lang = "zh_TW", template = "降低 {guard:abs}",
      values = { guard = { "-30", "-40", "-50", "-60", unit = "pct" } },
      expect = { runs = { { t = "降低 30/40/50/60%" } } } },

    { name = "format_signed", lang = "zh_TW", template = "{a:signed} {b:signed} {c:signed} {d:signed}",
      values = { a = { "20", unit = "armor" }, b = { "-5", unit = "armor" }, c = { "0", unit = "armor" },
        d = { "10", unit = "pct" } },
      expect = { runs = { { t = "+20 -5 0 +10%" } } } },

    { name = "format_int", lang = "zh_TW", template = "{x:int}",
      values = { x = { "2.5", "3.49", "-2.5", unit = "number" } },
      expect = { runs = { { t = "3/3/-3" } } } },

    { name = "format_int_collapses", lang = "zh_TW", template = "{x:int}", rank = 1,
      values = { x = { "2.6", "3.4", unit = "count" } },
      expect = { runs = { { t = "3" } } } },

    { name = "format_current_with_rank", lang = "zh_TW", template = "{d:current}", rank = 3,
      values = { d = { "100", "200", "300", unit = "dmg", damage_type = "physical" } },
      expect = { runs = { { t = "300", dt = "physical" } } } },

    { name = "format_current_without_rank", lang = "zh_TW", template = "{d:current}",
      values = { d = { "100", "200", "300", unit = "dmg" } },
      expect = { runs = { { t = "100/200/300" } } } },

    { name = "format_max", lang = "zh_TW", template = "最多 {d:max}", rank = 1,
      values = { d = { "100", "200", "300", unit = "dmg", damage_type = "pure" } },
      expect = { runs = { { t = "最多 " }, { t = "300", dt = "pure" } } } },

    { name = "per_hero_level_follows_hero_level", lang = "zh_TW", template = "{bonus}", rank = 1, hero_level = 3,
      values = { bonus = { "1", "2", "3", unit = "stat", per = "hero_level" } },
      expect = { runs = { { t = "1/2/" }, { t = "3", b = true } } } },

    { name = "per_hero_level_without_level", lang = "zh_TW", template = "{bonus}", rank = 2,
      values = { bonus = { "1", "2", "3", unit = "stat", per = "hero_level" } },
      expect = { runs = { { t = "1/2/3" } } } },

    { name = "rules_value", lang = "zh_TW", template = "基礎魔抗 {rules.hero.base_magic_resistance}",
      rules = { hero = { base_magic_resistance = "30" } },
      expect = { runs = { { t = "基礎魔抗 30" } } } },

    { name = "name_reference", lang = "zh_TW", template = "每層{@c01_blood_flame}",
      names = { c01_blood_flame = "血火共刃" },
      expect = { runs = { { t = "每層血火共刃" } } } },

    { name = "qualified_owner_value", lang = "en", template = "illusions deal {b11_crow_strike.illusion_rate}", rank = 2,
      owners = { b11_crow_strike = { illusion_rate = { "30", unit = "pct" } } },
      expect = { runs = { { t = "illusions deal 30%" } } } },

    { name = "english_rank_four", lang = "en", template = "Deals {damage} magic damage.", rank = 4,
      values = { damage = { "100", "200", "300", "400", unit = "dmg", damage_type = "magical" } },
      expect = { runs = { { t = "Deals " }, { t = "100/200/300/", dt = "magical" }, { t = "400", b = true, dt = "magical" },
        { t = " magic damage." } } } },

    { name = "newline_kept", lang = "zh_TW", template = "第一行\n{x} 秒",
      values = { x = { "1.5", unit = "s" } },
      expect = { runs = { { t = "第一行\n1.5 秒" } } } },

    { name = "adjacent_placeholders", lang = "zh_TW", template = "{a}{b}", rank = 1,
      values = { a = { "1", "2", unit = "s" }, b = { "5", unit = "s", damage_type = "magical" } },
      expect = { runs = { { t = "1", b = true }, { t = "/2" }, { t = "5", dt = "magical" } } } },

    { name = "error_missing_value", lang = "zh_TW", template = "{nope}", values = {}, expect = { error = "no value" } },
    { name = "error_unknown_format", lang = "zh_TW", template = "{a:bold}",
      values = { a = { "1", unit = "s" } }, expect = { error = "unknown format" } },
    { name = "error_unbalanced_brace", lang = "zh_TW", template = "a {b", values = {}, expect = { error = "unbalanced" } },
    { name = "error_missing_name", lang = "zh_TW", template = "{@ghost}", names = {}, expect = { error = "no name" } },
    { name = "error_missing_rule", lang = "zh_TW", template = "{rules.hero.nope}", rules = { hero = {} },
      expect = { error = "not a rules value" } },
  },
}
