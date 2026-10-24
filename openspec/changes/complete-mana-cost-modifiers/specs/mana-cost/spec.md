# Mana cost admission

## Requirement: distinguish invalid modifiers from free casts
Authority and bot planning SHALL use checked family-aware mana-cost aggregation and checked one-plus arithmetic. Valid negative multipliers SHALL clamp to zero. Invalid or unrepresentable aggregation SHALL reject a cast without resource/effect/cooldown commitment. Existing ceil-product cost arithmetic SHALL remain shared.

### Scenario: modifier changes during preparation
The final mana charge SHALL use release-time modifiers; rejected release SHALL not produce summons or cooldown. No early charge is committed during preparation.
