# Retire deferred zone pulses at serial settlement

BuffTick currently expands all due pulses into damage/cues before any hit settles. A source killed by an earlier pulse in the same tick can therefore leave later ghost damage. Defer host-only pulse expansion until serial settlement, rechecking exact source generation, living health/faction and current target state. Preserve chronological pulse ordering and existing source-bound lifetime policy.
