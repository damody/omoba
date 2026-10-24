# Design

Normal/native/script healing and positive HP regeneration use the same recipient multiplier and DisableHealing policy. Validate raw and legacy factors in 0..4, reduce in canonical source/family order with checked i128 intermediates, reject invalid/overflowing policies without healing. No negative healing or resurrection. Health clamps use wide intermediates. Mana, damage and shields are independent.

Summoner healing includes the checked multiplier in the authoritative context and immutable plan identity. Apply it to authored healing before the missing-HP cap, and revalidate at commit. No second multiplier in the sink. Zero-heal casts retain the ordinary committed cooldown.

Fountain recovery and a camp's completed hard reset are restoration, not combat healing. Preserve fountain recovery and make camp hard reset explicitly restore current HP, so an antiheal debuff cannot leave a reset camp partially injured.

UnitHex gains optional healing_reduction={multiplier,duration}, multiplier0..below1 and duration1/1024..60. Generated native constants, runtime validation and Unreal metadata share this contract. Existing hex authors multiplier0.5 for4s, alongside unchanged55 damage. One public native buff retains the strongest reduction and longest remaining deadline on refresh; different item IDs cannot multiply the same effect. Enemy/visibility/range/life and preflight transaction checks remain in force before cooldown or buff commitment. UI describes percent and duration using generated metadata.
