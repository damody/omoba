# Enemy mana drain errors and prevention

MD01 (read-only discovery): assumed ability_runtime.rs instead of inventory-returned ability_runtime/mod.rs. No source/build changed. Prevention: inventory module files before reading; directory modules are not interchangeable with sibling .rs files.

Design decision: hostile resource loss must not reuse GameWorld.spend_mana, whose notification means self spending. Use checked ManaPool::drain and a closed serial effect grant; no new ABI slot or raw buff state.

MD02: initial generic-effect patch assumed a single-line damage match arm; actual arm is multiline, so the patch was rejected atomically. Verified no partial writes before applying exact context. Prevention: read the specific match arm before patching, including test-only exhaustive consumers.
