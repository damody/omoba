# Resolved public attack range

NPC actors without TAttack publish zero attack range and the filtered bridge only knows a base TAttack range. Publish bounded resolved range independently of attack behavior, preserving private modifier/aggro data. Reuse existing render/CABI range field and world overlay consumer.
