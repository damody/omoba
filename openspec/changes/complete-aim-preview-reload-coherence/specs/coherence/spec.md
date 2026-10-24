# Requirements

- Shared projection, no hero IDs or cosmetic cue inference.
- Changing explicit geometry kind/keys/rows, rank range, effective area radius, swept width or formation count/spacing/front requires rebuild.
- Compiled startup validation and reload compatibility both reject divergent signatures.
- Nongeometry values retain prior policy; malformed geometry is validated before projection.
- Existing snapshot is not replaced after rejected validation; retain source/stage evidence and single-machine regressions.
