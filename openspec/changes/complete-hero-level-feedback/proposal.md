# Authoritative hero level feedback

Show a short readable HUD notice when an already observed local hero increases level. Initial synchronization, reconnect/reset, stale or duplicate samples and generation changes must not fabricate a level-up. Reuse current hero state and header layout; no new network fields or predicted progression.
