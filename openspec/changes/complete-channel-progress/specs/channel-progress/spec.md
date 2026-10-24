## ADDED Requirements

### Requirement: Authority-driven owned progress
The HUD SHALL display a living local hero's unique valid disclosed channel name, remaining time and elapsed fraction using the published total. It SHALL hide on cancellation, completion, death, owner/life/phase change, scope reset and retirement. Stale snapshots SHALL NOT rewind or revive cleared progress. Pause SHALL not advance a local clock.

#### Scenario: Join midway
- WHEN total3 seconds and remaining1.5 seconds arrive as the first valid snapshot
- THEN the strip displays50 percent elapsed, without inferring total from remaining.

### Requirement: Bounded public decoding
Native projection SHALL reject malformed, oversized, contradictory, unknown or ambiguous channel metadata without exposing arbitrary private JSON or synthesizing a total.
