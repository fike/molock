#!/bin/bash
# fetch_latest_logs.sh
# Procedural tool to assist in the 'Observe' phase of the OODA loop.
# It attempts to gather recent error and warning signals from the environment.

echo "--- OODA: Observe - Gathering Signals ---"

# 1. Check for log files in the root or common locations
LOG_FILES=$(find . -maxdepth 2 -name "*.log")

if [ -n "$LOG_FILES" ]; then
    for f in $LOG_FILES; do
        echo "Signals from $f:"
        tail -n 50 "$f" | grep -E "ERROR|WARN|panic|fail"
    done
else
    echo "No .log files detected in the immediate workspace."
fi

# 2. Check for recent test failures in target (if any)
if [ -d "target" ]; then
    echo "Checking target/ for recent test artifacts..."
    find target -name "*.out" -mmin -10 2>/dev/null | xargs grep -l "FAILED" 2>/dev/null
fi

echo "--- End of Signals ---"
