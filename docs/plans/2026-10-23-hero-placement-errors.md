# Hero placement errors/prevention

Read-only engine search used a wildcard path ending Private/Slate/*Widget*, not expanded by PowerShell; corrected by rg--files discovery and exact SWorldWidgetScreenLayer.cpp. Repeated path/glob mistake retained. Prevention: filter via rg-g or discovered exact paths, never wildcard directory arguments.

Pre-reader call-site review added a validated actor placement result and original-anchor reservation fallback, avoiding reserving a translated footprint if the actor rejects malformed size/scale/offset. Invalid scale is checked before division. Common144logical limit is shared by actor and layout; no duplicate bound literal in production.

UE5.8 source SWorldWidgetScreenLayer.cpp reads component GetPivot each tick and assigns SConstraintCanvas slot alignment; current pivot placement matches that native mechanism. Fixed grid budget prevents dense input quadratic scans; exhaustion retains all labels at original anchors. No full10000rendered benchmark or impossible dense separation claim. OpenSpec CLI absent, direct artifact fallback and fixedLua/Rust1.95.

Post-build/pre-native fixture review: SConstraintCanvas default alignment is(0.5,0.5), so the new capture heading at24,24 could be cropped. Successful UEBuildV1/stage and6inputs-v1 preserved; all readers retired/reaped before changing only fixture heading to explicit zero alignment.6inputs-v2 retains5production files unchanged; UEV2/native/source required. No runtime failure occurred. Prevention: every auto-sized capture heading must explicitly set its intended alignment, rather than assuming the engine default.
