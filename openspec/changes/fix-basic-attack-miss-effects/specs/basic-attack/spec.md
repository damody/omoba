# Requirements

- Native damage and native hero launch SHALL use the same deterministic bounded source-miss function.
- Hero legacy accuracy failures and shared source misses SHALL carry neither physical hit damage nor attached attack stun.
- Attempt timing and one-shot attack bonus consumption SHALL remain once per committed attempt, including misses.
- Visual pellets SHALL remain harmless and SHALL not become normal-attack hit sources.
- Launch decisions SHALL remain immutable under post-launch miss-buff removal/addition.
- Formal60Hz real PlayerInput attacks SHALL cover both miss policies and successful-hit controls, actual projectiles/impact/control state, not only packets.
