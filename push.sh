#!/bin/bash

# Get the current date and time
current_date_time=$(date "+%Y-%m-%d %H:%M:%S %Z")
service_name="authentication_service"

# Get the commit message from user input
read -p "Enter commit message: " commit_message

# If the commit message is empty, set it to "Blind Commit"
if [ -z "$commit_message" ]; then
    commit_message="Blind Commit"
fi

# Add changes to the staging area
git add .

# Get the number of changes
num_changes=$(git diff --cached --numstat | wc -l)

echo "Total Number of Changes: $num_changes"

# Determine the commit mark based on the number of changes
if [ "$num_changes" -le 30 ]; then
    mark="lite"
elif [ "$num_changes" -le 100 ]; then
    mark="small"
else
   mark="large"
fi

# Declaring change type
echo "The changes are categorized as $mark"

# Parse the current version from version.rs
current_version=$(grep '^pub const VERSION:' src/version.rs | sed -E 's/.*"([^"]+)".*/\1/')

# IFS='.' read -r -a version_parts <<< "$current_version"

# Split the version into parts
version_parts=(${current_version//./ })

# removing the date/time from last part of the version
version_parts[2]=$(echo "${version_parts[2]}" | cut -d':' -f1)

# Function to strip leading zeros
strip_leading_zeros() {
    if [ "$1" -eq 0 ]; then
        echo "0"
    else
        echo "$1" | sed 's/^0*//'
    fi
}

# Strip leading zeros from version parts
version_parts[0]=$(strip_leading_zeros "${version_parts[0]}")
version_parts[1]=$(strip_leading_zeros "${version_parts[1]}")
version_parts[2]=$(strip_leading_zeros "${version_parts[2]}")

# Increment version parts based on the mark
if [ "$mark" == "lite" ]; then
    version_parts[2]=$((version_parts[2] + 1))
elif [ "$mark" == "small" ]; then
    version_parts[1]=$((version_parts[1] + 1))
elif [ "$mark" == "large" ]; then
    version_parts[0]=$((version_parts[0] + 1))
else
    version_parts[0]=$((version_parts[2] + 1))
fi

formatted_date_time=$(date "+%y%m%d%H%M")

# Construct the new version string
new_version="${version_parts[0]}.${version_parts[1]}.${version_parts[2]}:${formatted_date_time}"

# Update version.rs with the new version
# sed -i.bak "s|^pub const VERSION: &str = \".*\";|pub const VERSION: &str = \"$new_version\";|" src/version.rs && rm src/version.rs.bak
awk -v v="$new_version" '
  /^pub const VERSION:/ {
    print "pub const VERSION: &str = \"" v "\";"
    next
  }
  { print }
' src/version.rs > src/version.rs.tmp && mv src/version.rs.tmp src/version.rs


# version update message
echo "Version updated from $current_version to $new_version"

# removing white spaces from num changes
num_changes=$(echo $num_changes | tr -d ' ')

# Format the commit message
formatted_commit_message="${current_date_time}-(${mark})-(${num_changes})-(V:${new_version})-> \"${commit_message}\""

# To add new version to package.json
git add .

# Commit the changes
git commit -m "$formatted_commit_message"

git push

# Summary message
echo
echo "---------Summary---------"
echo
echo "Date: $current_date_time"
echo "Service: $service_name"
echo "  1. Total Number of changes: $num_changes"
echo "  2. Change Category: $mark"
echo "  3. Version updated from $current_version to $new_version"
echo "  4. Pushed with Commit message: ${formatted_commit_message}"