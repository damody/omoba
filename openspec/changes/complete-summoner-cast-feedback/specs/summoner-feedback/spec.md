# Requirements

- Supported point summoners use current disclosed cast and movement controls in either local slot, without spell-name or catalog-number special cases.
- Unknown, wrong owner/catalog, retired or malformed state must not claim the cast is ready.
- Preview remains advisory. Out-of-range guidance must not infer unpublished clamp/reject policy or terrain landing.
- Resolved zero-distance blink is rejected atomically without changing HP, movement, commands or either cooldown; one raw displacement remains legal.
- Existing self-heal and nonzero blink behavior retain regression coverage.
