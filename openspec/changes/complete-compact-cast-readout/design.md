# Design

Keep the authoritative cast state and lifecycle unchanged. GetAbilityCastText supplies full or compact labels from the same state. Active compact text retains seconds and the positive <0.1s threshold; terminal labels distinguish cancelled and completed. Disable wrapping in both initial construction and retained legacy composition; measured card width drives existing responsive reflow. Never shrink the selected font or infer local cast time.

Extend the actual game-viewport regression to 800x600/1280x720, text scales 1/1.5/2, and maximum, decimal, subdecimal, cancelled and completed states. Verify glyph-measured single-line height, arranged width, full tooltip status, responsive reparenting and retirement. Build UE5.8 and run all 75 current native tests, stage/source SHA checks and actual PNG review. Inherit unchanged Rust/backend results only after verifying the complete prior input baseline.
