## ADDED Requirements

### Requirement: Actual original-process exit determines acceptance
Acceptance SHALL verify a retired exit receipt bound to the original PID, creation token and executable. Nonzero Windows exit SHALL fail regardless of logs or summary. Optional summary SHALL be fresh and successful. Historical outputs SHALL never be overwritten.

#### Scenario: Later failure after intermediate success
- WHEN a child prints exit=0 and writes success=true then fails
- THEN acceptance fails and retains its actual nonzero exit and original logs

#### Scenario: Forged identity or stale summary
- WHEN receipt lifetime/executable differs or summary predates the run
- THEN acceptance rejects and does not adopt another process lifetime
