#!/bin/bash
# Local build + GitHub Release upload script
# Usage: ./scripts/release.sh v0.1.0

set -e

VERSION="${1:?Usage: $0 <version-tag> (e.g. v0.1.0)}"

echo "=== Building SSH Forwarder ${VERSION} ==="

# 1. Build release
echo "[1/4] Building Tauri release..."
npm run tauri build

# 2. Find built artifacts
MSI=$(find src-tauri/target/release/bundle/msi -name "*.msi" 2>/dev/null | head -1)
NSIS=$(find src-tauri/target/release/bundle/nsis -name "*.exe" 2>/dev/null | head -1)

echo "[2/4] Built artifacts:"
[ -n "$MSI" ] && echo "  MSI:  $MSI" || echo "  MSI:  (not found)"
[ -n "$NSIS" ] && echo "  NSIS: $NSIS" || echo "  NSIS: (not found)"

# 3. Tag and push
echo "[3/4] Tagging ${VERSION}..."
git tag "${VERSION}" 2>/dev/null || echo "  Tag already exists"
git push origin "${VERSION}"

# Wait for GitHub to process the tag
sleep 3

# 4. Upload to release
echo "[4/4] Uploading to GitHub Release..."
if [ -n "$MSI" ]; then
  gh release upload "${VERSION}" "$MSI" --clobber 2>/dev/null || \
  gh release create "${VERSION}" "$MSI" --title "SSH Forwarder ${VERSION}" --draft
fi
if [ -n "$NSIS" ]; then
  gh release upload "${VERSION}" "$NSIS" --clobber 2>/dev/null || true
fi

echo ""
echo "=== Done! ==="
echo "Release: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/${VERSION}"
