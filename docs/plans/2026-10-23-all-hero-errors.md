# All-hero reservations errors and prevention

Before readers, geometry review caught a proposed gap fixture width100 overlapping the second hero despite being tangent to the first; reduced only that test label to50, fitting the exact unoccupied gap. No build/runtime failure. Prevention: compute both padded rectangle endpoints, not one neighbor.

Before readers, call-site review retained the old second-phase actor entity/generation identity check when moving hero reservation into the projection pass. Prevention: moving work between phases must carry its validation gates, not just the effect.

OpenSpec CLI remains unavailable; direct repository artifact fallback used. Fixed Lua/Rust1.95 workflows, no new toolchain. No failed runtime accepted.
