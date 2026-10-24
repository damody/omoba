# Correct desktop-to-paint coordinates

Nonzero-window wheel mismatch has a confirmed engine-space cause: SWidget::Paint passes window-relative allotted geometry, then caches a desktop geometry by appending FPaintArgs window offset. Input/cached viewport points are desktop-relative. Shared aim painting was applying paint AbsoluteToLocal directly to desktop points.
