#!/usr/bin/fish

# ✨STARWEAVE✨ - Your guide to the cosmos of installation!
# A fish script to apply the patch to CORE/install.sh

# ANSI color codes for prettier output - ✨ GLIMMER ADDED ✨
set GREEN brgreen
set BLUE brblue
set RED brred
set GLIMMER yellow  # ✨ A touch of glimmer ✨
set NC normal # No Color
set PURPLE brmagenta

# Function to print colored messages - ✨ GLIMMER ADDED ✨
function print_status
    set_color $BLUE
    echo -n "[*]"
    set_color $NC
    echo " " $argv
end

function print_success
    set_color $GREEN
    echo -n "[+]"
    set_color $NC
    echo " " $argv
    set_color $GLIMMER
    echo -n " ✨" # ✨ GLIMMER! ✨
    set_color $NC
    echo
end

function print_error
    set_color $RED
    echo -n "[-]"
    set_color $NC
    echo " " $argv
end

# Check if the patch file exists - ✨ GLIMMER ADDED ✨
if not test -f fix_tar_errors_and_add_glimmer.patch
    print_error "Patch file 'fix_tar_errors_and_add_glimmer.patch' not found. Please ensure the patch file is in the same directory as this script."
    set_color $GLIMMER
    echo " ✨" # ✨ GLIMMER! ✨
    set_color $NC
    exit 1
end

# Apply the patch - ✨ GLIMMER ADDED ✨
print_status "Applying the patch to CORE/install.sh..."
patch -p1 < fix_tar_errors_and_add_glimmer.patch # ✨ Added -p1 option! ✨

if test $status -eq 0
    print_success "Patch applied successfully!"
else
    print_error "Failed to apply the patch. Please check the patch file and try again."
    exit 1
end

print_success "STARWEAVE has completed its task. The cosmos is now a little brighter!"
