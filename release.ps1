#!/usr/bin/env pwsh
# Release script for mcs
# This script helps create a new release by tagging and pushing to GitHub

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

# Validate version format (e.g., 0.1.0)
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Version must be in format X.Y.Z (e.g., 0.1.0)"
    exit 1
}

$Tag = "v$Version"

Write-Host "Creating release $Tag..." -ForegroundColor Green

# Check if tag already exists
$existingTag = git tag -l $Tag
if ($existingTag) {
    Write-Error "Tag $Tag already exists!"
    exit 1
}

# Check if there are uncommitted changes
$status = git status --porcelain
if ($status) {
    Write-Warning "You have uncommitted changes:"
    Write-Host $status
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne 'y') {
        Write-Host "Release cancelled."
        exit 0
    }
}

# Update Cargo.toml version
Write-Host "Updating Cargo.toml version to $Version..." -ForegroundColor Cyan
$cargoContent = Get-Content "Cargo.toml" -Raw
$cargoContent = $cargoContent -replace 'version = "\d+\.\d+\.\d+"', "version = `"$Version`""
Set-Content "Cargo.toml" -Value $cargoContent -NoNewline

# Commit version bump
Write-Host "Committing version bump..." -ForegroundColor Cyan
git add Cargo.toml
git commit -m "Bump version to $Version"

# Create and push tag
Write-Host "Creating tag $Tag..." -ForegroundColor Cyan
git tag -a $Tag -m "Release $Tag"

Write-Host "Pushing to GitHub..." -ForegroundColor Cyan
git push origin master
git push origin $Tag

Write-Host ""
Write-Host "✓ Release $Tag created successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "GitHub Actions will now build binaries for:" -ForegroundColor Yellow
Write-Host "  - Windows (x86_64)"
Write-Host "  - Linux (x86_64)"
Write-Host "  - macOS (x86_64 and ARM64)"
Write-Host ""
Write-Host "Visit https://github.com/dxkyy/mcs/releases to see the release." -ForegroundColor Cyan
