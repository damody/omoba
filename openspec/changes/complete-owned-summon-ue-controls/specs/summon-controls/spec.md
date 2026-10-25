# Requirements

- A summon input SHALL be distinct from hero movement and SHALL retain exact opaque owner and summon disclosure lives with current connection/view scope.
- Only a current owned living summon selection SHALL be commandable; foreign, hidden, dead, retired or stale selection SHALL fail closed.
- Movement/release SHALL preserve hero commands/cast/channel and existing single-tick summon movement budget.
- Shortcut bindings SHALL be configurable and migrate without overwriting existing user chords; HUD SHALL describe actual bindings.
- Modal/focus/session reset SHALL retire input state.
- Active bridge generation/staging SHALL use clone-local Lua with verified artifacts, no PowerShell/shell fallback.
- Native fixtures SHALL be explicitly distinguished from new real network matches.
