# Complete authoritative cooldown modifiers

Existing cooldown_percentage and cooldown_percentage_stacking are exposed through UnitStats/ABI but production ability dispatch and managed resource reservations use the authored base duration. Apply the existing additive multiplier policy through a checked shared calculation, once per successful release. No hero-specific branches or new wire/ABI.
