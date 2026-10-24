## ADDED Requirements

### Requirement: Shared nonhero collision admission
Selected and hovered nonhero healthplates SHALL avoid current visible hero reservations and previously admitted nonhero rectangles through the same bounded screen-space admission used by other nonhero rows. Focus SHALL retain candidate priority but SHALL NOT bypass collision checks. The renderer SHALL preserve hero visibility, current identity/disclosure checks, world attachment and existing selected-unit details. Fresh layouts SHALL release obsolete reservation occupancy after movement, concealment or retirement.

### Requirement: Actual native verification
Tests SHALL exercise shared measured hero layouts, rejection without admission mutation, multiple UI scales and fresh recovery. The actual focus capture SHALL show the hero without an overlapping nonhero bar. Existing selection and details coverage SHALL pass; hero-to-hero overlap SHALL remain an explicit independent limit.
