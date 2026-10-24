# Native attack stat coherence

Native lane/camp DamageInstance processing hardcodes evasion to zero and ignores attacker miss modifiers. It also omits CProperty-only bodies from defense lookup. Reuse UnitStats for native attacks, preserving Unit and Hero base-stat precedence, legacy hero critical chance and existing ability flags. No new content, hero-name special case or network-match claim.
