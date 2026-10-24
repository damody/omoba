# Design

Hero reservations are an exact union query independent of iteration order, so a hero priority shortlist serves no purpose. Reserve each validated projected visible hero directly in the existing public-target pass, before considering any nonhero candidate. Inline64 rectangle storage may grow; it is an allocation optimization rather than an admission limit. The nonhero shortlist/output remains64; reservation storage is O(H), rectangle work O(H*64), with no new all-entity sort or per-entity Slate mutation. Current view constructs fresh reservations, preserving concealment/death/retirement/movement behavior. No conservative bounding union that could hide labels in empty gaps.

Test distinct65th footprint, input-order invariance, exact gap availability,10000 reservations and retained64 nonhero limit. Existing actual hero widget becomes65th reservation with existing concealment/generation/movement cases and capture. Hero-to-hero overlap remains pending.
