## ADDED Requirements

### Requirement: Native leech depends on accepted damage
Native physical lifesteal and magical vamp SHALL heal only from actual HP loss caused by their bound damage packet after serial admission, packet modifiers, shields and overkill. Pure damage SHALL earn neither. Healing SHALL honor source lifetime, received healing, maximum HP and no revival.

#### Scenario: Shield fully absorbs damage
- WHEN a native packet with leech reaches a fully protecting shield
- THEN the packet causes no HP loss and grants no healing.

#### Scenario: Target has less HP than packet damage
- WHEN a native packet overkills a target
- THEN only actual lost HP contributes proportionally to component leech, with one checked fixed-point rounding.
