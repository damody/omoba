# Native conditional leech settlement

DamageTick currently emits an unconditional Heal computed before serial damage admission, packet reduction, shields and target HP capping. Bind its existing physical lifesteal/magical vamp flags to the damage packet and derive healing from actual accepted HP loss. Do not add new item/stat policies or infer ordinary-attack membership.
