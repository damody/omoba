# Whole-game continuation audit

Scope: maintained backend/shared Rust/scripts and Unreal only. Continue the full inventory in 2026-10-21-whole-game-completeness-audit.md. Diagonal maps, camp return, towers/minions, cast windup/cancellation, public control and target feedback, point blasts, swept dashes and normal-hit passive sectors already have regression coverage; this batch does not claim to recreate a particular League patch.

This batch fixes the shipped self transformations' common admission gap, rather than another cosmetic recheck. Matchlock Gun and Three Stage now share checked ranks, bounded values and one living-self commit. Preserve all seven authored rank rows and the existing stat encodings, duration refresh, pause and single damaging normal projectile rule.

Single-machine tests cover actual adapters, BuffStore and formal 60Hz PlayerInput/normal projectiles. Consolidated acceptance additionally builds UE5.8/backend, checks staged DLLs, runs Rust/Lua and the existing native World matrix, and fingerprints production inputs. Failed fixtures and runner mistakes are retained in the errors document.

Still pending: sustained channels/zones, timed movement/residue, more specialized geometry/previews, summon-group atomic commit, complete visible combat-replica lifecycle acceptance, broader authored heroes/items/summoner operations, art/audio and balancing. Existing MVP 33/44 and 11 external ACK/LAN/physical DPI/fixed-performance items remain pending. No real matches, quota resets, commits/pushes or historical omfx work are authorized by this batch.
