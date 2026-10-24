# Immutable missed ordinary projectiles

Source-side misses currently become zero-damage shots without preserving the reason. On arrival they can still enter hit/control callbacks and omit the existing MISS feedback. Preserve an authority-only launch result for native hero and typed script ordinary attacks, independent of zero damage, source lifetime and current target evasion.
