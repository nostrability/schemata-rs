# Download latest schemata release and vendor schemas
update-schemas:
    #!/usr/bin/env bash
    set -euo pipefail

    # Resolve latest release tag
    TAG=$(curl -sI https://github.com/nostrability/schemata/releases/latest \
        | grep -i '^location:' | sed 's|.*/||' | tr -d '\r\n')
    echo "Latest release: $TAG"

    TMPDIR=$(mktemp -d)
    ZIP="$TMPDIR/schemata.zip"

    # Download release zip
    curl -sL "https://github.com/nostrability/schemata/releases/download/${TAG}/schemata-${TAG}.zip" -o "$ZIP"
    echo "Downloaded schemata-${TAG}.zip"

    # Extract
    unzip -qo "$ZIP" -d "$TMPDIR/extracted"

    # Clean previous schemas
    rm -rf schemas
    mkdir -p schemas

    # Copy nips/ and mips/ as-is
    cp -r "$TMPDIR/extracted/nips" schemas/nips
    cp -r "$TMPDIR/extracted/mips" schemas/mips

    # Copy @/ → schemas/_aliases/ (rename for filesystem compatibility)
    cp -r "$TMPDIR/extracted/@" schemas/_aliases

    # Write version file
    echo "$TAG" > schemas/SCHEMATA_VERSION

    # Cleanup
    rm -rf "$TMPDIR"

    echo "Vendored schemas from $TAG"
    echo "  nips:    $(find schemas/nips -name '*.json' | wc -l | tr -d ' ') files"
    echo "  mips:    $(find schemas/mips -name '*.json' | wc -l | tr -d ' ') files"
    echo "  aliases: $(find schemas/_aliases -name '*.json' | wc -l | tr -d ' ') files"

# Build the crate
build:
    cargo build

# Run tests
test:
    cargo test

# Run clippy
lint:
    cargo clippy -- -D warnings
