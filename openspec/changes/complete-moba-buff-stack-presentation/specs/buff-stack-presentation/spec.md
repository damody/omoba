## ADDED Requirements

### Requirement: Generic authority stack metadata
Buff payload MAY contain numeric integer stacks 1..65535. Objective rewards SHALL emit current capped authority count, including timed and permanent effects. Existing objective_stacks SHALL remain compatible.

### Requirement: Owned validated presentation
Unreal SHALL display multiple stacks with the content name and current duration, preserve single-stack output, reject malformed metadata and out-of-range string references, refresh count-only changes and clear retired snapshots or old entity generations.
