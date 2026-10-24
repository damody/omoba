# Shared admission policy

Add checked_mana_cost_mult using checked_sum_add and checked one-plus-percentage. Negative valid sums clamp to zero as before; invalid/overflow returns None. Production managed cast and bot affordability use the same result, then the existing checked_mana_cost ceil-product. Snapshot only at successful release; no resource reservation commits before preparation. No ABI/wire changes or name-specific runtime branches.
