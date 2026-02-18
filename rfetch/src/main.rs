use std::{fs, process::Command, path::Path};
use sysinfo::{System, Disks};

fn main () {
    let user = get_username();
    let host = get_hostname();
    let os = get_os();
    let init = detect_init();
    let kernel = get_kernel();
    let uptime = get_uptime();
    let mem = get_memory();
    let swap = get_swap();
    let _storage_boot = get_storage("/boot");
    let _storage_root = get_storage("/");
    let _storage_home = get_storage("/home");
    let shell = get_shell();

    println!("{}@{}", user, host);
    println!("----------");
    println!("OS      : {}", os);
    println!("Init    : {}", init);
    println!("Kernel  : {}", kernel);
    println!("Uptime  : {}", uptime);
    println!("Shell   : {}", shell);
    println!("Memory  : {}", mem);
    println!("Swap    : {}", swap);
    println!("Storage : {}", _storage_boot);
    println!("          {}", _storage_root);
    println!("          {}", _storage_home);
}

fn get_username() -> String {
    std::env::var("USER").unwrap_or_else(|_|"unknown".into())
}

fn get_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string()
}

fn get_os() -> String {
    let content = fs::read_to_string("/etc/os-release")
        .unwrap_or_else(|_| return "unknow".into());
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .replace ("PRETTY_NAME=", "")
                    .replace ('"', "");
            }
        }
    "unknown".into()
}

fn detect_init() -> String {
    let comm = fs::read_to_string("/proc/1/comm")
        .unwrap_or_default()
        .trim()
        .to_string();
  
    let exe = fs::read_link("/proc/1/exe")
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();
    
    match comm.as_str() {
        "systemd" => "systemd".into(),
        "runit" | "runsvinit" => "runit".into(),
        "s6-svscan" => "s6".into(),
        "init" => {
            if exe.contains("openrc") {
                "openrc".into()
            } else {
                "sysvinit".into()
            }
        }
        _ => {
            if Path::new("/run/systemd/systemd").exists() {
                "systemd (fallback)".into()
            } else {
                format!("unknown ({})", comm)
            }
        }
    }
}


fn get_kernel() -> String {
    let output = Command::new("uname")
        .arg("-r")
        .output();

    match output {
        Ok(o) =>
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "unknown".into(),
    }
}

fn get_uptime() -> String {
    let content = fs::read_to_string("/proc/uptime")
        .unwrap_or_else(|_| return "unknown".into());
   
    let seconds: f64 = content
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    
    let minutes = (seconds / 60.0) as u64;
    let hours = minutes / 60;
    let mins = minutes %60 ;
    
    format!("{}h {}m", hours, mins)
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .map( |s| s.to_string_lossy().into_owned())
        })
    .unwrap_or("unknown".into())
}

fn get_memory() -> String {
    let content = fs::read_to_string("/proc/meminfo")
        .unwrap_or_else(|_| return "unknown".into());

    let mut total = 0;
    let mut available = 0;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = extract_kb(line);
        } else if line.starts_with("MemAvailable: ") {
            available = extract_kb(line);
        }
    }

    if total == 0 {
        return "unknown".into();
    }

    let used = total - available;
    format!(
        "{:.1} GiB / {:.1} GiB",
        kb_to_gib(used),
        kb_to_gib(total)
    )
}

fn kb_to_gib(kb: u64) -> f64 {
    kb as f64 / 1024.0 / 1024.0
}

fn extract_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0)
}

fn get_swap() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total = sys.total_swap() as f64 / 1_000_000_000.0;
    let used = sys.used_swap()as f64 / 1_000_000_000.0;

    format!("{:.1} GiB / {:.1} GiB", used, total)
}

fn get_storage(path: &str) -> String {
    let disks=Disks::new_with_refreshed_list();
    let mut best_match = None;
    for disk in &disks {
        let mount = disk.mount_point().to_string_lossy();
        if path.starts_with(mount.as_ref()) {
            match &best_match {
                Some((best_mount_len, _)) => {
                    if mount.len() > *best_mount_len {
                        best_match = Some((mount.len(), disk));
                    }
                }
                    None => {
                        best_match = Some((mount.len(), disk));
                    }
                }
            }
        }
                    
        if let Some((_, disk)) = best_match {
            let total = disk.total_space() as f64 / 1_000_000_000.0;
            let avail = disk.available_space() as f64 / 1_000_000_000.0;
            let used = total - avail;

            format!("{:.1} GiB / {:.1} GiB ({})", used, total, path)
        } else {
            format!("N/A ({})", path)
        }
    }
