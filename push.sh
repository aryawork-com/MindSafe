#!/bin/bash

APP_NAME="MindSafe"
CARGO_TOML="Cargo.toml"

echo ""
echo "---------Initiating---------"
echo ""

# --- DateTime ---
CURRENT_DATETIME=$(date "+%Y-%m-%d %H:%M:%S %Z")
TIMESTAMP=$(date "+%y%m%d%H%M")

# --- Commit Message ---
read -rp "Enter commit message: " COMMIT_MESSAGE
COMMIT_MESSAGE="${COMMIT_MESSAGE:-Blind Commit}"

# --- Stage Changes ---
git add .

# --- Count Staged Changes ---
NUM_CHANGES=$(git diff --cached --numstat | wc -l | tr -d ' ')

# --- Categorize Changes ---
if [ "$NUM_CHANGES" -le 30 ]; then
    MARK="lite"
elif [ "$NUM_CHANGES" -le 100 ]; then
    MARK="small"
else
    MARK="large"
fi

# --- Read Current Version from Cargo.toml ---
CURRENT_VERSION=$(grep -m 1 '^version' "$CARGO_TOML" | sed 's/version *= *"\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo "ERROR: Could not find version in $CARGO_TOML"
    exit 1
fi

# --- Ask User for New Version ---
echo ""
echo "Current version: $CURRENT_VERSION"
read -rp "Enter new version (or press Enter to auto-increment patch): " NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    # Auto-increment the patch (last) segment
    MAJOR=$(echo "$CURRENT_VERSION" | cut -d'.' -f1)
    MINOR=$(echo "$CURRENT_VERSION" | cut -d'.' -f2)
    PATCH=$(echo "$CURRENT_VERSION" | cut -d'.' -f3)
    PATCH=$((PATCH + 1))
    NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
fi

# --- Update version in Cargo.toml (first occurrence only) ---
# Uses a temp file for compatibility across Linux/macOS
TEMP_FILE=$(mktemp)
awk -v new_ver="$NEW_VERSION" '
    !done && /^version *= *"[^"]*"/ {
        sub(/"[^"]*"/, "\"" new_ver "\"")
        done=1
    }
    { print }
' "$CARGO_TOML" > "$TEMP_FILE" && mv "$TEMP_FILE" "$CARGO_TOML"

# --- Format Commit Message ---
MARK_CAP="$(tr '[:lower:]' '[:upper:]' <<< "${MARK:0:1}")${MARK:1}"
COMMIT_MSG_CAP="$(tr '[:lower:]' '[:upper:]' <<< "${COMMIT_MESSAGE:0:1}")${COMMIT_MESSAGE:1}"
FORMATTED_MESSAGE="${CURRENT_DATETIME}-(${MARK})-(${NUM_CHANGES})-(V:${NEW_VERSION})-> \"${COMMIT_MESSAGE}\""

# --- Commit and Push ---
git add .
git commit -m "$FORMATTED_MESSAGE"
git push

# --- Summary Table ---
echo ""
echo "---------Summary---------"
echo ""
printf "%-20s %s\n" "Parameter" "Value"
printf "%-20s %s\n" "---------" "-----"
printf "%-20s %s\n" "Date"               "$CURRENT_DATETIME"
printf "%-20s %s\n" "App Name"           "$APP_NAME"
printf "%-20s %s\n" "Number of Changes"  "$NUM_CHANGES"
printf "%-20s %s\n" "Changes Category"   "$MARK_CAP"
printf "%-20s %s\n" "Old Version"        "$CURRENT_VERSION"
printf "%-20s %s\n" "New Version"        "$NEW_VERSION"
printf "%-20s %s\n" "Commit Message"     "$COMMIT_MSG_CAP"
echo ""
echo "---------END---------"
echo ""