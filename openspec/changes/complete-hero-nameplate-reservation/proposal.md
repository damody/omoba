# Preserve readable hero space

The previous nonhero collision budget does not reserve visible hero footprints. Add a bounded reservation pass using actual rendered hero state, current anchors and the same real Slate measurements. Ordinary nonhero labels avoid heroes; selected/hovered targets may overlap reservations so inspection remains available, but still compete against other admitted nonheroes. Do not hide or reposition hero labels.
