# Actual cooldown instance duration in owner HUD

After authoritative cooldown modifiers were wired, owner HUD still derives total cooldown from authored base metadata. A slow cooldown can report remaining greater than total and make native validation reject it. Preserve the actual admitted total in shared Hero state and use it in existing HUD fields.
