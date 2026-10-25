# Recovery skill requirements

- The skill SHALL resolve its pulse values from checked rank authoring and start no healing before its first interval.
- Start, resource cost and cooldown SHALL be admitted atomically; rejected starts spend nothing.
- Stop/move/attack/other accepted casts, damage, control, owner/life/phase changes SHALL interrupt according to the shared channel contract. A later same-frame Stop SHALL prevent late startup.
- Pause SHALL freeze time; root alone SHALL permit a stationary channel. Healing SHALL use existing healing reduction and max-health rules.
- Public status SHALL reveal only the existing validated duration presentation. Current tooltip and native channel UI SHALL describe/show the real behavior.
- Runtime SHALL contain no hero/skill-name execution branch or magic buff request payload.
