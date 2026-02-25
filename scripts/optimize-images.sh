#!/bin/bash
# Image Optimization Script for WebP Conversion
# This script converts images to WebP format and optimizes them for web use
# Requires: cwebp (from libwebp), imagemagick (optional, for comprehensive optimization)

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SOURCE_DIR="${1:-.}"
OUTPUT_QUALITY="${2:-80}"
BACKUP_ORIGINALS="${3:-false}"

echo -e "${YELLOW}Image Optimization Script${NC}"
echo "=================================================="

# Check if cwebp is installed
if ! command -v cwebp &> /dev/null; then
    echo -e "${RED}Error: cwebp is not installed${NC}"
    echo "Install libwebp: brew install libwebp (macOS) or apt-get install webp (Linux)"
    exit 1
fi

# Process all image files
TOTAL_SAVED=0
CONVERTED_COUNT=0

for img in "$SOURCE_DIR"/*.{jpg,jpeg,png,JPG,JPEG,PNG}; do
    [ -e "$img" ] || continue
    
    filename=$(basename "$img")
    webp_file="${img%.*}.webp"
    
    echo "Converting: $filename"
    
    # Get original file size
    original_size=$(stat -f%z "$img" 2>/dev/null || stat -c%s "$img" 2>/dev/null)
    
    # Convert to WebP
    if cwebp -q "$OUTPUT_QUALITY" "$img" -o "$webp_file"; then
        # Get new file size
        webp_size=$(stat -f%z "$webp_file" 2>/dev/null || stat -c%s "$webp_file" 2>/dev/null)
        saved=$((original_size - webp_size))
        
        echo -e "${GREEN}✓ Converted: $filename -> $(basename "$webp_file") (Saved: $(numfmt --to=iec-i --suffix=B $saved 2>/dev/null || echo "${saved} bytes"))${NC}"
        
        TOTAL_SAVED=$((TOTAL_SAVED + saved))
        CONVERTED_COUNT=$((CONVERTED_COUNT + 1))
        
        # Optional: backup original files
        if [ "$BACKUP_ORIGINALS" = "true" ]; then
            mv "$img" "${img}.bak"
            echo "  Original backed up to: ${img}.bak"
        fi
    else
        echo -e "${RED}✗ Failed to convert: $filename${NC}"
    fi
done

echo ""
echo "=================================================="
echo -e "${GREEN}Optimization Complete!${NC}"
echo "Files Converted: $CONVERTED_COUNT"
echo "Total Space Saved: $(numfmt --to=iec-i --suffix=B $TOTAL_SAVED 2>/dev/null || echo "${TOTAL_SAVED} bytes")"

# Usage instructions
echo ""
echo "Usage in HTML/Templates:"
echo "<picture>"
echo "  <source srcset=\"/path/to/image.webp\" type=\"image/webp\">"
echo "  <img src=\"/path/to/image.jpg\" alt=\"Description\" loading=\"lazy\" width=\"800\" height=\"600\">"
echo "</picture>"
