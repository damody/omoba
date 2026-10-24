## ADDED Requirements

### Requirement: Ability tooltips explain both actual cast routes
Tooltips SHALL list all active primary/secondary cast chords and explain current normal confirmation/cancellation for icon aiming independently of shortcut mode. Missing cast shortcuts SHALL retain complete icon instructions. No-target casts SHALL remain immediate without aiming help.

#### Scenario: Quick shortcut with custom confirmation
- WHEN quick casting is selected and confirmation is rebound
- THEN the shortcut instructions describe quick casting
- AND icon instructions describe normal aiming with actual current confirmation/cancel chords
