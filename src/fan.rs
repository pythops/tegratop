use anyhow::Context;
use tracing::error;

use anyhow::Result;
use std::{
    fs::{self, File},
    io::{Read, Seek},
};
use strum_macros::Display;

use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Row, Table},
    Frame,
};

#[derive(Debug, Default)]
pub struct Fan {
    rpm: Option<FanRPM>,
    profile: Option<FanProfile>,
}

#[derive(Debug)]
struct FanRPM {
    file: File,
    value: usize,
}

#[derive(Debug)]
struct FanProfile {
    file: File,
    value: Profile,
}

#[derive(Debug, Display)]
enum Profile {
    Quiet,
    Cool,
    Unknown,
}

impl FanRPM {
    fn new() -> Result<Option<Self>> {
        let entries = fs::read_dir("/sys/class/hwmon/")
            .context("Failed to read from the directory /sys/class/hwmon")?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.join("rpm").exists() {
                let rpm_file_path = entry_path.join("rpm");
                let mut file = File::open(&rpm_file_path)
                    .context(format!("Failed to read from {}", &rpm_file_path.display()))?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;

                let fan_rpm = buffer.trim().parse::<usize>()?;

                return Ok(Some(FanRPM {
                    file,
                    value: fan_rpm,
                }));
            }
        }

        Ok(None)
    }

    fn refresh(&mut self) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;
        let fan_rpm = buffer.trim().parse::<usize>()?;
        self.value = fan_rpm;
        Ok(())
    }
}

impl FanProfile {
    fn new() -> Result<Option<Self>> {
        let mut file = File::open("/etc/nvfancontrol.conf")
            .context("Faile to read from /etc/nvfancontrol.conf")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let ["FAN_DEFAULT_PROFILE", mode] = parts.as_slice() {
                match *mode {
                    "quiet" => {
                        return Ok(Some(FanProfile {
                            file,
                            value: Profile::Quiet,
                        }))
                    }
                    "cool" => {
                        return Ok(Some(FanProfile {
                            file,
                            value: Profile::Cool,
                        }))
                    }
                    _ => {
                        return Ok(Some(FanProfile {
                            file,
                            value: Profile::Unknown,
                        }))
                    }
                }
            }
        }

        Ok(None)
    }

    fn refresh(&mut self) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let ["FAN_DEFAULT_PROFILE", mode] = parts.as_slice() {
                match *mode {
                    "quiet" => {
                        self.value = Profile::Quiet;
                    }
                    "cool" => {
                        self.value = Profile::Cool;
                    }
                    _ => {
                        self.value = Profile::Unknown;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Fan {
    pub fn new() -> Self {
        let rpm = match FanRPM::new() {
            Ok(rpm) => rpm,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        let profile = match FanProfile::new() {
            Ok(profile) => profile,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        Self { rpm, profile }
    }

    pub fn refresh(&mut self) {
        if let Some(rpm) = &mut self.rpm {
            if let Err(e) = rpm.refresh() {
                error!("{}", e);
            }
        }
        if let Some(profile) = &mut self.profile {
            if let Err(e) = profile.refresh() {
                error!("{}", e);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows: [Row; 1] = [Row::new(vec![
            match &self.profile {
                Some(profile) => profile.value.to_string().to_lowercase(),
                None => " - ".to_string(),
            },
            match &self.rpm {
                Some(rpm) => rpm.value.to_string(),
                None => " - ".to_string(),
            },
        ])];

        let widths = [Constraint::Length(10), Constraint::Length(10)];

        let fan = Table::new(rows, widths)
            .header(Row::new(vec!["Profile", "RPM"]).style(Style::new().bold()))
            .block(
                Block::default()
                    .title("Fan")
                    .title_style(Style::new().bold())
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL),
            );

        frame.render_widget(fan, block);
    }
}
