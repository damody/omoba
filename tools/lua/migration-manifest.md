# Script migration manifest

Runtime: `D:\code\omoba\tools\lua\lua.exe` (Lua 5.4.8).

All replacements preserve accepted CLI arguments, referenced environment variables,
observable stdout/stderr, output files, exit status, child process identity, and
reverse-order cleanup unless the OpenSpec explicitly replaces the entry point.

| Legacy script | Lua replacement | Contract class |
|---|---|---|
| `run.bat` | `scripts/run.lua` | retained thin wrapper; build, stage, launch |
| `run_10000.bat` | `scripts/run_10000.lua` | retained thin wrapper; release/high-load launch |
| `run_2player.bat` | `scripts/run_2player.lua` | retained thin wrapper; server, proxy, runtimes, renderers, evidence |
| `run_ue.bat` | `scripts/run_ue.lua` | retained thin wrapper; Unreal launch |
| `run_lives1.bat` | `scripts/run_lives1.lua` | removed wrapper; special dev environment |
| `run_sandbox.bat` | `scripts/run_sandbox.lua` | removed wrapper; sandbox environment |
| `scripts/test_td_1_to_100.bat` | `scripts/test_td_1_to_100.lua` | build and test exit status |
| `scripts/capture_fog_screenshots.ps1` | `scripts/capture_fog_screenshots.lua` | PID-scoped screenshots |
| `scripts/compare_fog_evidence.ps1` | `scripts/compare_fog_evidence.lua` | evidence verdict |
| `scripts/dev_run_freshness.ps1` | `scripts/dev_run_freshness.lua` | fresh/stale/error exit status |
| `scripts/dump_process_memory.ps1` | `scripts/dump_process_memory.lua` | PID-scoped Windows dump |
| `scripts/dump_process_memory_linux.sh` | `scripts/dump_process_memory.lua` | PID-scoped Linux dump |
| `scripts/run_fog_lifecycle.ps1` | `scripts/run_fog_lifecycle.lua` | renderer/runtime lifecycle |
| `scripts/start_backend.ps1` | `scripts/start_backend.lua` | background server and ready probe |
| `scripts/start_client_runtime.ps1` | `scripts/start_client_runtime.lua` | background replica and ready probe |
| `scripts/start_fog_demo_frontend.ps1` | `scripts/start_fog_demo_frontend.lua` | renderer environment and PID |
| `scripts/test_run_session_launcher.ps1` | `scripts/test_run_session_launcher.lua` | PID-scope assertions |
| `scripts/validate_td_map_bounds.ps1` | `scripts/validate_td_map_bounds.lua` | validator exit status |
| `scripts/write_fog_run_manifest.ps1` | `scripts/write_fog_run_manifest.lua` | manifest schema |
| `scripts/gen_stress_map.py` | `scripts/gen_stress_map.lua` | deterministic generator |
| `docs/tools/gen_stat_keys.py` | `docs/tools/gen_stat_keys.lua` | deterministic generator |
| `docs/tools/migrate_sk_callers.py` | `docs/tools/migrate_sk_callers.lua` | source migration |
| `docs/character_pipeline/tools/bootstrap.py` | `docs/character_pipeline/tools/bootstrap.lua` | toolchain orchestration |
| `tools/selective_lockstep/common.py` | `tools/selective_lockstep/common.lua` | record schema module |
| `tools/selective_lockstep/network_fault_injection.py` | `.lua` peer | evidence producer |
| `tools/selective_lockstep/observer_slowdown.py` | `.lua` peer | evidence producer |
| `tools/selective_lockstep/packet_capture_scan.py` | `.lua` peer | fail-closed scanner |
| `tools/selective_lockstep/paired_world_fixture.py` | `.lua` peer | deterministic fixture |
| `tools/selective_lockstep/redaction_scan.py` | `.lua` peer | fail-closed scanner |
| `tools/selective_lockstep/stress_report.py` | `.lua` peer | evidence report |

The netem scripts created by the active, uncommitted delay change are also migrated:
`start_netem_proxy`, `stop_netem_proxy`, `send_netem_control`,
`run_client_delay_scenario`, and `run_client_delay_matrix`.

Archived design/history documents remain historical. Current source, automation,
operator docs, and non-archived specifications are executable-reference scope.
