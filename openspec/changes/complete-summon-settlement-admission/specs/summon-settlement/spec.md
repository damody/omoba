# Requirements

- A summon must have a current living owner, matching faction team and matching optional player owner at settlement.
- Lifetime must remain in the existing inclusive0..3600seconds range; zero preserves permanent semantics.
- Rejected spawns release their reservations and retire only their unmaterialized handle, with no successful spawn fact or Spawn hook.
- An already materialized entity cannot be overwritten, deleted or announced again by a repeated spawn outcome.
- Accepted spawns retain collision-safe placement, authored stats, ownership and callback behavior and produce exactly one committed spawn fact.
