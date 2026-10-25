# Enemy resource loss

The system SHALL drain at most the current target mana, with no HP/shield effect, no self-spend hook and no implicit restore. It SHALL validate declared positive finite rank amounts, enemy living target, current caster identity and range. Missing or empty target pools SHALL lose zero. Invalid requests SHALL reject the managed cast before committed cost, cooldown or sibling effects. The system SHALL show generated per-rank mana-loss hints and retain original damage, cooldown, range, cost and IDs.
