# Hero reservation contract

- Use actual current render visibility and entity generation, not old desktop geometry or hero names.
- Reserve up to 64 valid hero footprints without hiding heroes; admit up to 64 nonheroes.
- Invalid, offscreen or retired reservations do not occupy space; overlapping hero reservations are legal.
- Ordinary labels avoid reserved rectangles, focused labels bypass only reservations, and tangent boundaries remain shared with existing spacing.
- Fresh views restore labels when a hero becomes hidden or moves away.
- Verify real actor conceal/reveal/retire/reuse, actual Slate layout, geometry limits and existing full World/map regressions.
