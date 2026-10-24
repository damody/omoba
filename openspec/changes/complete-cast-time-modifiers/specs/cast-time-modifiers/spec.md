# Requirements

- CastTimePercentage affects every positive-time admitted hero preparation through common code.
- Instant abilities remain instant; positive duration never becomes zero or exceeds60sec.
- Buff aggregation follows shared checked/family policy and cannot wrap malformed/extreme values into a valid duration.
- Duration is snapshotted at start; midcast buff drift does not retime release.
- No effects/mana/cooldown before successful release; invalid start emits none.
- Public preparation total/remaining and existing Unreal progress use the effective duration.
