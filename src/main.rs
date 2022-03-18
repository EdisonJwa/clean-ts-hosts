use anyhow::anyhow;
use std::fs::{read, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, Write};
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

    let work_dir = PathBuf::from_str(windir.as_str())
        .unwrap()
        .join(Path::new("system32/derivers/etc/"));

    let hosts_path = work_dir.join("hosts");

    let backup_path = work_dir.join(format!("hosts_backup_{}", get_current_timestamp()));

    let hosts_file = File::open(&hosts_path)
        .map_err(|e| anyhow!("Open hosts file {:?} error: {:?}", &hosts_path, e))?;

    let backup_file = OpenOptions::new()
        .create_new(true)
        .open(&backup_path)
        .map_err(|e| anyhow!("Open backup hosts file {:?} error: {:?}", &backup_path, e))?;

    let mut reader = BufReader::new(hosts_file);

    let mut writer = BufWriter::new(backup_file);

    std::io::copy(&mut reader, &mut writer)
        .map_err(|e| anyhow!("Got error while copy hosts to backup: {:?}", e))?;

    drop(writer);
    drop(reader);

    let hosts_file = File::open(hosts_path)
        .map_err(|e| anyhow!("Open hosts file {:?} error: {:?}", &hosts_path, e))?;

    let backup_file = File::open(&backup_path)
        .map_err(|e| anyhow!("Open backup hosts file {:?} error: {:?}", &backup_path, e))?;

    let mut reader = BufReader::new(backup_file);
    let mut writer = BufWriter::new(hosts_file);

    let mut buff = String::new();
    'outside: loop {
        let size = reader
            .read_line(&mut buff)
            .map_err(|e| anyhow!("Got error while read backup file: {:?}", e))?;

        if size == 0 {
            break;
        }

        let origin_content = buff.trim();
        if origin_content.starts_with('#') {
            writer
                .write(buff.as_bytes())
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
                let buf = format!("#{}", origin_content);
                writer
                    .write(buf.as_bytes())
                    .map_err(|e| anyhow!("Got error while write hosts: {:?}", e))?;
                continue 'outside;
            }
        }

        writer
            .write(buff.as_bytes())
            .map_err(|e| anyhow!("Got error while write hosts: {:?}", e))?;
    }

    Ok(())
}

fn main() {
    staff().map_err(|e| eprintln!("Error: {:?}", e)).ok();
    let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
}
