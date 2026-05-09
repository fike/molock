#!/bin/bash
# find_untested_functions.sh
# Script to identify functions without tests in Rust source files

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <rust_source_file>"
    echo "Example: $0 src/telemetry/metrics.rs"
    exit 1
fi

FILE="$1"

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found"
    exit 1
fi

echo "=============================================="
echo "Analyzing: $FILE"
echo "=============================================="
echo ""

# Count total functions
TOTAL_FUNCTIONS=$(grep -c "^\s*pub fn\|^\s*fn [a-z_]" "$FILE" | grep -v "test_" | grep -v "#\[test\]" | grep -v "#\[cfg(test)\]")
echo "Total functions found: $TOTAL_FUNCTIONS"

# Find functions without 'test_' prefix (excluding test functions)
echo ""
echo "Functions without 'test_' prefix (potential untested functions):"
echo "----------------------------------------------------------------"

LINE_NUMBER=0
while IFS= read -r line; do
    LINE_NUMBER=$((LINE_NUMBER + 1))
    
    # Skip test functions and test module declarations
    if echo "$line" | grep -q "test_\|#\[test\]\|#\[cfg(test)\]"; then
        continue
    fi
    
    # Look for function definitions (pub fn or fn with lowercase starting name)
    if echo "$line" | grep -q "^\s*pub fn\|^\s*fn [a-z_]"; then
        # Extract function name
        FUNC_NAME=$(echo "$line" | sed -n 's/.*fn \([a-zA-Z_][a-zA-Z0-9_]*\).*/\1/p')
        
        # Check if there's a test for this function
        if grep -q "test_$FUNC_NAME\|test_.*$FUNC_NAME" "$FILE"; then
            echo "✓ $LINE_NUMBER: $FUNC_NAME (has test)"
        else
            echo "✗ $LINE_NUMBER: $FUNC_NAME (NO TEST FOUND)"
        fi
    fi
done < "$FILE"

echo ""
echo "=============================================="
echo "Coverage Analysis Recommendations:"
echo "=============================================="
echo ""
echo "1. Run coverage check:"
echo "   cargo tarpaulin --lib --src $FILE"
echo ""
echo "2. Add tests for functions marked 'NO TEST FOUND'"
echo ""
echo "3. Run all tests:"
echo "   cargo test --lib"
echo ""
echo "4. Check overall coverage:"
echo "   make test-coverage"
echo ""
echo "Note: Minimum required coverage is 80%"
echo "=============================================="