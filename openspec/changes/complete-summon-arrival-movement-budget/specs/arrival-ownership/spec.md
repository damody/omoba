# Requirements
- A summon moved by the host SHALL retain movement ownership through arrival during the same tick.
- Ownership SHALL use full entity generation and current simulation tick and SHALL NOT block a later tick.
- Actual no-movement, pause, root, expiration and existing follow/explicit target semantics SHALL remain intact.
- A real authored summon arriving at a one-unit destination SHALL stop there for that tick, then resume autonomous pursuit next tick with deterministic paired traces.
