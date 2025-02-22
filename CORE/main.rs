use std::process::Command;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    loop {
        clear_screen();
        println!("  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!(" ~ GROW ~ ... If you're ready to Bloom");
        println!("  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");

        let drives_info = get_drives_info();
        print_drives_info(&drives_info);

        let device = select_device(&drives_info);
        println!("\n   DEVICE set to: {}", device);

        // Unmount all partitions of the specified drive
        unmount_all_partitions(&device);

        // Get the original username of the user who invoked sudo
        let username = env::var("SUDO_USER").unwrap_or_else(|_| whoami::username());

        if whoami::username() == "root" {
            let tarball_path = format!("/home/{}/ArchLinuxARM-am33x-latest.tar.gz", username);
            download_file_if_not_exists(&tarball_path);

            // Check if the tarball file exists and is readable
            ensure_file_exists(&tarball_path);

            let mnt_path = format!("/home/{}/mnt", username);
            prepare_mount_path(&mnt_path);

            print!("\n   Deleting partitions   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                zero_out_device(&device);
                delete_partitions(&device);
            });

            print!("\n   Creating new partition   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                create_new_partition_table(&device);
            });

            print!("\n   Creating ext4 filesystem   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                create_ext4_filesystem(&(device.to_string() + "1"));
                mount_device(&(device.to_string() + "1"), &mnt_path);
                ensure_directory_exists(&format!("{}/boot", mnt_path));
            });

            let mlo_path = format!("{}/boot/MLO", mnt_path);
            let uboot_img_path = format!("{}/boot/u-boot.img", mnt_path);

            print!("\n   Extracting tarball to microSD   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                extract_tarball(&tarball_path, &mnt_path);

                // Confirm the required files are extracted
                ensure_file_exists(&mlo_path);
                ensure_file_exists(&uboot_img_path);
            });

            print!("\n   Syncing   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                sync();
            });

            write_mlo_to_device(&device, &mlo_path);
            write_uboot_to_device(&device, &uboot_img_path);

            print!("\n   Copying files to microSD   ");
            io::stdout().flush().unwrap();
            show_spinner_until_next_task(|| {
                copy_files_to_mnt(&username, &mnt_path);
                sync();
            });

            println!("\n\n         ~~~~~~ DONE ~~~~~~   ");
            show_spinner_until_next_task(|| {
                unmount(&mnt_path);
                sync();
            });

            sleep(1);
            println!("\n    Remove microSD & insert into");
            println!("    Beagle Bone Black. Hold the button");
            println!("    found near the microSD slot while you");
            println!("    apply power. Let go once all lights");
            println!("    begin flashing.\n");
            sleep(1);
            println!("        ~~~~~~~~~~~~~~~~~~~~\n");
        } else {
            println!("\n        Must be ran as root!");
            sleep(5);
            println!("\n\n         ~~~~ EXITING ~~~~");
            sleep(3);
            clear_screen();
        }
        break;
    }
}

fn clear_screen() {
    print!("{}[2J", 27 as char);
}

fn get_drives_info() -> Vec<String> {
    let output = Command::new("lsblk")
    .arg("-d")
    .arg("-n")
    .arg("-o")
    .arg("NAME")
    .output()
    .expect("Failed to execute lsblk command");

    let drives = String::from_utf8_lossy(&output.stdout);
    let drives: Vec<&str> = drives.split_whitespace().collect();

    let mut drives_info = Vec::new();
    for (i, drive) in drives.iter().enumerate() {
        let total_space = get_total_space(drive);
        if total_space < 1 * 1024 * 1024 * 1024 {
            continue;
        }
        let mount_points = get_mount_points(drive);
        if mount_points.contains("/boot/efi") {
            continue;
        }
        let available_space = get_available_space(drive);
        drives_info.push(format!("drive{} /dev/{} {} {:.2} GB", i, drive, available_space, total_space as f64 / (1024.0 * 1024.0 * 1024.0)));
    }
    drives_info
}

fn print_drives_info(drives_info: &[String]) {
    print_header();
    for drive in drives_info {
        let drive_array: Vec<&str> = drive.split_whitespace().collect();
        let drive_name = drive_array[0];
        let device = drive_array[1];
        let avail = drive_array[2];
        let total = drive_array[3..].join(" ");
        print_row(drive_name, device, avail, &total);
    }
    println!("");
    for _ in drives_info {
        println!("");
    }
}

fn select_device(drives_info: &[String]) -> String {
    loop {
        print!(" > Select target drive for flashing: ");
        io::stdout().flush().unwrap();
        let mut targ = String::new();
        io::stdin().read_line(&mut targ).unwrap();
        let targ = targ.trim();
        if let Some(device) = get_device(drives_info, targ) {
            return device.to_string();
        } else {
            clear_screen();
            println!("  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
            println!("  Beagle Bone Black Arch Installer");
            println!("  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");

            print_drives_info(drives_info);
            println!("   Invalid choice. Please try again.\n");
        }
    }
}

fn get_total_space(drive: &str) -> u64 {
    let output = Command::new("lsblk")
    .arg("-d")
    .arg("-n")
    .arg("-o")
    .arg("SIZE")
    .arg("-b")
    .arg(format!("/dev/{}", drive))
    .output()
    .expect("Failed to execute lsblk command for total space");

    let total_space = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(0);
    total_space
}

fn get_mount_points(drive: &str) -> String {
    let output = Command::new("lsblk")
    .arg("-o")
    .arg("MOUNTPOINT")
    .arg("-n")
    .arg(format!("/dev/{}", drive))
    .output()
    .expect("Failed to execute lsblk command for mount points");

    let mount_points = String::from_utf8_lossy(&output.stdout).replace("\n", " :: ").trim().to_string();
    mount_points
}

fn get_available_space(drive: &str) -> String {
    let output = Command::new("lsblk")
    .arg("-d")
    .arg("-n")
    .arg("-o")
    .arg("SIZE")
    .arg(format!("/dev/{}", drive))
    .output()
    .expect("Failed to execute lsblk command for available space");

    let available_space = String::from_utf8_lossy(&output.stdout).trim().to_string();
    available_space
}

fn print_header() {
    println!("{:<8} | {:<10} | {:<8} | {:<8}", "Drive", "Device", "Avail", "Total");
    println!("{:<8}-+-{:<10}-+-{:<8}-+-{:<8}", "--------", "----------", "--------", "--------");
}

fn print_row(drive: &str, device: &str, avail: &str, total: &str) {
    println!("{:<8} | {:<10} | {:<8} | {:<8}", drive, device, avail, total);
}

fn get_device<'a>(drives_info: &'a [String], targ: &'a str) -> Option<&'a str> {
    for drive in drives_info {
        let drive_array: Vec<&str> = drive.split_whitespace().collect();
        let drive_name = drive_array[0];
        let device = drive_array[1];
        let select_var = format!("select_{}", &drive_name[5..]);

        // Check if the input matches "drive[n]" or "[n]"
        if targ == drive_name || targ == select_var || targ == &drive_name[5..] {
            return Some(device);
        }
    }
    None
}

fn download_file_if_not_exists(file_path: &str) {
    if !Path::new(file_path).exists() {
        Command::new("wget")
        .arg("http://os.archlinuxarm.org/os/ArchLinuxARM-am33x-latest.tar.gz")
        .arg("-O")
        .arg(file_path)
        .status()
        .expect("Failed to download file");
    }
}

fn terminate_processes_using(path: &str) {
    Command::new("fuser")
    .arg("-k")
    .arg(path)
    .status()
    .ok();
}

fn unmount(path: &str) {
    Command::new("umount")
    .arg("-f")
    .arg(path)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .ok();
}

fn unmount_force(path: &str) {
    Command::new("umount")
    .arg("-l")
    .arg(path)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .ok();
}

fn unmount_all_partitions(device: &str) {
    let output = Command::new("lsblk")
    .arg("-ln")
    .arg(format!("/dev/{}", device))
    .output()
    .expect("Failed to execute lsblk command to list partitions");

    let partitions = String::from_utf8_lossy(&output.stdout);
    for partition in partitions.split_whitespace() {
        let partition_path = format!("/dev/{}", partition);
        unmount(&partition_path);
    }
}

fn zero_out_device(device: &str) {
    Command::new("dd")
    .arg("if=/dev/zero")
    .arg(format!("of={}", device))
    .arg("bs=1M")
    .arg("count=8")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to zero out device");
}

fn delete_partitions(device: &str) {
    Command::new("sfdisk")
    .arg("-f")
    .arg("--delete")
    .arg(device)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to delete partitions");
}

fn create_new_partition_table(device: &str) {
    Command::new("parted")
    .arg(device)
    .arg("--script")
    .arg("mklabel")
    .arg("msdos")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to create new partition table");

    Command::new("parted")
    .arg(device)
    .arg("--script")
    .arg("mkpart")
    .arg("primary")
    .arg("ext4")
    .arg("2048s")
    .arg("100%")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to create new partition");
}

fn create_ext4_filesystem(device: &str) {
    Command::new("mkfs.ext4")
    .arg(device)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to create ext4 filesystem");
}

fn mount_device(device: &str, path: &str) {
    Command::new("mount")
    .arg(device)
    .arg(path)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to mount device");
}

fn extract_tarball(file_path: &str, dest: &str) {
    Command::new("bsdtar")
    .arg("-xpf")
    .arg(file_path)
    .arg("-C")
    .arg(dest)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to extract tarball");
}

fn sync() {
    Command::new("sync")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to sync");
}

fn write_mlo_to_device(device: &str, mlo_path: &str) {
    Command::new("dd")
    .arg(format!("if={}", mlo_path))
    .arg(format!("of={}", device))
    .arg("count=1")
    .arg("seek=1")
    .arg("conv=notrunc")
    .arg("bs=128k")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to write MLO to device");
}

fn write_uboot_to_device(device: &str, uboot_img_path: &str) {
    Command::new("dd")
    .arg(format!("if={}", uboot_img_path))
    .arg(format!("of={}", device))
    .arg("count=2")
    .arg("seek=1")
    .arg("conv=notrunc")
    .arg("bs=384k")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .expect("Failed to write u-boot to device");
}

fn copy_files_to_mnt(username: &str, mnt_path: &str) {
    let src_tarball = format!("/home/{}/ArchLinuxARM-am33x-latest.tar.gz", username);
    let dest_tarball = format!("{}/home/alarm/ArchLinuxARM-am33x-latest.tar.gz", mnt_path);
    let src_script = format!("/home/{}/bbb_eMMC.sh", username);
    let dest_script = format!("{}/home/alarm/bbb_eMMC.sh", mnt_path);

    fs::copy(&src_tarball, &dest_tarball).expect("Failed to copy tarball to mnt");
    fs::copy(&src_script, &dest_script).expect("Failed to copy script to mnt");
}

fn sleep(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
}

fn ensure_file_exists(file_path: &str) {
    if !Path::new(file_path).exists() {
        panic!("File does not exist: {}", file_path);
    }
}

fn prepare_mount_path(mnt_path: &str) {
    terminate_processes_using(mnt_path);
    unmount(mnt_path);
    fs::remove_dir_all(mnt_path).ok();
    fs::create_dir(mnt_path).unwrap();
}

fn ensure_directory_exists(dir_path: &str) {
    if !Path::new(dir_path).exists() {
        fs::create_dir_all(dir_path).expect("Failed to create directory");
    }
}

fn is_mounted(device: &str) -> bool {
    let output = Command::new("lsblk")
    .arg("-no")
    .arg("MOUNTPOINT")
    .arg(device)
    .output()
    .expect("Failed to execute lsblk command to check mount status");

    !String::from_utf8_lossy(&output.stdout).trim().is_empty()
}

fn show_spinner(duration: u64) {
    let spinner = vec!['|', '/', '-', '\\'];
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(duration);
    while start.elapsed() < duration {
        for &c in &spinner {
            print!("\r{}", c);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    }
    print!("\r \r");
    io::stdout().flush().unwrap();
}

fn show_spinner_until_next_task<F: FnOnce()>(task: F) {
    let spinner = vec!['|', '/', '-', '\\'];
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done_clone = done.clone();

    let handle = thread::spawn(move || {
        while !done_clone.load(std::sync::atomic::Ordering::Relaxed) {
            for &c in &spinner {
                if done_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                print!("{}[3G{}", 27 as char, c); // Move cursor 3 spaces back and print spinner without space
                io::stdout().flush().unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        }
        print!("\r   \r"); // Clear the spinner when done
        io::stdout().flush().unwrap();
    });

    task();

    done.store(true, std::sync::atomic::Ordering::Relaxed);
    handle.join().unwrap();
}
