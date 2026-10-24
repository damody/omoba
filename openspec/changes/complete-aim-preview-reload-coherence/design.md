# Design

Content model owns a canonical projection of authored aim kind/keys, target semantics, rank ranges, effective point-area radii, swept widths, facing formation rows/count/spacing/front and movement requirement. Generator consumes that projection instead of duplicating interpretation. Internal AbilityConst stores its canonical JSON signature in compiled and runtime content. Runtime catalog validation and ContentShape compatibility reject divergent signatures before replacing a snapshot. No wire/GameWorld ABI changes; all formal Rust/Unreal artifacts rebuild together.

Do not freeze unrelated damage/duration/cosmetic values merely because they share extras. Effective geometry and explicit contract changes require all peers to rebuild. Failed validation leaves the installed snapshot/generation untouched using the existing transaction.
