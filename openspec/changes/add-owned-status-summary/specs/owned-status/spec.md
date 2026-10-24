## ADDED Requirements

### Requirement: Current public statuses are visible and bounded
A valid active local hero SHALL see a localized bounded summary of existing authoritative statuses, with full accepted list in a shared tooltip. Status identity, stacks, lifetime and privacy use existing validation. The summary SHALL clear for expiry, mismatched life, dead/finished, non-MOBA, hidden HUD or retirement. No local countdown is inferred.

#### Scenario: Overflow
- WHEN four valid states are current
- THEN two bounded rows and an explicit two-more count appear
- AND tooltip retains all four names, lifetimes and stacks
