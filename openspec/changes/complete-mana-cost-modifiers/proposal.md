# Checked mana-cost modifiers

The cost product is checked, but its shared multiplier currently uses wrapping aggregation/addition, allowing malformed or overflowing data to resemble a free spell. Authority and bot budgeting must distinguish invalid input from intentional zero-cost modifiers using the existing schema and family policy.
