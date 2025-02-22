#!/bin/bash

# ANSI color codes for prettier output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color
PURPLE='\033[0;35m'

# Function to print colored messages
print_status() {
    echo -e "${BLUE}[*]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[+]${NC} $1"
}

print_error() {
    echo -e "${RED}[-]${NC} $1"
}

print_header() {
    echo -e "\n${PURPLE}=== $1 ===${NC}\n"
}

# Function to check if running with sudo/root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        print_error "Please run as root"
        exit 1
    fi
}

# Function to check for required tools
check_dependencies() {
    local deps=("wget" "lsblk" "dd" "sync")
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            print_error "$dep is required but not installed."
            exit 1
        fi
    done
}

# Function to download latest Arch Linux ARM image
download_arch() {
    local arch_url="http://os.archlinuxarm.org/os/ArchLinuxARM-am33x-latest.tar.gz"
    local download_path="/tmp/arch-arm.tar.gz"
    
    if [ -f "$download_path" ]; then
        print_status "Existing Arch Linux ARM download found. Would you like to download fresh? [y/N]: "
        read -r redownload
        if [[ $redownload =~ ^[Yy]$ ]]; then
            rm "$download_path"
        else
            return
        fi
    fi
    
    print_status "Downloading latest Arch Linux ARM..."
    wget "$arch_url" -O "$download_path" || {
        print_error "Failed to download Arch Linux ARM"
        exit 1
    }
    print_success "Download complete"
}

# Function to list and select available drives
select_drive() {
    print_header "Available Drives"
    lsblk -d -o NAME,SIZE,TYPE,MODEL
    
    while true; do
        echo
        print_status "Enter the device name to install to (e.g., sdb, mmcblk0): "
        read -r selected_drive
        
        if [[ -b "/dev/$selected_drive" ]]; then
            print_status "You selected /dev/$selected_drive"
            echo -e "${RED}WARNING: This will ERASE ALL DATA on /dev/$selected_drive${NC}"
            print_status "Are you sure you want to continue? [y/N]: "
            read -r confirm
            
            if [[ $confirm =~ ^[Yy]$ ]]; then
                break
            fi
        else
            print_error "Invalid device. Please try again."
        fi
    done
    
    SELECTED_DRIVE="/dev/$selected_drive"
}

# Function to prepare the drive
prepare_drive() {
    local drive=$1
    
    print_header "Preparing Drive"
    
    # Create partition table and partitions
    print_status "Creating partitions..."
    
    # Create new partition table
    parted "$drive" mklabel msdos
    
    # Create boot partition (100MB)
    parted "$drive" mkpart primary fat32 1MiB 100MiB
    
    # Create root partition (rest of the space)
    parted "$drive" mkpart primary ext4 100MiB 100%
    
    # Format partitions
    mkfs.vfat "${drive}1"
    mkfs.ext4 "${drive}2"
    
    print_success "Drive prepared successfully"
}

# Function to mount partitions
mount_partitions() {
    local drive=$1
    
    print_header "Mounting Partitions"
    
    # Create mount points
    mkdir -p /mnt/arch
    mkdir -p /mnt/arch/boot
    
    # Mount partitions
    mount "${drive}2" /mnt/arch
    mount "${drive}1" /mnt/arch/boot
    
    print_success "Partitions mounted"
}

# Function to extract Arch Linux
extract_arch() {
    print_header "Extracting Arch Linux"
    
    cd /mnt/arch || exit 1
    tar xf /tmp/arch-arm.tar.gz
    
    print_success "Extraction complete"
}

# Function to prepare for eMMC installation
setup_emmc_install() {
    print_header "Setting up eMMC Installation"
    
    # Create script to run on first boot for eMMC installation
    cat > /mnt/arch/root/emmc_install.sh << 'EOF'
#!/bin/bash

# This script will run on first boot to install to eMMC
if [ -b /dev/mmcblk1 ]; then
    echo "Installing to eMMC..."
    dd if=/dev/mmcblk0 of=/dev/mmcblk1 bs=1M status=progress
    sync
    echo "eMMC installation complete! You can now remove the SD card and reboot."
else
    echo "eMMC device not found!"
fi
EOF
    
    chmod +x /mnt/arch/root/emmc_install.sh
    
    print_success "eMMC installation script prepared"
}

# Main installation function
main() {
    print_header "BeagleBone Black Arch Linux Installer"
    
    check_root
    check_dependencies
    download_arch
    select_drive
    prepare_drive "$SELECTED_DRIVE"
    mount_partitions "$SELECTED_DRIVE"
    extract_arch
    setup_emmc_install
    
    print_header "Installation Complete"
    print_success "Arch Linux has been installed to $SELECTED_DRIVE"
    print_status "To complete eMMC installation:"
    echo "1. Boot from the SD card"
    echo "2. Log in as root"
    echo "3. Run /root/emmc_install.sh"
    
    # Cleanup
    umount -R /mnt/arch
    sync
    
    print_status "You can now safely remove the drive and boot your BeagleBone Black"
}

# Run the installer
main
