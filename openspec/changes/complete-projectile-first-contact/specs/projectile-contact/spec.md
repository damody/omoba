# Requirements

- Straight projectiles SHALL select the earliest valid Q32 contact over all candidates returned by the current spatial range query.
- Equal Q32 contacts SHALL use stable canonical entity identity, independent of index traversal.
- Capsule boundaries SHALL use shared fixed-point predicates; the first contact SHALL be rounded conservatively to the next Q32 quantum.
- Missing current target positions SHALL be ignored, not replaced with origin coordinates.
- The change SHALL preserve existing creep collision policy, source attribution and endpoint/lifetime flight budget.
