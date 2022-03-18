use anyhow::anyhow;
use std::fs::{read_to_string, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

fn get_current_timestamp() -> u64 {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

const TARGET_DOMAIN: [&str; 5] = [
    "clientapi.myteamspeak.com",
    "versions.teamspeak.com",
    "tmspk.gg",
    "blacklist.teamspeak.com",
    "blacklist2.teamspeak.com",
];

fn staff() -> anyhow::Result<()> {
    let windir = std::env::var("windir")
        .map_err(|e| anyhow!("Got error while get windir variable: {:?}", e))?;

    let sub_dir = Path::new("system32").join("drivers").join("etc");

    let work_dir = PathBuf::from_str(windir.as_str()).unwrap().join(sub_dir);

    let hosts_path = work_dir.join("hosts");

    let backup_path = work_dir.join(format!("hosts_backup_{}", get_current_timestamp()));

    println!(
        "Hosts path: {:?}\nBackup path: {:?}",
        &hosts_path, &backup_path
    );

    std::fs::copy(&hosts_path, &backup_path)
        .map_err(|e| anyhow!("Got error while copy hosts to backup: {:?}", e))?;

    let hosts_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(hosts_path)
        .map_err(|e| anyhow!("Open hosts file(2) error: {:?}", e))?;

    let content = read_to_string(backup_path)
        .map_err(|e| anyhow!("Read backup hosts file error: {:?}", e))?;

    let mut writer = BufWriter::new(hosts_file);

    'outside: for line in content.lines() {
        let origin_content = line.trim();
        let line = format!("{}\n", line);
        if origin_content.starts_with('#') {
            writer
                .write(line.as_bytes())
                .map_err(|e| anyhow!("Got error while write hosts: {:?}", e))?;
            continue;
        }

        let content = if origin_content.contains('#') {
            origin_content.split_once('#').unwrap().0
        } else {
            origin_content
        };

        for domain in TARGET_DOMAIN {
            if content.contains(domain) {
                let buf = format!("#{}\n", origin_content);
                println!("Find {:?} match {:?}", origin_content, domain);
                writer
                    .write(buf.as_bytes())
                    .map_err(|e| anyhow!("Got error while write hosts: {:?}", e))?;
                continue 'outside;
            }
        }

        writer
            .write(line.as_bytes())
            .map_err(|e| anyhow!("Got error while write hosts: {:?}", e))?;
    }

    Ok(())
}

fn main() {
    staff().map_err(|e| eprintln!("Error: {:?}", e)).ok();
    let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
}
