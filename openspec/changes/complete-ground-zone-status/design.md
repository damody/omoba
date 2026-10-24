# Design

Append a stable native-only ground_damage_zone catalog identity. BuffVisualState recognizes only validated DamageZoneSpec payloads with positive bounded remaining lifetime, aggregates simultaneous sources to one public icon with maximum remaining lifetime, and publishes catalog ID/time only. Reserved canonical name without a valid zone payload is not sufficient. Death cancels owned zones immediately in the authoritative death handler, not one buff tick later.

Reuse owned-buff hero details formatting, ownership/generation gates, retirement and Slate tooltip. No new wire fields or public source/location data. Native UI acceptance adds a real initialized widget and screenshot, with expiry/refresh/death proof in Rust and formal input.
