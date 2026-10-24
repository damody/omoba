## Requirements

- Direct scripts, DoT and script projectiles settle base physical/magical defenses exactly once; pure damage bypasses these defenses.
- Already-settled native packets never gain another base-defense pass.
- Current defense buffs, shield absorption, HP facts, source generation and projectile provenance remain authoritative.
- TD profile immunity/layer damage remains unchanged.
- Invalid raw amounts or malformed defense aggregation fail closed without changing HP or consuming shields.
