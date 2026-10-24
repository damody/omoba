# Design

Resolve a checked authored rank, strict rank JSON and a declarative list of modifier keys before any outcome. Omitted extras use that exact authored rank; present invalid extras fail. Scalars are finite, nonnegative, bounded and no smaller than one Q10 quantum when positive. Counts must be integral in both JSON and fixed point, 1 through 16. Duration is positive and bounded to 3600 seconds. Instant self metadata requires zero range and cast time. Reject invalid/dead/retired caster or any non-self target.

Raw statistics remain Q10 integers. Attack stun probability and duration remain normalized JSON numbers because the production normal-projectile consumer explicitly reads decimals. A single stat buff uses the existing ID so refresh and expiry remain in BuffStore. No extra visual projectiles gain damage rights. No ABI, schema, Unreal C++ or authored balance changes are required.
