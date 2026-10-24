# Post-seal documentation recovery

The original immutable close-docs-v2.lua failed before any document mutation with: `attempt to index a nil value (field source)` at line8. The sealed index has top-level source_count=3472, not source.count. All formal acceptance,8original worker retirement receipts and593-file closure were already complete. Original sealed helper/index bytes remain untouched.

V3 checks actual index schema and the original helper SHA against the sealed index, then evaluates the original writer with only index.source.count replaced by index.source_count. It writes the final report/tasks/handoff/audit; no formal code or acceptance rerun. Prevent recurrence: inspect actual JSON keys before implementing consumers and assert the schema boundary before formatting. This recovery MD/helper are post-seal documentation records and are not claimed as part of the593-file sealed cohort.
