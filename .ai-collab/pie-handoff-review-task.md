# Bounded independent Grok review, no tools or writes

Review these newly compiled native handoff fragments only for concrete contract defects. No tools, files, build, simulations, credentials or external operations. Do not invent new requirements. Return concise findings or accepted scope; do not claim tests ran. Default behavior without in-place flag stays unchanged. Codex owns integration.

Widget StartSelection calls StopSelection then resets bFinalizedReceiptSaved=false before any validation. Only after accepted Rust terminal reply is saved successfully (FILEWRITE_NoReplaceExisting):
```cpp
bFinalizedReceiptSaved = true;
StopSelection(); // clears protocol state but preserves PlayerId and this flag
if (!FParse::Param(FCommandLine::Get(), TEXT("om-hero-selection"))
    || !FParse::Param(FCommandLine::Get(), TEXT("om-selection-in-place")))
    FPlatformMisc::RequestExit(false);
```
Getters expose saved flag and original PlayerId only to native C++; not UFUNCTION. Existing reply parser strictly checks player/catalog/request/final plan. Widget Fail currently calls StopSelection, logs error, and RequestExit when smoke hero opt-in exists; no future tick occurs after successful StopSelection because Process invalid. NativeDestruct also calls StopSelection.

Controller:
```cpp
bool AOmPlayerController::IsHeroSelectionPending() const {
 return !bHeroSelectionCompleted && FParse::Param(FCommandLine::Get(), TEXT("om-hero-selection"));
}
bool AOmPlayerController::ContinueAfterHeroSelection() {
 if (bEndingPlay || !FParse::Param(FCommandLine::Get(), TEXT("om-hero-selection"))
  || !FParse::Param(FCommandLine::Get(), TEXT("om-selection-in-place"))) return false;
 if (bHeroSelectionCompleted) return true;
 if (!HeroSelectionWidget || !HeroSelectionWidget->HasFinalizedSelectionReceipt()
  || HeroSelectionWidget->GetSelectionPlayerId() != static_cast<uint32>(GetOmLocalPlayerId())) return false;
 UGameInstance* Instance = GetGameInstance();
 UOmRuntimeBridgeSubsystem* Bridge = Instance ? Instance->GetSubsystem<UOmRuntimeBridgeSubsystem>() : nullptr;
 if (!Bridge || !Bridge->StartPresentationAfterHeroSelection()) return false;
 bHeroSelectionCompleted = true;
 HeroSelectionWidget->StopSelection(); HeroSelectionWidget->RemoveFromParent(); HeroSelectionWidget = nullptr;
 bHudConsumesInput = false; CreateOmHud();
 UWidgetBlueprintLibrary::SetInputMode_GameAndUIEx(this, HudRootWidget, EMouseLockMode::DoNotLock, false);
 return true;
}
```
BlueprintCallable sole reflected gateway. BeginPlay still selection. Tick/CreateOmHud/IsHudConsumingInput now use IsHeroSelectionPending. All member bools default false per instance; no FCommandLine/env mutation. HUD native fallback already exists.

Subsystem native-only gateway:
```cpp
bool UOmRuntimeBridgeSubsystem::StartPresentationAfterHeroSelection() {
 if (!FParse::Param(FCommandLine::Get(), TEXT("om-hero-selection"))
  || !FParse::Param(FCommandLine::Get(), TEXT("om-selection-in-place"))) return false;
 const UOmRuntimeSettings* Settings = GetDefault<UOmRuntimeSettings>();
 if (!Settings || ResolveRuntimeMode(Settings) != EOmRuntimeMode::PresentationIpc
  || ResolvePresentationAddress(Settings).IsEmpty()) return false;
 const bool bPrevious = bHeroSelectionCompleted; bHeroSelectionCompleted = true;
 if (StartRuntimeFromSettings()) return true;
 bHeroSelectionCompleted = bPrevious; return false;
}
```
StartRuntimeFromSettings existing guard now rejects selection iff !bHeroSelectionCompleted; otherwise original normal IPC startup and compiled-only mode. Init still skips LoadBridge during selection. WorldBridge already ticks whenever runtime started and catalog available; no actor changes.

Fixed-Lua entry owns fresh Editor and private native Workflow Job. Prevalidates Rust candidate, boots same PIE world with selection flags and predetermined player/team/presentation address. Accepts saved receipt through existing strict selection_result comparison against original host-owned rules; only hero choice can change. Re-prepares authoritative recipe, starts ONE server and ONE client-runtime, waits original runtime readiness, then calls controller method via project-bound MCP. Each MCP request now also checks original editor executable/creation-token lifetime and registry PID. Natural native-visible result+actual screenshot+live PIE status required; no forced winner, no gameplay Lua, no PID/name cleanup or adopting another editor. Output is exclusively mkdir reserved. Ledger repeated writes explicitly overwrite only this workflow's own JSON. No changes to original role default launch.

UE NoEngineChanges build succeeded37.01s. First real PIE selection has select/lock/finalize pointer receipts and no gameplay bridge beforehand; subsequent server launch hit own-ledger overwrite guard, cleanup passed. Fixed ledger; actual second same-world flow running, no result claim yet. Exact scope excludes actual2machineLAN and performance compliance.
