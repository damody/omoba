# Design

Recognize current closed generic ability slow sources via shared declarative source parsing, generic_ability_slow family and one bounded negative Q10 move_speed_bonus. Recognize canonical projectile slow only under its reserved slow identity with the two current matching Q10-representable legacy factor/bonus fields. Require positive remaining lifetime and exact payload shape to avoid deleting mixed unrelated effects. No hero/camp/ability-name matching; no removal of arbitrary negative movement stats or malformed packets.

Cleanse collects recognized IDs, sorts, and removes them through existing canonical remove so reverse indexes stay correct. It does not mutate retained payloads, lifetimes or periodic clocks. New slows applied after cleansing work immediately; cooldown and pause policies remain unchanged. Both existing item and summoner operations already share this method. No new IDs, ABI, wire, skills, real network games or extra frame loops.

Verify valid/malformed/unknown/voluntary payloads, retained effects and index consistency; actual authored ranger slow followed by item/summoner cleansing and reapplication, pause and replay; then maintained builds and native tooltip pixels with immutable evidence closure.
