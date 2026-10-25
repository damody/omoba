# Plan

1. Preserve sealed bot-channel-intent source generation and add an append-only player action with separate exact owner and summon references; closed operation1 MoveTo /2 ReleaseToAutonomy and strict point presence.
2. Use the existing ordered pending movement boundary without clearing owner hero commands/casts. Bound32 owned-summon requests per player per tick. Authenticate the player roster/current hero and summoned relationship, optional player owner consistency, same faction, positive health, unexpired life and supported geometry. Movement obeys target control/terrain; release is allowed under movement control and resets cached auto navigation.
3. Reuse MoveTarget, SummonNavigation and common attack interruption. Do not add a competing timer or a content-name branch. Preserve existing arrival-frame movement budget and autonomous resumption.
4. Add strict/protobuf/formal60Hz actual summoned-unit owner isolation/movement/release/control/terrain/stale/phase/pause/replay coverage; consolidate maintained builds/staging/source checks, retain errors/original workers and update inventory/handoff/tasks. Renderer/UE issuing this input is explicitly pending next.
