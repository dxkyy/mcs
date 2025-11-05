#!/usr/bin/env bash
# Release script for mcs (Linux/Mac version)
# This script helps create a new release by tagging and pushing to GitHub

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./release.sh <version>"
    echo "Example: ./release.sh 0.1.0"
    exit 1
fi

# Validate version format (e.g., 0.1.0)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z (e.g., 0.1.0)"
    exit 1
fi

TAG="v$VERSION"

echo "Creating release $TAG..."

# Check if tag already exists
if git tag -l | grep -q "^$TAG$"; then
    echo "Error: Tag $TAG already exists!"
    exit 1
fi

# Check if there are uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Warning: You have uncommitted changes:"
    git status --porcelain
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Release cancelled."
        exit 0
    fi
fi

# Update Cargo.toml version
echo "Updating Cargo.toml version to $VERSION..."
sed -i.bak "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Commit version bump
echo "Committing version bump..."
git add Cargo.toml
git commit -m "Bump version to $VERSION"

# Create and push tag
echo "Creating tag $TAG..."
git tag -a $TAG -m "Release $TAG"

echo "Pushing to GitHub..."
git push origin master
git push origin $TAG

echo ""
echo "✓ Release $TAG created successfully!"
echo ""
echo "GitHub Actions will now build binaries for:"
echo "  - Windows (x86_64)"
echo "  - Linux (x86_64)"
echo "  - macOS (x86_64 and ARM64)"
echo ""
echo "Visit https://github.com/dxkyy/mcs/releases to see the release."
