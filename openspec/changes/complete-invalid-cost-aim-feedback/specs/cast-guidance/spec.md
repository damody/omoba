# Cast guidance

## ADDED Requirements

### Requirement: Disclosed invalid cost has explicit aim feedback
The pending ability aim overlay SHALL show the authority's invalid-cost reason and refusal color when the matching current ability has cost state 3. Unknown cost SHALL NOT be represented as invalid cost. Unlearned and cooldown reasons retain precedence. Retiring owner or ability disclosure SHALL clear the cost indication.

#### Scenario: In-range ability with invalid cost
- WHEN the pending ability matches the disclosed catalog and cost state is 3
- THEN aim feedback states that mana cost cannot be calculated and the ability cannot currently be cast
- AND it uses the existing refusal color instead of normal in-range feedback
