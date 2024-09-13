use std::process::Command;
use std::fs;
use std::path::Path;
use std::env;

const NETDATA_INSTALL_DIR: &str = if cfg!(windows) { "C:\\Program Files\\Netdata" } else { "/opt/netdata" };
const NETDATA_CONFIG_DIR: &str = if cfg!(windows) { "C:\\ProgramData\\netdata" } else { "/etc/netdata" };
const LOG_CONFIG_FILE: &str = if cfg!(windows) { "C:\\ProgramData\\netdata\\go.d\\logs.conf" } else { "/etc/netdata/go.d/logs.conf" };
const CLAIM_TOKEN: &str = "2DWIb-OeXPHH4N9oyvRhCxyAZmEx1iae4BiDH-OAPjz6io6arpoH4m8hMhSTLFYFtKPq_jnWKcY3gSkgOILGCAGZIm7LNPeHgu_Ahu0xsKA-ZU92as5rF_b0jx3j6wtqnxfjfmU";
const CLAIM_ROOMS: &str = "bce3dbd7-0e36-4a77-ba2f-35b56db95b1f";
const CLAIM_URL: &str = "https://app.netdata.cloud";

fn main() {
    if !is_root() {
        println!("This script must be run with root/administrator privileges.");
        return;
    }

    if !netdata_installed() {
        install_netdata();
    }

    configure_log_monitoring();
    restart_netdata();

    println!("Netdata installation and configuration complete.");
}

fn is_root() -> bool {
    if cfg!(unix) {
        Command::new("id").arg("-u").output().map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0").unwrap_or(false)
    } else if cfg!(windows) {
        // On Windows, check if running with administrator privileges
        Command::new("net").args(&["session"]).status().map(|s| s.success()).unwrap_or(false)
    } else {
        false
    }
}

fn netdata_installed() -> bool {
    Path::new(NETDATA_INSTALL_DIR).exists()
}

fn install_netdata() {
    println!("Installing Netdata...");
    
    let install_command = if command_exists("wget") {
        format!("wget -O /tmp/netdata-kickstart.sh https://get.netdata.cloud/kickstart.sh && sh /tmp/netdata-kickstart.sh --nightly-channel --claim-token {} --claim-rooms {} --claim-url {}", CLAIM_TOKEN, CLAIM_ROOMS, CLAIM_URL)
    } else if command_exists("curl") {
        format!("curl https://get.netdata.cloud/kickstart.sh > /tmp/netdata-kickstart.sh && sh /tmp/netdata-kickstart.sh --nightly-channel --claim-token {} --claim-rooms {} --claim-url {}", CLAIM_TOKEN, CLAIM_ROOMS, CLAIM_URL)
    } else {
        println!("Neither wget nor curl is available. Please install one of them and try again.");
        return;
    };

    let output = if cfg!(unix) {
        Command::new("sh").arg("-c").arg(&install_command).output()
    } else {
        Command::new("powershell").arg("-Command").arg(&install_command).output()
    };

    match output {
        Ok(output) => {
            println!("Netdata installation output: {}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                println!("Netdata installation failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        },
        Err(e) => println!("Failed to execute installation command: {}", e),
    }
}

fn configure_log_monitoring() {
    println!("Configuring Netdata for log monitoring...");
    
    // Get log file paths from environment variable or use default paths
    let log_paths = env::var("LOG_PATHS").unwrap_or_else(|_| String::from("/var/log/syslog,/var/log/auth.log"));
    let log_paths: Vec<&str> = log_paths.split(',').collect();

    let mut config = String::from("logs:\n  enabled: yes\n  paths:\n");
    for path in log_paths {
        config.push_str(&format!("    - '{}'\n", path.trim()));
    }

    config.push_str(r#"  exclude_patterns: []
  
  custom_parsers:
    - name: custom_parser_name
      pattern: '(?<time>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (?<severity>\w+) (?<message>.*)'
      time_format: '%Y-%m-%d %H:%M:%S'
"#);

    fs::create_dir_all(Path::new(LOG_CONFIG_FILE).parent().unwrap()).unwrap();
    fs::write(LOG_CONFIG_FILE, config).unwrap();
}

fn restart_netdata() {
    println!("Restarting Netdata service...");
    let restart_command = if cfg!(unix) {
        "systemctl restart netdata"
    } else {
        "Restart-Service -Name netdata"
    };

    let output = if cfg!(unix) {
        Command::new("sh").arg("-c").arg(restart_command).output()
    } else {
        Command::new("powershell").arg("-Command").arg(restart_command).output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Netdata service restarted successfully.");
            } else {
                println!("Failed to restart Netdata service: {}", String::from_utf8_lossy(&output.stderr));
            }
        },
        Err(e) => println!("Failed to execute restart command: {}", e),
    }
}

fn command_exists(cmd: &str) -> bool {
    if cfg!(unix) {
        Command::new("which").arg(cmd).status().map(|s| s.success()).unwrap_or(false)
    } else {
        Command::new("where").arg(cmd).status().map(|s| s.success()).unwrap_or(false)
    }
}