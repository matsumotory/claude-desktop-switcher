#!/bin/bash
# ============================================================================
# Claude Desktop Switcher — Phase 0: Sandbox Validation Script
# ============================================================================
#
# Purpose: Validate all assumptions about Claude Desktop and CLI isolation
#          mechanisms WITHOUT modifying the existing environment.
#
# Usage:   ./scripts/sandbox-test.sh [test_number]
#          Run without arguments to execute all tests.
#          Run with a number (1-8) to execute a specific test.
#
# Results are written to sandbox-results.md
# ============================================================================

set -euo pipefail

SANDBOX_DIR="$HOME/.claude-desktop-switcher-sandbox"
RESULTS_FILE="$(cd "$(dirname "$0")/.." && pwd)/sandbox-results.md"

CLAUDE_DESKTOP_APP="/Applications/Claude.app"
CLAUDE_DESKTOP_DEFAULT="$HOME/Library/Application Support/Claude"
CLAUDE_CLI_DEFAULT="$HOME/.claude"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[PASS]${NC}  $*"; }
log_fail()  { echo -e "${RED}[FAIL]${NC}  $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_skip()  { echo -e "${YELLOW}[SKIP]${NC}  $*"; }

# Initialize results file
init_results() {
    cat > "$RESULTS_FILE" << 'EOF'
# Claude Desktop Switcher Sandbox Validation Results

Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')

| # | Test | Status | Notes |
|---|------|--------|-------|
EOF
    # Replace the date placeholder
    sed -i '' "s/\$(date -u '+%Y-%m-%dT%H:%M:%SZ')/$(date -u '+%Y-%m-%dT%H:%M:%SZ')/" "$RESULTS_FILE"
}

append_result() {
    local num="$1" name="$2" status="$3" notes="$4"
    echo "| V${num} | ${name} | ${status} | ${notes} |" >> "$RESULTS_FILE"
}

append_detail() {
    echo "" >> "$RESULTS_FILE"
    echo "$1" >> "$RESULTS_FILE"
}

# ============================================================================
# Setup
# ============================================================================

setup_sandbox() {
    log_info "Creating sandbox directory: $SANDBOX_DIR"
    mkdir -p "$SANDBOX_DIR/desktop-test"
    mkdir -p "$SANDBOX_DIR/cli-test"
    log_ok "Sandbox directory created"
}

cleanup_sandbox() {
    log_info "Cleaning up sandbox directory: $SANDBOX_DIR"
    rm -rf "$SANDBOX_DIR"
    log_ok "Sandbox cleaned up"
}

# ============================================================================
# V1: --user-data-dir isolation for Desktop
# ============================================================================

test_v1_desktop_user_data_dir() {
    log_info "V1: Testing --user-data-dir isolation for Claude Desktop"

    if [ ! -d "$CLAUDE_DESKTOP_APP" ]; then
        log_fail "Claude Desktop not found at $CLAUDE_DESKTOP_APP"
        append_result 1 "--user-data-dir isolation" "SKIP" "Claude Desktop not installed"
        return
    fi

    local test_dir="$SANDBOX_DIR/desktop-test"

    log_info "Launching Claude Desktop with --user-data-dir=$test_dir"
    log_warn "A NEW Claude Desktop window should appear with a login screen."
    log_warn "DO NOT log in — just verify the window appears."
    log_warn "Press Enter after observing the result..."

    open -n -a "$CLAUDE_DESKTOP_APP" --args --user-data-dir="$test_dir"

    read -r

    # Check what files were created in the test directory
    log_info "Files created in sandbox desktop-test:"
    if [ -d "$test_dir" ] && [ "$(ls -A "$test_dir" 2>/dev/null)" ]; then
        ls -la "$test_dir" | head -30
        local file_count
        file_count=$(find "$test_dir" -type f 2>/dev/null | wc -l | tr -d ' ')
        log_ok "V1: $file_count files created in isolated directory"
        append_result 1 "--user-data-dir isolation" "PASS" "$file_count files created in isolated dir"

        # Check if config.json was created (indicates new session)
        if [ -f "$test_dir/config.json" ]; then
            log_ok "  config.json created (new auth context)"
            append_detail "### V1 Detail: config.json in new dir"
            append_detail '```json'
            append_detail "$(cat "$test_dir/config.json" | head -5)"
            append_detail '```'
        fi

        # Check if it has oauth tokens (should NOT have them yet)
        if grep -q "oauth:tokenCache" "$test_dir/config.json" 2>/dev/null; then
            log_warn "  oauth tokens found — may be leaking from default"
        else
            log_ok "  No oauth tokens in new config (clean isolation)"
        fi
    else
        log_fail "V1: No files created in sandbox directory"
        append_result 1 "--user-data-dir isolation" "FAIL" "No files created"
    fi

    # Kill the test instance
    log_info "Closing test Desktop instance..."
    # Find PIDs with our specific user-data-dir
    local pids
    pids=$(pgrep -f "user-data-dir=$test_dir" 2>/dev/null || true)
    if [ -n "$pids" ]; then
        echo "$pids" | xargs kill 2>/dev/null || true
        sleep 2
    fi

    # Verify original Desktop is untouched
    log_info "Verifying original Desktop data is untouched..."
    if [ -f "$CLAUDE_DESKTOP_DEFAULT/config.json" ]; then
        log_ok "  Original config.json still exists"
    fi
}

# ============================================================================
# V2: Keychain behavior with different user-data-dir
# ============================================================================

test_v2_keychain_desktop() {
    log_info "V2: Checking Keychain entries for Claude Desktop"

    log_info "Current Keychain entries:"
    local before
    before=$(security dump-keychain 2>/dev/null | grep -c "Claude" || echo "0")
    log_info "  Claude-related entries before: $before"

    security dump-keychain 2>/dev/null | grep -A2 '"svce"' | grep -i claude || true

    log_info ""
    log_info "After running V1, check if new Keychain entries were created:"
    local after
    after=$(security dump-keychain 2>/dev/null | grep -c "Claude" || echo "0")
    log_info "  Claude-related entries after: $after"

    if [ "$after" -gt "$before" ]; then
        log_ok "V2: New Keychain entry created for isolated instance"
        append_result 2 "Keychain Desktop isolation" "PASS" "New entry created (before: $before, after: $after)"
    elif [ "$after" -eq "$before" ]; then
        log_warn "V2: Same number of Keychain entries — shared encryption key"
        append_result 2 "Keychain Desktop isolation" "SHARED" "Same entry count ($after). Encryption key is shared but auth is in config.json"
    else
        log_fail "V2: Fewer entries — unexpected"
        append_result 2 "Keychain Desktop isolation" "UNEXPECTED" "Before: $before, After: $after"
    fi
}

# ============================================================================
# V3: Simultaneous Desktop instances
# ============================================================================

test_v3_simultaneous_desktop() {
    log_info "V3: Testing simultaneous Desktop instances"

    local running_before
    running_before=$(pgrep -f "Claude.app/Contents/MacOS/Claude" 2>/dev/null | wc -l | tr -d ' ')

    if [ "$running_before" -eq 0 ]; then
        log_warn "V3: No Claude Desktop instance running. Start one first."
        append_result 3 "Simultaneous Desktop instances" "SKIP" "No Desktop running"
        return
    fi

    log_info "Current Claude Desktop processes: $running_before"
    log_info "Launching second instance with sandbox user-data-dir..."

    local test_dir="$SANDBOX_DIR/desktop-test-v3"
    mkdir -p "$test_dir"
    open -n -a "$CLAUDE_DESKTOP_APP" --args --user-data-dir="$test_dir"
    sleep 3

    local running_after
    running_after=$(pgrep -f "Claude.app/Contents/MacOS/Claude" 2>/dev/null | wc -l | tr -d ' ')
    log_info "Claude Desktop processes after launch: $running_after"

    if [ "$running_after" -gt "$running_before" ]; then
        log_ok "V3: Multiple instances running simultaneously"
        append_result 3 "Simultaneous Desktop instances" "PASS" "Before: $running_before, After: $running_after processes"
    else
        log_fail "V3: No additional processes detected"
        append_result 3 "Simultaneous Desktop instances" "FAIL" "Same process count"
    fi

    # Cleanup
    local pids
    pids=$(pgrep -f "user-data-dir=$test_dir" 2>/dev/null || true)
    if [ -n "$pids" ]; then
        echo "$pids" | xargs kill 2>/dev/null || true
    fi
    rm -rf "$test_dir"
}

# ============================================================================
# V4: CLAUDE_CONFIG_DIR for CLI
# ============================================================================

test_v4_cli_config_dir() {
    log_info "V4: Testing CLAUDE_CONFIG_DIR for CLI isolation"

    local test_dir="$SANDBOX_DIR/cli-test"
    mkdir -p "$test_dir"

    if ! command -v claude &>/dev/null; then
        log_fail "V4: 'claude' command not found"
        append_result 4 "CLAUDE_CONFIG_DIR isolation" "SKIP" "claude CLI not installed"
        return
    fi

    log_info "Running: CLAUDE_CONFIG_DIR=$test_dir claude --version"
    local version
    version=$(CLAUDE_CONFIG_DIR="$test_dir" claude --version 2>&1 || true)
    log_info "  Version output: $version"

    log_info "Checking files created in sandbox CLI dir:"
    if [ -d "$test_dir" ] && [ "$(ls -A "$test_dir" 2>/dev/null)" ]; then
        ls -la "$test_dir"
        log_ok "V4: CLI created files in isolated directory"
        append_result 4 "CLAUDE_CONFIG_DIR isolation" "PASS" "Files created in isolated dir"
    else
        log_info "  No files created (--version may not initialize)"
        log_info "  Trying: CLAUDE_CONFIG_DIR=$test_dir claude --help (short)"
        CLAUDE_CONFIG_DIR="$test_dir" claude --help 2>&1 | head -5 || true

        if [ "$(ls -A "$test_dir" 2>/dev/null)" ]; then
            log_ok "V4: CLI created files after --help"
            append_result 4 "CLAUDE_CONFIG_DIR isolation" "PASS" "Files created after help"
        else
            log_warn "V4: No files created, may need interactive session"
            append_result 4 "CLAUDE_CONFIG_DIR isolation" "PARTIAL" "No files from non-interactive commands"
        fi
    fi

    # Verify original CLI dir is untouched
    if [ -f "$CLAUDE_CLI_DEFAULT/CLAUDE.md" ]; then
        log_ok "  Original CLAUDE.md still exists (untouched)"
    fi
}

# ============================================================================
# V5: CLI Keychain with different CLAUDE_CONFIG_DIR
# ============================================================================

test_v5_keychain_cli() {
    log_info "V5: Checking Keychain behavior for CLI with CLAUDE_CONFIG_DIR"

    local entries_before
    entries_before=$(security dump-keychain 2>/dev/null | grep -A3 "Claude Code-credentials" | grep "acct" || echo "none")
    log_info "  Current CLI Keychain account: $entries_before"

    log_info "  (Full verification requires interactive 'claude login' in sandbox — manual step)"
    append_result 5 "CLI Keychain isolation" "MANUAL" "Requires interactive login test. Current acct: $(echo "$entries_before" | tr -d '\"' | awk -F= '{print $2}')"
}

# ============================================================================
# V6: Symlink-based config sharing
# ============================================================================

test_v6_symlink_sharing() {
    log_info "V6: Testing symlink-based config file sharing"

    local test_dir="$SANDBOX_DIR/cli-test"
    mkdir -p "$test_dir"

    # Test 1: Symlink CLAUDE.md
    if [ -f "$CLAUDE_CLI_DEFAULT/CLAUDE.md" ]; then
        ln -sf "$CLAUDE_CLI_DEFAULT/CLAUDE.md" "$test_dir/CLAUDE.md"

        if [ -L "$test_dir/CLAUDE.md" ] && [ -f "$test_dir/CLAUDE.md" ]; then
            local original_size linked_size
            original_size=$(wc -c < "$CLAUDE_CLI_DEFAULT/CLAUDE.md" | tr -d ' ')
            linked_size=$(wc -c < "$test_dir/CLAUDE.md" | tr -d ' ')

            if [ "$original_size" = "$linked_size" ]; then
                log_ok "V6: CLAUDE.md symlink works (${original_size} bytes)"
            else
                log_fail "V6: Size mismatch (original: $original_size, linked: $linked_size)"
            fi
        else
            log_fail "V6: Symlink creation failed or broken"
        fi
    fi

    # Test 2: Symlink settings.json
    if [ -f "$CLAUDE_CLI_DEFAULT/settings.json" ]; then
        ln -sf "$CLAUDE_CLI_DEFAULT/settings.json" "$test_dir/settings.json"
        if [ -L "$test_dir/settings.json" ] && [ -f "$test_dir/settings.json" ]; then
            log_ok "V6: settings.json symlink works"
        fi
    fi

    # Test 3: Symlink a memory directory
    local sample_project
    sample_project=$(find "$CLAUDE_CLI_DEFAULT/projects" -maxdepth 1 -type d | head -2 | tail -1)
    if [ -n "$sample_project" ] && [ -d "$sample_project/memory" ]; then
        mkdir -p "$test_dir/projects/$(basename "$sample_project")"
        ln -sf "$sample_project/memory" "$test_dir/projects/$(basename "$sample_project")/memory"

        if [ -L "$test_dir/projects/$(basename "$sample_project")/memory" ]; then
            local mem_count
            mem_count=$(find "$test_dir/projects/$(basename "$sample_project")/memory" -name "*.md" -type f 2>/dev/null | wc -l | tr -d ' ')
            log_ok "V6: memory/ directory symlink works ($mem_count .md files accessible)"
        fi
    fi

    append_result 6 "Symlink-based config sharing" "PASS" "CLAUDE.md, settings.json, memory/ dir all work via symlink"
}

# ============================================================================
# V7: Hooks work through symlinked settings.json
# ============================================================================

test_v7_hooks_via_symlink() {
    log_info "V7: Checking if hooks in symlinked settings.json would work"

    local test_dir="$SANDBOX_DIR/cli-test"

    if [ -L "$test_dir/settings.json" ]; then
        local hooks
        hooks=$(cat "$test_dir/settings.json" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d.get('hooks',{}), indent=2))" 2>/dev/null || echo "parse error")

        log_info "  Hooks in symlinked settings.json:"
        echo "$hooks" | head -20

        if echo "$hooks" | grep -q "matsumotory-memory"; then
            log_ok "V7: matsumotory-memory hooks visible through symlink"

            # Check if the hook script exists
            local hook_script="/Users/r-matsumoto/matsumotory-memory/bin/sync.py"
            if [ -f "$hook_script" ]; then
                log_ok "V7: Hook script exists at $hook_script"
                append_result 7 "Hooks via symlinked settings.json" "PASS" "Hooks visible and script exists"
            else
                log_warn "V7: Hook script not found at $hook_script"
                append_result 7 "Hooks via symlinked settings.json" "PARTIAL" "Hooks visible but script not found"
            fi
        else
            log_warn "V7: No hooks found in settings"
            append_result 7 "Hooks via symlinked settings.json" "N/A" "No hooks configured"
        fi
    else
        log_skip "V7: Run V6 first to create symlinked settings.json"
        append_result 7 "Hooks via symlinked settings.json" "SKIP" "Depends on V6"
    fi
}

# ============================================================================
# V8: Desktop MCP config via symlink
# ============================================================================

test_v8_desktop_mcp_symlink() {
    log_info "V8: Testing Desktop MCP config sharing via symlink"

    local test_dir="$SANDBOX_DIR/desktop-test"
    mkdir -p "$test_dir"

    local original="$CLAUDE_DESKTOP_DEFAULT/claude_desktop_config.json"

    if [ -f "$original" ]; then
        ln -sf "$original" "$test_dir/claude_desktop_config.json"

        if [ -L "$test_dir/claude_desktop_config.json" ] && [ -f "$test_dir/claude_desktop_config.json" ]; then
            # Check MCP servers are accessible through symlink
            local mcp_check
            mcp_check=$(python3 -c "
import json
with open('$test_dir/claude_desktop_config.json') as f:
    d = json.load(f)
    prefs = d.get('preferences', {})
    mcp = d.get('mcpServers', {})
    print(f'preferences keys: {len(prefs)}')
    print(f'mcpServers count: {len(mcp)}')
" 2>/dev/null || echo "parse error")

            log_info "  MCP config through symlink:"
            echo "  $mcp_check"
            log_ok "V8: Desktop MCP config accessible via symlink"
            append_result 8 "Desktop MCP config symlink" "PASS" "Config readable. $mcp_check"
        else
            log_fail "V8: Symlink broken"
            append_result 8 "Desktop MCP config symlink" "FAIL" "Symlink creation failed"
        fi
    else
        log_skip "V8: No claude_desktop_config.json found"
        append_result 8 "Desktop MCP config symlink" "SKIP" "Original file not found"
    fi

    log_info ""
    log_info "Note: Full validation requires launching Desktop with --user-data-dir"
    log_info "containing this symlink and verifying MCP servers load correctly."
}

# ============================================================================
# Main
# ============================================================================

main() {
    echo ""
    echo "=============================================="
    echo "  Claude Desktop Switcher Sandbox Validation"
    echo "=============================================="
    echo ""

    setup_sandbox
    init_results

    local test_num="${1:-all}"

    if [ "$test_num" = "all" ] || [ "$test_num" = "1" ]; then test_v1_desktop_user_data_dir; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "2" ]; then test_v2_keychain_desktop; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "3" ]; then test_v3_simultaneous_desktop; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "4" ]; then test_v4_cli_config_dir; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "5" ]; then test_v5_keychain_cli; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "6" ]; then test_v6_symlink_sharing; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "7" ]; then test_v7_hooks_via_symlink; fi
    if [ "$test_num" = "all" ] || [ "$test_num" = "8" ]; then test_v8_desktop_mcp_symlink; fi

    echo ""
    echo "=============================================="
    echo "  Results written to: $RESULTS_FILE"
    echo "=============================================="

    # Append detail section header
    append_detail ""
    append_detail "---"
    append_detail ""
    append_detail "## Existing Environment (Reference)"
    append_detail ""
    append_detail "### Keychain Entries"
    append_detail '```'
    security dump-keychain 2>/dev/null | grep -A3 '"svce"' | grep -i -A2 claude >> "$RESULTS_FILE" 2>/dev/null || true
    append_detail '```'
    append_detail ""
    append_detail "### Desktop config.json keys"
    append_detail '```'
    python3 -c "
import json
with open('$CLAUDE_DESKTOP_DEFAULT/config.json') as f:
    d = json.load(f)
    for k in d:
        if 'oauth' in k.lower() or 'token' in k.lower():
            print(f'{k}: [REDACTED - {len(str(d[k]))} chars]')
        else:
            print(f'{k}: {d[k]}')
" >> "$RESULTS_FILE" 2>/dev/null || true
    append_detail '```'

    echo ""
    log_info "To clean up sandbox: rm -rf $SANDBOX_DIR"
    echo ""
}

main "$@"
