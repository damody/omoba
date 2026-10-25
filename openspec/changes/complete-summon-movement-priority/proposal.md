# Summon movement priority
Host summon navigation and script aggro movement can both execute in one simulation tick. Keep an active host MoveTarget authoritative so autonomous script chasing cannot add a conflicting step while the order/follow target remains active.
