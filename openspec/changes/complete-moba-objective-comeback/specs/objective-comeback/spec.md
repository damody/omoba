## ADDED Requirements

### Requirement: authority team compensation
The match SHALL use actual earned reward gold independent of spending, retain the ledger across hero lives, and publish eligibility through the existing public buff path.

#### Scenario: bounded objective payout
- WHEN a side meets authored elapsed-time and earned-gap requirements
- AND it destroys an enemy lane tower or kills a reward-bearing epic root
- THEN authenticated living and admitted dead teammates receive equal bounded bonus shares, once per objective life, in canonical tick order.

#### Scenario: invalid or inactive action
- WHEN damage is anonymous, allied, repeated, paused, outside active play, or forced retirement
- THEN no compensation claim is manufactured.

#### Scenario: budget exhaustion
- WHEN the activation pool is exhausted
- THEN eligibility is not refilled until the gap first reaches the authored release threshold.
