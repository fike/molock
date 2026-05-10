#!/bin/bash
# add_license_headers.sh
# Script to add Apache 2.0 license headers to Rust source files
# Following CNCF/Apache Foundation best practices

set -e

LICENSE_HEADER="/*
 * Copyright 2026 Molock Team
 *
 * Licensed under the Apache License, Version 2.0 (the \"License\");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an \"AS IS\" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */"

echo "Adding Apache 2.0 license headers to Rust source files..."
echo "=========================================================="

# Process each Rust source file
find src -name "*.rs" -type f | while read -r file; do
    echo "Processing: $file"
    
    # Check if file already has a license header
    if head -n 1 "$file" | grep -q "Copyright\|Licensed under"; then
        echo "  ✓ Already has license header"
        continue
    fi
    
    # Create temporary file with license header
    TEMP_FILE=$(mktemp)
    
    # Add license header
    echo "$LICENSE_HEADER" > "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    
    # Add original content
    cat "$file" >> "$TEMP_FILE"
    
    # Replace original file
    mv "$TEMP_FILE" "$file"
    
    echo "  ✓ Added license header"
done

echo ""
echo "Processing Cargo.toml..."
echo "========================="

# Also add license to Cargo.toml if not present
if ! grep -q "license" Cargo.toml; then
    # Find the [package] section and add license after it
    sed -i '/\[package\]/a license = "Apache-2.0"' Cargo.toml
    echo "✓ Added license to Cargo.toml"
else
    echo "✓ Cargo.toml already has license field"
fi

echo ""
echo "Processing README.md..."
echo "========================"

# Update README.md to mention Apache 2.0 license
if ! grep -q "Apache 2.0" README.md; then
    # Add license mention at the end of README
    echo "" >> README.md
    echo "## License" >> README.md
    echo "" >> README.md
    echo "Molock is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text." >> README.md
    echo "✓ Added license section to README.md"
else
    echo "✓ README.md already mentions Apache 2.0 license"
fi

echo ""
echo "=========================================================="
echo "License headers added successfully!"
echo ""
echo "Summary:"
echo "- Added Apache 2.0 headers to all Rust source files in src/"
echo "- Added license field to Cargo.toml"
echo "- Added license section to README.md"
echo ""
echo "Best practices followed:"
echo "1. Copyright notice with current year"
echo "2. Full Apache 2.0 boilerplate text"
echo "3. License field in Cargo.toml for cargo metadata"
echo "4. License mention in README.md for visibility"
echo "=========================================================="