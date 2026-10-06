# Bounded native widget lifetime implementation

Workspace D:/code/omoba; user explicitly requests Grok Build subagent. HEAD02757d51f201a03bcea46e1ca1d622594db28dde. Primary owns outcome/build/docs. Existing controller EndPlay/header/test changes MUST preserve. OmRuntime is only active frontend (no omfx). Lua author/build tools only, native Rust/C++ gameplay.

Read AGENTS.md then only bounded relevant sections of these files. Allowed writes:
- omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmCommandBarWidget.h
- omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmCommandBarWidget.cpp
- omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmPlayerController.cpp (ONE call before own command bar RemoveFromParent in existing EndPlay)
- omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmEditorAutomationTests.cpp (one new native test; preserve all existing tests)

Concrete defect: controller ending guard does not cover widget's independent SubmitShopAction/SubmitMinimapMove. Old retained widget or Slate callback can still submit using retained baseline and connected runtime. General solution: public idempotent RetireGameplayInput, terminal per-widget flag; NativeDestruct uses it (Super called). Controller EndPlay explicitly invokes it before removing own bar, including widgets with no built Slate tree. Retired widget stays retired across TakeWidget/rebuild or stale state setters: never reset flag in construct/rebuild. New HUD creates new widget. Do NOT stop shared runtime/server, clear another controller, change gameplay rules/ABI/content/generated code/Blueprint, or add second simulation.

Retire clears own economy and minimap baseline, submission status IDs/queued success, and disables those surfaces. CanSubmitShop and SubmitShopAction/SubmitMinimapMove must gate flag before any bridge query or global NotifyOwnedGameplayUnavailable. Submission attempts can update diagnostics as existing, but never call runtime or broadcast invalidation for a retired widget (would clear other controller). IsEnabled_Lambda for shop uses gate. After retirement new state setter must not restore readiness/interactive retained data. Rebuild may display unavailable data but cannot reactivate. Expose readonly IsGameplayInputRetired for narrow fixture if needed. Pure input builders stay pure/current ABI.

Add native test in existing EditorPreview style (NO launching/building Editor): populated economy/minimap setters -> retire idempotent -> late setters + rebuilt TakeWidget still retired; CanSubmitShop false, submission methods false with no game instance; other widget not retired. Explicit callback test is NOT true viewport/Actor destruction proof. Record findings in final response; primary central MD. Do not write docs or runner.

No commit/push/rebase/reset/clean/restore/switch branches/delete/install/credentials/network/MCP/engine mutation/process termination/privileged operations. Escalate exact action/target/risk to primary if needed, don't ask user. No UE/game/simulation/full-suite/Cargo. Apply_patch edits only. For read_file tool failures use one bounded terminal read; don't retry a stuck reader. Terminal validation only git diff --check; primary independent scoped C++ compile. Finish implementation promptly; don't explore historical docs. Report exact diff + validation actually run, no fabricated test pass/cost/root cause.
