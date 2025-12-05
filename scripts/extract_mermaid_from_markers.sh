#!/bin/bash
# Extract Mermaid diagrams from README.md using HTML comment markers
# Converts them to SVG files using mermaid-cli

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$REPO_ROOT"

echo -e "${BLUE}🔍 Extracting Mermaid diagrams from README.md...${NC}"
echo "=================================================="

# Create assets directory
mkdir -p assets/images

# Check if mermaid-cli is installed
if ! command -v mmdc &> /dev/null; then
    echo -e "${YELLOW}⚠️  mermaid-cli not found. Only extracting .mmd files.${NC}"
    echo -e "${YELLOW}   To generate SVGs, install with: npm install -g @mermaid-js/mermaid-cli${NC}"
    HAS_MMDC=false
else
    echo -e "${GREEN}✅ mermaid-cli found - will generate SVG files${NC}"
    HAS_MMDC=true
fi

# Create a temporary Python script for extraction
cat > /tmp/extract_mermaid.py << 'EOF'
import re
import sys
import os
import subprocess

diagram_count = 0
with open('README.md', 'r') as f:
    content = f.read()

# Find all mermaid blocks with their start markers containing filenames
start_pattern = r'<!-- MERMAID_START:?\s*([^>\s]+\.mmd)?\s*-->(.*?)<!-- MERMAID_END -->'
matches = re.findall(start_pattern, content, re.DOTALL)

for filename, match in matches:
    diagram_count += 1
    print(f'\n📍 Found Mermaid diagram #{diagram_count}')

    # Use the filename from the start marker if available
    if filename:
        filename = filename.strip()
        print(f'   📝 Found filename in marker: {filename}')
    else:
        # Generate filename based on content
        if 'gitGraph' in match:
            filename = f'git-diagram-{diagram_count}.mmd'
        else:
            filename = f'diagram-{diagram_count}.mmd'

    # Clean the content - remove mermaid code block markers
    cleaned = re.sub(r'^```mermaid\s*\n', '', match.strip())
    cleaned = re.sub(r'\n```\s*$', '', cleaned)

    # Write to file
    with open(f'assets/images/{filename}', 'w') as out:
        out.write(cleaned)

    print(f'✅ Extracted: assets/images/{filename}')

    # Convert to SVG and PNG if mermaid-cli is available
    if os.environ.get('HAS_MMDC') == 'true':
        # Generate SVG
        svg_filename = filename.replace('.mmd', '.svg')
        try:
            subprocess.run(['mmdc', '-i', f'assets/images/{filename}',
                          '-o', f'assets/images/{svg_filename}'],
                         capture_output=True, check=True)
            print(f'🎨 Generated SVG: assets/images/{svg_filename}')
        except:
            print('⚠️  Warning generating SVG')

        # Generate PNG with higher resolution for sharpness
        png_filename = filename.replace('.mmd', '.png')
        try:
            subprocess.run(['mmdc', '-i', f'assets/images/{filename}',
                          '-o', f'assets/images/{png_filename}',
                          '--scale', '2'],  # 2x scale for sharper images
                         capture_output=True, check=True)
            print(f'🖼️  Generated PNG: assets/images/{png_filename} (2x scale)')
        except:
            print('⚠️  Warning generating PNG')

if diagram_count > 0:
    print(f'\n📊 Summary:')
    print(f'   Extracted {diagram_count} diagram(s) to assets/images/')
    if os.environ.get('HAS_MMDC') == 'true':
        print(f'   Generated {diagram_count} SVG file(s)')
        print(f'   Generated {diagram_count} PNG file(s)')
    else:
        print('   SVG/PNG generation skipped (mermaid-cli not installed)')
else:
    print('\nℹ️  No Mermaid diagrams found with <!-- MERMAID_START --> markers')
EOF

# Export the mermaid-cli availability
export HAS_MMDC

# Run the Python script
python3 /tmp/extract_mermaid.py

# Clean up temporary file
rm -f /tmp/extract_mermaid.py

echo
echo "=================================================="
echo -e "${GREEN}✅ Done! Check assets/images/ for generated files.${NC}"
