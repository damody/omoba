# Design

Select semantic channel mapping first, dedicated channel slot second, then recall/cast assets only under UseGenericState. A broken explicit mapping fails closed. Sample the existing clip at authoritative elapsed fraction and hold between snapshots, so local mesh time never predicts progress or completion. O(1) actor fields only; no timers/scans/components. Opaque instance keys use every nonzero bit pattern.

Reuse native epoch/tick/sequence/identity/life checks, adding the existing world presentation scope gate before epoch reset. Death/respawn, recall and preparation win. Channel blocks locomotion/hit/release animation until a current clear; cancellation restores latest locomotion. Hide/generation/reset retire state; reveal alone cannot replay cached channel. Current recall retires prior channel and requires a new frame for restoration.

Tests cover pure selection/fallback, two distinct real hero skeletons, exact pose cursors, no local progress, lifecycle/old/foreign/epoch/recall and actual rendered early/mid/late poses. Existing world VFX test now requires an authoritative channel pose in its combined character capture, preserving actual filtered ABI coverage. Consolidated native77, final44 PNGs, UE5.8 build/stage/source checks; full Rust/backend inherited only under whole-baseline SHA proof.
