# omfx Start-Muted Design

## Goal

omfx must never start background music automatically, including when an existing
configuration file contains a non-zero music volume.

## Design

- Change the default music volume from `0.2` to `0.0`, so newly created and
  fallback settings are silent.
- Preserve a valid stored volume on disk, but normalize the runtime settings
  returned by `AudioSettings::load_or_create` to `0.0` on every process start.
- Keep the existing in-game music slider and save behavior. A player may enable
  music for the current run, while the next run starts silent again.
- Preserve unrelated configuration keys and existing invalid-input fallback
  behavior.

## Verification

- Unit-test that missing configuration is created at zero volume.
- Unit-test that startup normalization mutes a stored non-zero volume without
  rewriting it.
- Keep round-trip and clamping tests for the lower-level persistence helpers.

## Non-goals

- Removing the BGM asset or sound node.
- Disabling the music slider.
- Erasing the player's stored preference.
