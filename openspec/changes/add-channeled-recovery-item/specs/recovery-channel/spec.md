# Recovery channel

## ADDED Requirements

### Requirement: Checked timed recovery channel
A successful ChannelHeal item SHALL start a stationary owned channel and charge cooldown once. Healing SHALL occur only at authored interval boundaries for the same eligible living owner. Pause SHALL freeze time. Committed HP damage and accepted interrupting commands SHALL prevent subsequent healing; shield-only hits and root alone SHALL not interrupt. Inactive, dead, relocated, silenced, stunned, recalling or replaced owners SHALL lose the channel without a late pulse. Healing SHALL use current received-healing and maximum-HP policies.

#### Scenario: Full channel
- WHEN a valid3-second/1-second-interval channel starts
- THEN activation does not heal and three authored pulses occur at interval boundaries
- AND status disappears at completion without another cooldown charge

#### Scenario: Damage at pulse boundary
- WHEN positive HP damage commits in the tick reaching a pulse
- THEN cancellation precedes that tick's healing

### Requirement: Public guidance
Shop/hotbar SHALL expose the typed self item, authored timing and interruption explanation. Public hero status SHALL expose only a validated countdown and clear on completion, cancellation or retirement.
