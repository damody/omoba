# Hero-Independent TD Economy Design

## Goal

Tower Defense must retain Bloons-style cash when campaign heroes are disabled. Starting cash, tower purchases, upgrades, sales, pop rewards, round income, affordability UI, and deterministic state must no longer depend on a `Hero` entity.

## Scope

- Make TD cash authoritative per player rather than per hero.
- Keep `OMB_TD_STARTING_GOLD` as the launcher override; `run_10000.bat` continues to start each TD player with 10,000 cash.
- Preserve the existing difficulty defaults, including the current Bloons-aligned 650 starting cash.
- Preserve non-TD hero gold behavior.
- Do not restore invisible, placeholder, or renderable heroes.

## Architecture

Add a `PlayerEconomy` ECS resource in `omoba-core`. It stores a deterministic `BTreeMap<u32, i32>` from `player_id` to cash and provides checked read, debit, and saturating credit operations.

TD initialization creates accounts independently of hero creation. It initializes the same two player IDs previously represented by the two TD heroes, using the resolved `TdDifficultyConfig::starting_gold`. The hero-disable policy therefore has no effect on account creation.

TD tower placement, upgrade, and sale paths use `PlayerEconomy` as their authoritative wallet. A missing account is a rejected command with an explicit diagnostic; insufficient cash remains a normal rejected command. Successful mutations occur only after all command validation succeeds.

MOBA and hero-specific item or ability systems continue using the existing `Gold` component. If heroes are enabled in TD, their `Gold` component may remain for compatibility, but tower-defense purchasing and rewards use only `PlayerEconomy` so there is one TD authority.

## Reward Flow

- A completed TD round credits the existing Bloons-derived round income to every initialized TD player account, matching the prior behavior that credited every TD hero.
- A TD creep pop credits its bounty to the owning player when the damaging source has a `PlayerOwner` (for example, an owned tower).
- Rewards without an attributable player do not invent an owner.
- Non-TD proximity-based hero bounty and experience behavior remains unchanged.

All credits saturate instead of overflowing. Debits reject negative costs, missing accounts, and insufficient balances without partial mutation.

## Snapshot and UI

`SimWorldSnapshot` exposes deterministic player cash separately from render entities. The frontend selects the entry for `local_player_id` and updates its existing cash display and affordability checks even when there is no Hero entity.

Hero fields continue to update only when a hero exists. Receiving a player-cash snapshot must not create a fake hero selection or entity ID.

## Determinism

The player economy uses ordered storage and is included in the authoritative state hash. The same player IDs and balances therefore produce the same hash regardless of entity allocation or hero presence.

## Compatibility

- Existing hero-enabled TD launches receive the same starting balance and can build normally.
- Hero-free `run.bat` starts player 1 and player 2 with the configured TD balance.
- Hero-free `run_10000.bat` starts both accounts with 10,000 cash.
- `OMB_NO_HEROES` values other than exact `1` retain hero creation.
- No protocol migration is required because the local renderer consumes the shared `SimWorldSnapshot`; additions use defaults where snapshots are manually constructed in tests.

## Testing

Add focused tests proving:

1. TD account initialization works with zero Hero entities and applies both default and overridden starting cash through a resolved, environment-independent helper.
2. Tower placement and upgrades debit the requesting player's wallet; sales credit it; missing or underfunded accounts reject without mutation.
3. Round income credits every initialized TD account without heroes.
4. Owned tower pop rewards credit the correct account, while MOBA hero bounty behavior is unchanged.
5. Snapshot cash is available without Hero render data and the frontend consumes the local player's value.
6. State hashes change when a player balance changes and remain deterministic across insertion order.
7. Existing `omoba-core`, backend, and relevant frontend tests pass.

## Out of Scope

- Rebalancing tower prices or Bloons round tables.
- Shared co-op wallets, cash transfers, farms, or income-producing tower mechanics.
- Refactoring MOBA inventory, items, or hero experience.
