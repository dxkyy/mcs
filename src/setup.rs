use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use crate::config::ServerConfig;

pub fn create_start_scripts(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    let bat_content = format!(
        r#"@echo off
java -Xms{memory} -Xmx{memory} -jar server.jar nogui
pause
"#,
        memory = config.memory
    );

    fs::write(path.join("start.bat"), bat_content)?;

    let sh_content = format!(
        r#"#!/bin/bash
java -Xms{memory} -Xmx{memory} -jar server.jar nogui
"#,
        memory = config.memory
    );

    fs::write(path.join("start.sh"), sh_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path.join("start.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path.join("start.sh"), perms)?;
    }

    println!("✓ Created start scripts");

    Ok(())
}

pub fn create_eula(path: &PathBuf) -> Result<()> {
    let eula_content = r#"#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).
eula=true
"#;

    fs::write(path.join("eula.txt"), eula_content)?;

    println!("✓ EULA automatically accepted");

    Ok(())
}
