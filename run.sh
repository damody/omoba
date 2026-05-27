#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
REPO_ROOT="$(pwd)"

if ! command -v cargo >/dev/null 2>&1 && [[ -f "${HOME}/.cargo/env" ]]; then
  # Keep the script usable from non-interactive shells that do not load ~/.profile.
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust with rustup, then rerun ./run.sh." >&2
  exit 1
fi

setup_pkg_config_fallback() {
  local pc_name="$1"
  local lib_name="$2"
  local header_hint="$3"
  local runtime_lib

  if pkg-config --exists "$pc_name" >/dev/null 2>&1; then
    return 0
  fi

  runtime_lib="$(find /usr/lib /lib -name "lib${lib_name}.so*" 2>/dev/null | sort | head -n 1 || true)"
  if [[ -z "$runtime_lib" ]]; then
    return 0
  fi

  mkdir -p "${REPO_ROOT}/.run-local/lib" "${REPO_ROOT}/.run-local/pkgconfig"
  ln -sf "$runtime_lib" "${REPO_ROOT}/.run-local/lib/lib${lib_name}.so"
  cat >"${REPO_ROOT}/.run-local/pkgconfig/${pc_name}.pc" <<EOF
prefix=${REPO_ROOT}/.run-local
exec_prefix=\${prefix}
libdir=\${prefix}/lib
includedir=/usr/include

Name: ${pc_name}
Description: local fallback for ${header_hint}
Version: 0
Libs: -L\${libdir} -l${lib_name}
Cflags: -I\${includedir}
EOF
}

export PKG_CONFIG_PATH="${REPO_ROOT}/.run-local/pkgconfig${PKG_CONFIG_PATH:+:${PKG_CONFIG_PATH}}"
export LIBRARY_PATH="${REPO_ROOT}/.run-local/lib${LIBRARY_PATH:+:${LIBRARY_PATH}}"
setup_pkg_config_fallback "alsa" "asound" "ALSA"
setup_pkg_config_fallback "x11" "X11" "X11"
setup_pkg_config_fallback "xcursor" "Xcursor" "Xcursor"
setup_pkg_config_fallback "xi" "Xi" "Xi"
setup_pkg_config_fallback "xrandr" "Xrandr" "Xrandr"
setup_pkg_config_fallback "xinerama" "Xinerama" "Xinerama"
setup_pkg_config_fallback "gl" "GL" "OpenGL"
setup_pkg_config_fallback "wayland-client" "wayland-client" "Wayland client"
setup_pkg_config_fallback "xkbcommon" "xkbcommon" "xkbcommon"

# Options:
#   --trace  Enable omfx Perfetto trace. The output can be customized with
#            OMFX_PERFETTO_PATH / OMFX_PERFETTO_DETAIL / OMFX_PERFETTO_MAX_SECONDS.

case "$(uname -s)" in
  Darwin*) DYLIB_EXT="dylib" ;;
  Linux*) DYLIB_EXT="so" ;;
  *)
    echo "Unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

EXECUTOR="omfx/target/debug/executor"
BACKEND="omb/target/debug/omobab"
SCRIPT_LIB="scripts/target/debug/libbase_content.${DYLIB_EXT}"
STAGED_SCRIPT_LIB="scripts/libbase_content.${DYLIB_EXT}"

export OMFX_BACKEND_EXE="${REPO_ROOT}/${BACKEND}"
export OMB_GAME_TOML="${REPO_ROOT}/omb/game.toml"
export OMFX_GAME_TOML="${REPO_ROOT}/omfx/game.toml"
export OMB_STORY="TD_1"
if [[ -n "${OMB_SCENE_PATH:-}" ]]; then
  export OMB_SCENE_PATH
else
  unset OMB_SCENE_PATH
fi
export CARGO_PROFILE_DEV_DEBUG=false
export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=false
export RUSTFLAGS="-C debuginfo=0"

RUN_TRACE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --trace)
      RUN_TRACE=1
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -n "$RUN_TRACE" ]]; then
  export OMFX_PERFETTO_TRACE=1
  export OMFX_PERFETTO_DETAIL="${OMFX_PERFETTO_DETAIL:-frame}"
  export OMFX_PERFETTO_PATH="${OMFX_PERFETTO_PATH:-omfx/target/profiles/run.perfetto-trace}"
  echo "Perfetto trace enabled for run."
  echo "  -> trace path: ${OMFX_PERFETTO_PATH}"
fi

BACKEND_PID=""
BACKEND_PID_FILE="omb/log/launcher_backend.pid"

stop_backend() {
  if [[ -n "${BACKEND_PID}" ]]; then
    echo "Stopping backend PID ${BACKEND_PID}..."
    kill "${BACKEND_PID}" >/dev/null 2>&1 || true
    wait "${BACKEND_PID}" >/dev/null 2>&1 || true
    rm -f "${BACKEND_PID_FILE}"
    BACKEND_PID=""
  fi
}

fail() {
  stop_backend
  exit 1
}

trap stop_backend EXIT INT TERM

newest_input() {
  local paths=()
  local path
  for path in "$@"; do
    if [[ -e "$path" ]]; then
      paths+=("$path")
    fi
  done

  if [[ "${#paths[@]}" -eq 0 ]]; then
    return 0
  fi

  find "${paths[@]}" -type f -printf '%T@ %p\n' | sort -nr | head -n 1
}

artifact_is_fresh() {
  local name="$1"
  local output="$2"
  local fingerprint_dir="$3"
  local fingerprint_file="$4"
  shift 4

  if [[ ! -f "$output" ]]; then
    echo "stale: ${name} output missing: ${output}"
    return 1
  fi

  if [[ -n "$fingerprint_dir" ]]; then
    if [[ ! -d "$fingerprint_dir" ]]; then
      echo "stale: ${name} fingerprint directory missing: ${fingerprint_dir}"
      return 1
    fi

    local latest_fingerprint
    latest_fingerprint="$(find "$fingerprint_dir" -type f -name "$fingerprint_file" -printf '%T@ %p\n' | sort -nr | head -n 1 | cut -d' ' -f2-)"
    if [[ -z "$latest_fingerprint" ]]; then
      echo "stale: ${name} fingerprint missing: ${fingerprint_file}"
      return 1
    fi
    if ! grep -q 'runtime-lua-content' "$latest_fingerprint"; then
      echo "stale: ${name} latest fingerprint missing feature 'runtime-lua-content': ${latest_fingerprint}"
      return 1
    fi
  fi

  local newest
  newest="$(newest_input "$@" | cut -d' ' -f2-)"
  if [[ -z "$newest" ]]; then
    echo "freshness check failed: no inputs found for ${name}" >&2
    return 2
  fi

  if [[ "$newest" -nt "$output" ]]; then
    echo "stale: ${name} input newer than output: ${newest}"
    return 1
  fi

  echo "fresh: ${name} output is up-to-date: ${output}"
  return 0
}

ensure_fresh() {
  local artifact="$1"
  local label="$2"
  local output="$3"
  local fingerprint_dir="$4"
  local fingerprint_file="$5"
  local build_cmd="$6"
  shift 6

  set +e
  artifact_is_fresh "${artifact}-debug" "$output" "$fingerprint_dir" "$fingerprint_file" "$@"
  local fresh_err=$?
  set -e

  if [[ "$fresh_err" -eq 0 ]]; then
    echo "  -> ${label} up-to-date; skipping build."
    return 0
  fi
  if [[ "$fresh_err" -ne 1 ]]; then
    echo "  -> freshness check failed for ${label}; aborting."
    return 1
  fi

  echo "  -> ${label} stale; building..."
  eval "$build_cmd"
}

stage_script_lib() {
  if [[ ! -f "$SCRIPT_LIB" ]]; then
    echo "Script library missing: ${SCRIPT_LIB}" >&2
    return 1
  fi

  mkdir -p "$(dirname "$STAGED_SCRIPT_LIB")"
  if [[ -f "$STAGED_SCRIPT_LIB" ]] && cmp -s "$SCRIPT_LIB" "$STAGED_SCRIPT_LIB" && [[ ! "$SCRIPT_LIB" -nt "$STAGED_SCRIPT_LIB" ]]; then
    echo "fresh: staged script library is up-to-date: ${STAGED_SCRIPT_LIB}"
    return 0
  fi

  cp -f "$SCRIPT_LIB" "$STAGED_SCRIPT_LIB"
  touch -r "$SCRIPT_LIB" "$STAGED_SCRIPT_LIB"
  echo "staged: copied $(basename "$SCRIPT_LIB") to scripts/"
}

start_backend() {
  mkdir -p omb/log
  rm -f "$BACKEND_PID_FILE" omb/log/launcher_backend_stdout.log omb/log/launcher_backend_stderr.log

  (
    cd omb
    "../${BACKEND}"
  ) >omb/log/launcher_backend_stdout.log 2>omb/log/launcher_backend_stderr.log &
  BACKEND_PID=$!
  echo "${BACKEND_PID}" >"$BACKEND_PID_FILE"

  sleep 1.5
  if ! kill -0 "$BACKEND_PID" >/dev/null 2>&1; then
    rm -f "$BACKEND_PID_FILE"
    echo "Backend process exited during startup. See omb/log/launcher_backend_stdout.log and omb/log/launcher_backend_stderr.log." >&2
    return 1
  fi

  echo "  -> backend PID ${BACKEND_PID}"
}

echo "[0/5] Killing stale processes (if any)..."
pkill -f "${REPO_ROOT}/omb/target/debug/omobab" >/dev/null 2>&1 || true
pkill -f "${REPO_ROOT}/omfx/target/debug/executor" >/dev/null 2>&1 || true

echo "[1/5] Checking script library (scripts/base_content)..."
ensure_fresh "script-dll" "script library" "$SCRIPT_LIB" "scripts/target/debug/.fingerprint" "lib-base_content.json" \
  "cargo build --manifest-path scripts/Cargo.toml -p base_content --features runtime-lua-content" \
  rust-toolchain.toml scripts/Cargo.toml scripts/Cargo.lock scripts/base_content/Cargo.toml scripts/base_content/src \
  scripts/script-abi/Cargo.toml scripts/script-abi/src omoba-core/Cargo.toml omoba-core/Cargo.lock omoba-core/build.rs \
  proto/game.proto omoba-core/src omoba-template-ids/Cargo.toml omoba-template-ids/build.rs omoba-template-ids/src \
  omoba-sim/Cargo.toml omoba-sim/Cargo.lock omoba-sim/src || fail
stage_script_lib || fail

echo "[2/5] Checking backend (omb)..."
ensure_fresh "backend" "backend" "$BACKEND" "omb/target/debug/.fingerprint" "bin-omobab.json" \
  "cargo build --manifest-path omb/Cargo.toml --features runtime-lua-content" \
  rust-toolchain.toml scripts/script-abi/Cargo.toml scripts/script-abi/src omoba-core/Cargo.toml omoba-core/Cargo.lock \
  omoba-core/build.rs proto/game.proto omoba-core/src omoba-template-ids/Cargo.toml omoba-template-ids/build.rs \
  omoba-template-ids/src omoba-sim/Cargo.toml omoba-sim/Cargo.lock omoba-sim/src specs/Cargo.toml specs/Cargo.lock \
  specs/src log4rs/Cargo.toml log4rs/Cargo.lock log4rs/src omb/Cargo.toml omb/Cargo.lock omb/build.rs omb/src || fail

echo "[3/5] Checking frontend (omfx executor)..."
ensure_fresh "frontend" "frontend" "$EXECUTOR" "omfx/target/debug/.fingerprint" "bin-executor.json" \
  "cargo build --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content" \
  rust-toolchain.toml scripts/script-abi/Cargo.toml scripts/script-abi/src omoba-core/Cargo.toml omoba-core/Cargo.lock \
  omoba-core/build.rs proto/game.proto omoba-core/src omoba-template-ids/Cargo.toml omoba-template-ids/build.rs \
  omoba-template-ids/src omoba-sim/Cargo.toml omoba-sim/Cargo.lock omoba-sim/src specs/Cargo.toml specs/Cargo.lock \
  specs/src log4rs/Cargo.toml log4rs/Cargo.lock log4rs/src omfx/Cargo.toml omfx/Cargo.lock omfx/executor/Cargo.toml \
  omfx/game/Cargo.toml omfx/executor/src omfx/game/src third_party/fyrox-impl-1.0.1/src || fail

if [[ ! -x "$BACKEND" ]]; then
  echo "Backend executable missing: ${BACKEND}" >&2
  fail
fi
if [[ ! -x "$EXECUTOR" ]]; then
  echo "Frontend executable missing: ${EXECUTOR}" >&2
  fail
fi

echo "[4/5] Running frontend..."
echo "  -> frontend session launcher will start backend: ${OMFX_BACKEND_EXE}"
"./${EXECUTOR}"
