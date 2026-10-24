# Strict summon formation admission

Existing formation casts silently truncate fractional counts and use fixed rank-one defaults when metadata is malformed or omitted. Replace permissive parsing with a reusable bounded formation resolver, preserve authored rank values and facing-relative placement, and correct misleading point-location wording. Keep existing host cast transaction and reservation lifecycle; whole-group settlement after concurrent ownership changes remains a separate scope.
