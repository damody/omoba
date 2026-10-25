# Requirements

- A role bot SHALL emit no new gameplay inputs while its exact live owner is in an authoritative recovery channel.
- The rule SHALL apply equally to authored skills and items, without names or public status inference.
- Root and pause SHALL preserve waiting; normal completion or accepted interruption SHALL allow normal decisions on the next eligible think.
- Another entity's channel or an injected duration-only buff SHALL NOT freeze the bot.
- Existing disclosure, phase, visibility freshness and think cadence gates SHALL remain unchanged.
