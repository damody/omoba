# Slow cleansing errors and prevention

SC01 (read-only audit): assumed native/comp/buff.rs and a sniper handler filename before inventory. Actual buff storage is ability_runtime/buff_store.rs; sniper source is inventory-returned heroes/B01_saika_magoichi/No1_sniper_mode.rs. No source or worker was changed. Prevention: inventory files before reading, including small discovery probes.

Audit finding: canonical cleanse currently removes only stun/root/silence. Do not claim it already removes slows. Current hostile slow producers are generic source+family/Q10 payload and reserved projectile slow with factor/bonus floats; classify their closed semantics, not arbitrary negative movement stats or substring names.

SC02 (pre-seal accounting): predicted six focused tests, but the control_cleanse filter includes three existing cases, giving seven actual passes. Corrected the unlaunched seal/closure assertions from the original probe logs; no test failed or result was discarded. Prevention: derive accepted counts from actual test-result records rather than guessed filter cardinality.
