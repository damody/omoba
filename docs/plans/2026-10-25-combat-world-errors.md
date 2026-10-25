# Combat visual world isolation errors

CW01 (read-only discovery): a native Windows rg path containing OmCombatText* failed with OS error 123. No such named files were returned by inventory. No source or worker was changed. Prevention: inventory with rg --files first; search a real directory with -g filters, never assume shell expansion of native path wildcards.

Finding: floating text and one-shot/persistent combat allocation did not consistently verify Owner->GetWorld() before shared history or pool mutation. Channel and ground registration already did. Matching entity identifiers across independent worlds must not permit visual leakage or consume the local world's deduplication keys. Use a constant-time world identity guard at public admission and the shared allocator; retain existing hidden/overload consumption for legitimate local cues.

CW02 (prelaunch helper review): the drafted test selector used NativeCombatVisualLifecycle; inventory of IMPLEMENT_SIMPLE_AUTOMATION_TEST showed the actual name NativeCombatVfxLifecycle. Corrected before launching any worker; no failed test or discarded run. Prevention: derive exact test selectors from registration strings before freezing a worker.

Acceptance observations: all three native tests passed on their first execution, with zero test-report warnings. The full preserved engine startup log contains existing tool-plugin configuration HTTP 401 warnings and non-Windows SDK validation messages; zero test warnings does not mean an empty global startup log. No unrelated plugin/account/SDK changes were made. Three actual 512x256 captures inspected with view_image show readable +10 (cyan), MISS (white), CRIT (gold), with no clipping.
