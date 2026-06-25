#!/usr/bin/env bash
set -u
# NOTE: no set -e — we handle errors explicitly

RESULTS_DIR="/tmp/gguy_stress_results"
GODOT="${GODOT_BIN:-godot}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ALL_PASS=true
TIMEOUT=120

if ! command -v "$GODOT" &>/dev/null; then
    echo "ERROR: '$GODOT' not found. Set GODOT_BIN env var to the godot binary path."
    exit 1
fi

mkdir -p "$RESULTS_DIR" || { echo "ERROR: cannot create $RESULTS_DIR"; exit 1; }

run_test() {
    local name="$1"
    local subtest="$2"
    local log_file="$RESULTS_DIR/${name}.log"
    local verdict_file="$RESULTS_DIR/${name}_verdict.txt"

    echo ""
    echo "=== Running: $name $subtest ==="

    local wrap=()
    if command -v timeout &>/dev/null; then
        wrap=("timeout" "$TIMEOUT")
    else
        echo "  (no 'timeout' command — running without timeout)"
    fi

    local cmd=("${wrap[@]}" "$GODOT" "--headless" "--log-file" "$log_file" "--path" "$PROJECT_DIR")
    if [[ -n "$subtest" ]]; then
        IFS=' ' read -ra extra_args <<< "$subtest"
        for arg in "${extra_args[@]}"; do
            cmd+=("$arg")
        done
    fi
    cmd+=("res://stress/${name}.tscn")


    local output
    set +e
    output=$("${cmd[@]}" 2>&1)
    local exit_code=$?
    set -e
    echo "$output" > "${log_file}.stdout"

    local verdict
    verdict=$(echo "$output" | grep -E "^=== TEST (COMPLETE|FAILED)" | tail -1)

    if [[ -z "$verdict" ]]; then
        if [[ $exit_code -eq 124 ]]; then
            verdict="=== TEST FAILED: timeout (${TIMEOUT}s) ==="
        else
            echo "$output" | tail -5 > "$verdict_file"
            verdict="=== TEST FAILED: no verdict line found (exit=$exit_code) ==="
        fi
    fi

    echo "$verdict" > "$verdict_file"
    echo "$verdict"
    if echo "$verdict" | grep -q "FAILED"; then
        ALL_PASS=false
    fi
}

run_test "test_complexity" ""
run_test "test_mutations" ""
run_test "test_multisurface" "--surfaces 2"
run_test "test_multisurface" "--surfaces 4"
run_test "test_multisurface" "--surfaces 8"

echo ""
echo "============================================"
echo "         Gguy Stress Test Results           "
echo "============================================"
for f in "$RESULTS_DIR"/*_verdict.txt; do
    [[ -f "$f" ]] || continue
    name=$(basename "$f" _verdict.txt)
    verdict=$(cat "$f")
    echo "  $name: $verdict"
done
echo "============================================"

if $ALL_PASS; then
    echo "All tests passed."
    exit 0
else
    echo "Some tests failed."
    exit 1
fi
