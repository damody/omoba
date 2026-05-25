#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMFUE="${ROOT}/omfue"
PROJECT="${OMFUE}/om.uproject"
UE_PROJECT_NAME="$(basename "${PROJECT}" .uproject)"
UE_MAP="/Game/Map/Main"
RUN_MODE="game"
SMOKE_SECONDS=90
SKIP_BUILD=0
RUN_BACKEND=0
UE_RUNTIME_ARG="-om-single-player"
UE_RHI_ARGS=("-vulkan")
UE_BUILD_ARGS=("-WaitMutex" "-NoHotReloadFromIDE")
UE_UAT_INCREMENTAL_ARGS=("-iterate" "-iterativecooking" "-cookincremental" "-nocleanstage")

usage() {
  cat <<'USAGE'
Usage: ./run_ue.sh [--editor|--headless-smoke|--build-only|--game-smoke|--safe]
                   [--networked|--with-backend|--single-player]
                   [--seconds N] [--no-build] [--vulkan|--opengl]

Ubuntu/Linux equivalent of run_ue.bat. Builds are always incremental: Rust uses
Cargo's normal incremental cache, UE editor builds do not clean, and standalone
BuildCookRun always uses iterative cook/deploy with a preserved staged dir.
USAGE
}

if [[ -n "${UE_5_7_ROOT:-}" ]]; then
  UE_ROOT_RESOLVED="${UE_5_7_ROOT}"
  UE_ROOT_SOURCE="UE_5_7_ROOT"
elif [[ -n "${UE_ROOT:-}" ]]; then
  UE_ROOT_RESOLVED="${UE_ROOT}"
  UE_ROOT_SOURCE="UE_ROOT"
else
  UE_ROOT_RESOLVED="/home/damody/work/UE5.7"
  UE_ROOT_SOURCE="default"
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --editor)
      RUN_MODE="editor"
      shift
      ;;
    --headless-smoke)
      RUN_MODE="headless"
      shift
      ;;
    --build-only)
      RUN_MODE="build-only"
      shift
      ;;
    --game-smoke)
      RUN_MODE="game-smoke"
      shift
      ;;
    --safe)
      RUN_MODE="safe"
      shift
      ;;
    --vulkan)
      UE_RHI_ARGS=("-vulkan")
      shift
      ;;
    --opengl)
      UE_RHI_ARGS=("-opengl4")
      shift
      ;;
    --networked|--with-backend)
      RUN_BACKEND=1
      UE_RUNTIME_ARG="-om-networked"
      shift
      ;;
    --single-player)
      RUN_BACKEND=0
      UE_RUNTIME_ARG="-om-single-player"
      shift
      ;;
    --seconds)
      if [[ $# -lt 2 || ! "$2" =~ ^[0-9]+$ ]]; then
        echo "[run_ue] --seconds requires a positive integer." >&2
        exit 2
      fi
      SMOKE_SECONDS="$2"
      shift 2
      ;;
    --no-build)
      SKIP_BUILD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[run_ue] Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

UE_EDITOR="${UE_ROOT_RESOLVED}/Engine/Binaries/Linux/UnrealEditor"
UE_EDITOR_CMD="${UE_ROOT_RESOLVED}/Engine/Binaries/Linux/UnrealEditor-Cmd"
UE_BUILD="${UE_ROOT_RESOLVED}/Engine/Build/BatchFiles/Linux/Build.sh"
UE_UAT="${UE_ROOT_RESOLVED}/Engine/Build/BatchFiles/RunUAT.sh"
UE_STAGED_DIR="${OMFUE}/Saved/StagedBuilds/Linux"
BACKEND_EXE="${ROOT}/omb/target/debug/omobab"
BASE_CONTENT_TARGET="${ROOT}/scripts/target/debug/libbase_content.so"
BASE_CONTENT_STAGED="${ROOT}/scripts/libbase_content.so"
PLUGIN_BIN="${OMFUE}/Plugins/OmRuntime/Binaries/Linux"
GENERATED_OUT="${OMFUE}/Plugins/OmRuntime/Source/OmGenerated"
BRIDGE_MANIFEST="${OMFUE}/bridge/Cargo.toml"
CODEGEN_MANIFEST="${OMFUE}/codegen/Cargo.toml"
BRIDGE_TARGET="${OMFUE}/bridge/target/debug"
BRIDGE_INCLUDE="${OMFUE}/Plugins/OmRuntime/Source/ThirdParty/OmBridge/include"
BRIDGE_HEADER_TMP="${OMFUE}/Saved/BuildBridge/om_bridge.h"
BACKEND_PID_FILE="${ROOT}/omb/log/run_ue_backend.pid"
BACKEND_STDOUT="${ROOT}/omb/log/run_ue_backend_stdout.log"
BACKEND_STDERR="${ROOT}/omb/log/run_ue_backend_stderr.log"
UE_STDOUT="${OMFUE}/Saved/Logs/run_ue_stdout.log"
UE_STDERR="${OMFUE}/Saved/Logs/run_ue_stderr.log"
BACKEND_PID=""
UE_RUN_EXE=""
UE_STAGE_ROOT=""

export OMB_GAME_TOML="${ROOT}/omb/game.toml"
export OMB_LUA_CONTENT=1
export OMB_LUA_CONTENT_ROOT="${ROOT}/scripts/lua_data"
export OMB_STORY_DATA_DIR="${ROOT}/scripts/lua_data"
export OMB_SCRIPTS_DIR="${ROOT}/scripts"
export OMB_DLL_PATH="${BASE_CONTENT_STAGED}"
export OMB_STORY="${OMB_STORY:-TD_1}"

require_path() {
  if [[ ! -e "$1" ]]; then
    echo "[run_ue] missing: $1" >&2
    exit 1
  fi
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[run_ue] $1 was not found in PATH." >&2
    exit 1
  fi
}

copy_if_changed() {
  local src="$1"
  local dst="$2"
  local label="$3"
  require_path "$src"
  mkdir -p "$(dirname "$dst")"
  if [[ -f "$dst" ]] && cmp -s "$src" "$dst"; then
    echo "[run_ue] unchanged: ${label}"
  else
    cp -f "$src" "$dst"
    echo "[run_ue] updated: ${label}"
  fi
}

stop_backend() {
  if [[ -n "${BACKEND_PID}" ]]; then
    echo "[run_ue] stopping backend PID ${BACKEND_PID}..."
    kill "${BACKEND_PID}" >/dev/null 2>&1 || true
    wait "${BACKEND_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -f "${BACKEND_PID_FILE}" ]]; then
    rm -f "${BACKEND_PID_FILE}"
  fi
  BACKEND_PID=""
}

fail() {
  stop_backend
  exit 1
}

trap stop_backend EXIT INT TERM

build_all() {
  pushd "${ROOT}" >/dev/null
  mkdir -p "${PLUGIN_BIN}" "${BRIDGE_INCLUDE}" "$(dirname "${BRIDGE_HEADER_TMP}")"

  echo "[run_ue] [1/5] building script shared library with runtime Lua content..."
  cargo build --manifest-path "${ROOT}/scripts/Cargo.toml" -p base_content --features runtime-lua-content
  copy_if_changed "${BASE_CONTENT_TARGET}" "${BASE_CONTENT_STAGED}" "scripts/libbase_content.so"
  mkdir -p "${ROOT}/omb/scripts"
  copy_if_changed "${BASE_CONTENT_TARGET}" "${ROOT}/omb/scripts/libbase_content.so" "omb/scripts/libbase_content.so"
  copy_if_changed "${BASE_CONTENT_TARGET}" "${PLUGIN_BIN}/libbase_content.so" "OmRuntime libbase_content.so"

  if [[ "${RUN_BACKEND}" -eq 1 ]]; then
    echo "[run_ue] [2/5] building Rust backend with runtime Lua content..."
    cargo build --manifest-path "${ROOT}/omb/Cargo.toml" -p omobab --features runtime-lua-content
  else
    echo "[run_ue] [2/5] skipping Rust backend build for single-player runtime."
  fi

  echo "[run_ue] [3/5] generating and staging UE bridge..."
  cargo run --manifest-path "${CODEGEN_MANIFEST}" -- --content-root "${ROOT}/scripts/lua_data" --out "${GENERATED_OUT}"
  cargo build --manifest-path "${BRIDGE_MANIFEST}"
  copy_if_changed "${BRIDGE_TARGET}/libom_bridge.so" "${PLUGIN_BIN}/libom_bridge.so" "OmRuntime libom_bridge.so"

  echo "[run_ue] [4/5] checking generated bridge/code freshness..."
  if command -v cbindgen >/dev/null 2>&1; then
    cbindgen --config "${OMFUE}/bridge/cbindgen.toml" --crate om_bridge --output "${BRIDGE_HEADER_TMP}" "${OMFUE}/bridge"
    copy_if_changed "${BRIDGE_HEADER_TMP}" "${BRIDGE_INCLUDE}/om_bridge.h" "om_bridge.h"
  elif [[ -f "${BRIDGE_INCLUDE}/om_bridge.h" ]]; then
    echo "[run_ue] cbindgen not found; reusing existing ${BRIDGE_INCLUDE}/om_bridge.h"
  else
    echo "[run_ue] cbindgen not found and om_bridge.h is missing." >&2
    popd >/dev/null
    return 1
  fi

  if [[ "${RUN_MODE}" == "editor" || "${RUN_MODE}" == "headless" || "${RUN_MODE}" == "safe" || "${RUN_MODE}" == "build-only" ]]; then
    echo "[run_ue] [5/5] building UE editor target..."
    "${UE_BUILD}" OmGameEditor Linux Development -Project="${PROJECT}" "${UE_BUILD_ARGS[@]}"
  else
    echo "[run_ue] [5/5] building, cooking, and staging UE standalone game incrementally..."
    "${UE_UAT}" BuildCookRun -project="${PROJECT}" -noP4 -platform=Linux -clientconfig=Development -build -cook -stage "${UE_UAT_INCREMENTAL_ARGS[@]}" -map="${UE_MAP}" -NoCompileEditor -unattended -utf8output
    resolve_staged_exe
    stage_runtime_script_lib
    echo "[run_ue] staged UE game: ${UE_RUN_EXE}"
  fi

  popd >/dev/null
}

start_backend() {
  echo "[run_ue] starting Rust backend..."
  mkdir -p "${ROOT}/omb/log"
  if [[ -f "${BACKEND_PID_FILE}" ]]; then
    local old_pid
    old_pid="$(cat "${BACKEND_PID_FILE}" 2>/dev/null || true)"
    if [[ -n "${old_pid}" ]]; then
      kill "${old_pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${BACKEND_PID_FILE}"
  fi
  pkill -f "${BACKEND_EXE}" >/dev/null 2>&1 || true

  (
    cd "${ROOT}/omb"
    "${BACKEND_EXE}"
  ) >"${BACKEND_STDOUT}" 2>"${BACKEND_STDERR}" &
  BACKEND_PID=$!
  echo "${BACKEND_PID}" >"${BACKEND_PID_FILE}"

  sleep 1.5
  if ! kill -0 "${BACKEND_PID}" >/dev/null 2>&1; then
    echo "[run_ue] backend failed to start." >&2
    [[ -f "${BACKEND_STDOUT}" ]] && cat "${BACKEND_STDOUT}" >&2
    [[ -f "${BACKEND_STDERR}" ]] && cat "${BACKEND_STDERR}" >&2
    rm -f "${BACKEND_PID_FILE}"
    BACKEND_PID=""
    return 1
  fi
  echo "[run_ue] backend PID: ${BACKEND_PID}"
}

resolve_staged_exe() {
  local candidate
  UE_RUN_EXE=""
  UE_STAGE_ROOT=""

  for candidate in \
    "${UE_STAGED_DIR}/${UE_PROJECT_NAME}/Binaries/Linux/OmGame" \
    "${UE_STAGED_DIR}/${UE_PROJECT_NAME}.sh" \
    "${UE_STAGED_DIR}/OmGame.sh" \
    "${UE_STAGED_DIR}/OmGame/Binaries/Linux/OmGame" \
    "${UE_STAGED_DIR}/LinuxNoEditor/OmGame/Binaries/Linux/OmGame"; do
    if [[ -f "${candidate}" ]]; then
      UE_RUN_EXE="${candidate}"
      UE_STAGE_ROOT="${UE_STAGED_DIR}"
      return 0
    fi
  done

  candidate="$(find "${OMFUE}/Saved/StagedBuilds" -type f \( -name 'OmGame' -o -name 'OmGame.sh' \) -print -quit 2>/dev/null || true)"
  if [[ -n "${candidate}" ]]; then
    UE_RUN_EXE="${candidate}"
    UE_STAGE_ROOT="$(cd "$(dirname "${candidate}")/../../.." 2>/dev/null && pwd || dirname "${candidate}")"
    return 0
  fi

  return 1
}

stage_runtime_script_lib() {
  if [[ -z "${UE_RUN_EXE}" ]]; then
    echo "[run_ue] cannot stage script library before resolving UE_RUN_EXE." >&2
    return 1
  fi
  local source_lib="${PLUGIN_BIN}/libbase_content.so"
  local staged_plugin_bin
  staged_plugin_bin="${UE_STAGE_ROOT}/${UE_PROJECT_NAME}/Plugins/OmRuntime/Binaries/Linux"
  if [[ ! -f "${source_lib}" ]]; then
    echo "[run_ue] missing source script library for staging: ${source_lib}" >&2
    return 1
  fi
  copy_if_changed "${source_lib}" "${staged_plugin_bin}/libbase_content.so" "staged libbase_content.so"
}

assert_runtime_started() {
  local log_file="${OMFUE}/Saved/Logs/om.log"
  if [[ -f "${log_file}" ]] && grep -q "Started bridge runtime" "${log_file}"; then
    echo "[run_ue] UE smoke reached bridge runtime startup."
    return 0
  fi
  if [[ -f "${UE_STDOUT}" ]] && grep -q "Started bridge runtime" "${UE_STDOUT}"; then
    echo "[run_ue] UE smoke reached bridge runtime startup."
    return 0
  fi
  echo "[run_ue] UE smoke did not reach bridge runtime startup. Recent UE log:" >&2
  [[ -f "${log_file}" ]] && tail -80 "${log_file}" >&2
  [[ -f "${UE_STDOUT}" ]] && tail -80 "${UE_STDOUT}" >&2
  return 1
}

run_bounded() {
  local exe="$1"
  shift
  mkdir -p "${OMFUE}/Saved/Logs"
  rm -f "${OMFUE}/Saved/Logs/om.log" "${UE_STDOUT}" "${UE_STDERR}"
  set +e
  timeout --preserve-status "${SMOKE_SECONDS}s" "${exe}" "$@" >"${UE_STDOUT}" 2>"${UE_STDERR}"
  local status=$?
  set -e
  if [[ "${status}" -eq 124 || "${status}" -eq 143 ]]; then
    status=0
  fi
  if [[ "${status}" -ne 0 ]]; then
    echo "[run_ue] UE process exited with ${status}." >&2
    tail -80 "${UE_STDOUT}" >&2 || true
    tail -80 "${UE_STDERR}" >&2 || true
    return "${status}"
  fi
  assert_runtime_started
}

run_frontend() {
  case "${RUN_MODE}" in
    editor)
      echo "[run_ue] launching UE editor. Press Play to run the frontend world."
      "${UE_EDITOR}" "${PROJECT}" "${UE_RUNTIME_ARG}"
      ;;
    headless)
      echo "[run_ue] launching headless UE frontend smoke for ${SMOKE_SECONDS}s..."
      run_bounded "${UE_EDITOR_CMD}" "${PROJECT}" "${UE_MAP}" -game "${UE_RUNTIME_ARG}" -NullRHI -unattended -nop4 -nosplash -stdout -FullStdOutLogOutput -log
      ;;
    safe)
      echo "[run_ue] launching safe UE frontend smoke with NullRHI for ${SMOKE_SECONDS}s..."
      run_bounded "${UE_EDITOR}" "${PROJECT}" "${UE_MAP}" -game "${UE_RUNTIME_ARG}" -NullRHI -unattended -nop4 -nosplash -stdout -FullStdOutLogOutput -log
      ;;
    game-smoke)
      echo "[run_ue] launching bounded standalone UE frontend smoke for ${SMOKE_SECONDS}s..."
      resolve_staged_exe || {
        echo "[run_ue] missing staged UE game executable. Run without --no-build once to cook and stage it." >&2
        return 1
      }
      stage_runtime_script_lib
      run_bounded "${UE_RUN_EXE}" "${UE_MAP}" "${UE_RHI_ARGS[@]}" "${UE_RUNTIME_ARG}" -noraytracing -windowed -ResX=1280 -ResY=720 -unattended -nop4 -nosplash -stdout -FullStdOutLogOutput -log
      ;;
    game)
      resolve_staged_exe || {
        echo "[run_ue] missing staged UE game executable. Run without --no-build once to cook and stage it." >&2
        return 1
      }
      stage_runtime_script_lib
      echo "[run_ue] launching UE standalone game frontend..."
      pushd "$(dirname "${UE_RUN_EXE}")" >/dev/null
      "${UE_RUN_EXE}" "${UE_MAP}" "${UE_RHI_ARGS[@]}" "${UE_RUNTIME_ARG}" -noraytracing -windowed -ResX=1280 -ResY=720 -unattended -nop4 -nosplash -stdout -FullStdOutLogOutput -log
      local ue_exit=$?
      popd >/dev/null
      return "${ue_exit}"
      ;;
    *)
      echo "[run_ue] unsupported mode: ${RUN_MODE}" >&2
      return 2
      ;;
  esac
}

echo "[run_ue] root: ${ROOT}"
echo "[run_ue] UE root: ${UE_ROOT_RESOLVED} (${UE_ROOT_SOURCE})"
echo "[run_ue] project: ${PROJECT}"
echo "[run_ue] mode: ${RUN_MODE}"
if [[ "${RUN_BACKEND}" -eq 1 ]]; then
  echo "[run_ue] runtime: networked backend"
else
  echo "[run_ue] runtime: single-player local simulation"
fi

require_path "${PROJECT}"
require_path "${UE_EDITOR}"
require_path "${UE_EDITOR_CMD}"
require_path "${UE_BUILD}"
require_path "${UE_UAT}"
require_command cargo

if [[ "${SKIP_BUILD}" -eq 0 ]]; then
  build_all || fail
else
  echo "[run_ue] skipping build because --no-build was specified."
fi

if [[ "${RUN_MODE}" == "build-only" ]]; then
  echo "[run_ue] build-only completed."
  exit 0
fi

if [[ "${RUN_BACKEND}" -eq 1 ]]; then
  start_backend || fail
else
  echo "[run_ue] backend not started because single-player runtime is enabled."
fi

run_frontend
