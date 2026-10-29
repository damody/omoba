-- Manual icon assignments for scripts/import_nobu_icons.lua; they win over the automatic mapping.
--   kind = { [id] = "path relative to the history_nobu root" | "site:hero/<file>.png" | false (placeholder) }
-- site_portraits: visual review (2026-10-29) of the nobugg site's hero pictures. Only the ones in
-- `nobu` are nobu's own art (its WC3-era framed icons and three later paintings); the rest are
-- Dota 2 hero portraits and are never used. A picture in neither list is treated as unreviewed.
return {
  hero = {},
  ability = {},
  item = {},
  site_portraits = {
    nobu = { "a04", "a06", "a07", "a11", "a12", "a13", "a14", "a15", "a16", "a19", "a25", "a26", "a27", "a28", "a31", "a32", "b01", "b02", "b04", "b05", "b06", "b08", "b09", "b11", "b13", "b14", "b15", "b16", "b17", "b21", "b23", "b24", "b25", "b26", "b28", "b32", "b33", "b34", "b36", "c01", "c02", "c05", "c07", "c10", "c11", "c15", "c17", "c19", "c20", "c21", "c22", "c24" },
    dota = { "a01", "a02", "a03", "a05", "a08", "a09", "a10", "a17", "a18", "a20", "a21", "a22", "a23", "a24", "a29", "a30", "a33", "a34", "a35", "a36", "b03", "b07", "b10", "b12", "b18", "b19", "b20", "b22", "b27", "b29", "b30", "b31", "b35", "c03", "c04", "c06", "c08", "c09", "c12", "c13", "c14", "c16", "c18", "c23", "c25", "hero" },
  },
}
