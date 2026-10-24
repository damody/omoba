# Design

Timer formatting: exact0 =>0, positive<0.1 =><0.1, other finite nonnegative times =>one decimal with trailing.0 removed. Reject invalid negative/nonfinite and remaining>total display pairs. Keep range scalar formatting separate so timer policy does not alter distance semantics. Shared pair helper drives existing logical/expanded readouts; tooltip adds named remaining/total in seconds. No interpolation, local countdown, catalog name branch, authority mutation or new client input gate.

Extend existing actual summoner native test for D/F tiny positive values, fractions, invalid pairs and scope clearing, and add positive-timer guidance to the existing real tooltip capture. Native-only two-source change: exact SHA proof inherits prior Rust1863/9ignored/backend; rerun UE build/stage/full82native/source3447/51PNG. Hero overlaps/externals unchanged.
