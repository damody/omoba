# Design

Append InvalidManaCost to the local preview enum and an optional authority cost state to Evaluate. Preserve unlearned/cooldown precedence, then show explicit invalid cost before shortage/geometry. Only matching pending ability catalog identity supplies the cost state. Missing, unknown and unsupported values never imply invalid. Existing owner snapshot retirement clears the cache. Submission remains governed by the existing authority path.

Native PointAreaPreview covers typed state, label/color, precedence, normal/unknown/malformed states and unlearned/missing-shape/owner retirement. Rebuild UE5.8, run the complete existing native suite, compare/review captures and verify formal source stability. Rust/backend remain unchanged and inherit the previous accepted cohort only after source identity is checked.
