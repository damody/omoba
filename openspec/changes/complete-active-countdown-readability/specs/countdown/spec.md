## ADDED Requirements

### Requirement: Positive active countdowns remain positive
Cast-control/item-aim/recall guidance SHALL show positive sub-tenth seconds as<0.1. Exact zero SHALL retain existing authority state/lifecycle behavior, invalid values SHALL not become readiness, and permanent control SHALL not invent a countdown. Shared formatting SHALL preserve ordinary countdown and compact cooldown precision.

#### Scenario: Active final fraction
- WHEN disclosed stun/silence/root/item cooldown or recall remains0.01second
- THEN guidance shows<0.1seconds and original action/state rules remain unchanged
