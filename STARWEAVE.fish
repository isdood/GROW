#!/usr/bin/fish

# ✨STARWEAVE✨ - Your guide to the cosmos of installation!
# A fish script to apply the patch to CORE/install.sh

# ANSI color codes for prettier output - ✨ GLIMMER ADDED ✨
set GREEN '\033[0;32m'
set BLUE '\033[0;34m'
set RED '\033[0;31m'
set GLIMMER '\033[1;33m'  # ✨ A touch of glimmer ✨
set NC '\033[0m' # No Color
set PURPLE '\033[0;35m'

# Function to print colored messages - ✨ GLIMMER ADDED ✨
function print_status
    echo -e "$BLUE[*]$NC $argv"
end

function print_success
    echo -e "$GREEN[+]$NC $argv $GLIMMER✨$NC"
end

function print_error
    echo -e "$RED[-]$NC $argv"
end

# Check if the patch file exists - ✨ GLIMMER ADDED ✨
if not test -f fix_tar_errors_and_add_glimmer.patch
    print_error "Patch file 'fix_tar_errors_and_add_glimmer.patch' not found.  Please ensure the patch file is in the same directory as this script.$GLIMMER✨$NC"
    exit 1
end

# Apply the patch - ✨ GLIMMER ADDED ✨
print_status "Applying the patch to CORE/install.sh...$GLIMMER✨$NC"
patch < fix_tar_errors_and_add_glimmer.patch

if test $status -eq 0
    print_success "Patch applied successfully!$GLIMMER✨$NC"
else
    print_error "Failed to apply the patch.  Please check the patch file and try again.$GLIMMER✨$NC"
    exit 1
end

print_success "STARWEAVE has completed its task.  The cosmos is now a little brighter!$GLIMMER✨$NC"
