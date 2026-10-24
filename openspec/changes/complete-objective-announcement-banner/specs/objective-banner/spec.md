# Objective announcement banner

## ADDED Requirements

### Requirement: Public objective notices have readable bounded presentation
The maintained Unreal HUD MUST display a distinct banner only for a valid active public objective notice and clear it on absent/expired/terminal/retired state. It MUST use existing header width and text-scale budget and MUST NOT block gameplay input or invent a local lifetime.

#### Scenario: Authority notice expires
- WHEN the next authority snapshot has zero remaining notice time
- THEN both the stored banner text and rendered banner visibility clear
