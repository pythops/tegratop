use anyhow::{Context, Result};
use log::error;
use std::{
    fs::{self, File},
    io::{Read, Seek},
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table},
    Frame,
};
use regex::Regex;

#[derive(Debug, Default)]
pub struct Power {
    pub channels: Vec<Channel>,
    pub nvpmode: Option<NVPMode>,
}

#[derive(Debug)]
pub struct Channel {
    current_file: File,
    voltage_file: File,
    name: String,
    power: f32,
}

#[derive(Debug, Default)]
pub struct NVPMode {
    file: Option<File>,
    mode: Option<Mode>,
    available_modes: Vec<Mode>,
}

#[derive(Debug, Clone, Default)]
pub struct Mode {
    id: u8,
    name: String,
}

impl Channel {
    pub fn refresh(&mut self) -> Result<()> {
        self.current_file.seek(std::io::SeekFrom::Start(0))?;
        self.voltage_file.seek(std::io::SeekFrom::Start(0))?;

        let mut buffer = String::new();
        self.current_file.read_to_string(&mut buffer)?;
        let current = buffer.trim().parse::<f32>()?;

        let mut buffer = String::new();
        self.voltage_file.read_to_string(&mut buffer)?;
        let voltage = buffer.trim().parse::<f32>()?;

        self.power = (current * voltage / 1000.0).round();

        Ok(())
    }
}

impl NVPMode {
    fn new() -> Result<Option<Self>> {
        let mut file =
            File::open("/etc/nvpmodel.conf").context("Failed to open /etc/nvpmodel.conf")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let mut modes: Vec<Mode> = Vec::new();
        let pattern = r#"< POWER_MODEL ID=(\d+) NAME=(\d+)W >"#;

        let regex = Regex::new(pattern)?;

        for line in buffer.lines() {
            if let Some(captures) = regex.captures(line) {
                let id = captures.get(1).unwrap().as_str().parse::<u8>()?;
                let name = captures.get(2).unwrap().as_str().to_owned();

                let mode = Mode { id, name };

                modes.push(mode);
            }
        }

        // Current mode if active

        if let Ok(mut fd) = File::open("/var/lib/nvpmodel/status") {
            let mut buffer = String::new();
            fd.read_to_string(&mut buffer)?;

            let parts: Vec<&str> = buffer.split(':').collect();
            let current_mode_id = parts[1].parse::<u8>()?;

            if let Some(mode) = modes.iter().find(|&mode| mode.id == current_mode_id) {
                return Ok(Some(Self {
                    file: Some(fd),
                    mode: Some(mode.clone()),
                    available_modes: modes,
                }));
            }
        }

        // Default mode
        let pattern = r#"< PM_CONFIG DEFAULT=(\d+) >"#;

        let regex = Regex::new(pattern)?;

        for line in buffer.lines() {
            if let Some(captures) = regex.captures(line) {
                let default_id = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();

                if let Some(mode) = modes.iter().find(|&mode| mode.id == default_id) {
                    return Ok(Some(Self {
                        file: None,
                        mode: Some(mode.clone()),
                        available_modes: modes,
                    }));
                }
            }
        }

        Ok(None)
    }

    fn refresh(&mut self) -> Result<()> {
        if let Some(fd) = &mut self.file {
            fd.seek(std::io::SeekFrom::Start(0))?;
            let mut buffer = String::new();
            fd.read_to_string(&mut buffer)?;

            let parts: Vec<&str> = buffer.split(':').collect();
            let current_mode_id = parts[1].parse::<u8>()?;

            if let Some(mode) = self
                .available_modes
                .iter()
                .find(|&mode| mode.id == current_mode_id)
            {
                self.mode = Some(mode.clone());
            }
        }
        Ok(())
    }
}

impl Power {
    pub fn init() -> Result<Vec<Channel>> {
        let hwmon_dir_content = fs::read_dir("/sys/class/hwmon/")
            .context("Failed to read from the directory /sys/class/hwmon")?;

        let channel_regex = Regex::new(r"in(\d)_label")?;

        let mut channels: Vec<Channel> = Vec::new();

        for entry in hwmon_dir_content {
            let entry = entry?;
            let entry_path = entry.path();

            let hwmon_name = fs::read_to_string(entry_path.join("name"))?;

            if hwmon_name.trim() == "ina3221" {
                if let Ok(power_entries) = fs::read_dir(&entry_path) {
                    for f in power_entries.flatten() {
                        let path = f.path();
                        if let Some(file_name) = path.file_name() {
                            if let Some(name) = file_name.to_str() {
                                if let Some(captures) = channel_regex.captures(name) {
                                    if let Some(channel_index) = captures.get(1) {
                                        let channel_index = channel_index.as_str();

                                        let channel_name = match fs::read_to_string(&path) {
                                            Ok(v) => v,
                                            Err(_) => continue,
                                        };

                                        if !&entry_path
                                            .join(format!("curr{}_input", channel_index))
                                            .exists()
                                        {
                                            continue;
                                        }

                                        let current_path = &entry_path
                                            .join(format!("curr{}_input", channel_index));
                                        let mut current_file = File::open(current_path)?;
                                        let mut buffer = String::new();
                                        current_file.read_to_string(&mut buffer)?;
                                        let current = buffer.trim().parse::<f32>()?;

                                        let voltage_path =
                                            &entry_path.join(format!("in{}_input", channel_index));

                                        let mut voltage_file = File::open(voltage_path)?;
                                        let mut buffer = String::new();
                                        voltage_file.read_to_string(&mut buffer)?;
                                        let voltage = buffer.trim().parse::<f32>()?;

                                        let channel = Channel {
                                            current_file,
                                            voltage_file,
                                            name: channel_name,
                                            power: (current * voltage / 1000.0).round(),
                                        };

                                        channels.push(channel);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(channels)
    }

    pub fn new() -> Self {
        let nvpmode = match NVPMode::new() {
            Ok(mode) => mode,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        let channels = match Power::init() {
            Ok(channels) => channels,
            Err(e) => {
                error!("{}", e);
                Vec::new()
            }
        };

        Self { channels, nvpmode }
    }

    pub fn refresh(&mut self) {
        if let Some(nvpmode) = &mut self.nvpmode {
            if let Err(e) = nvpmode.refresh() {
                error!("{}", e);
            }
        }

        for channel in &mut self.channels {
            if let Err(e) = channel.refresh() {
                error!("{}", e);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let container = Block::default()
            .borders(Borders::ALL)
            .title("Power")
            .padding(Padding::horizontal(1))
            .title_style(Style::new().bold());

        let inside_container = container.inner(block);

        let (nvpmodel_block, power_consumption_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(1)])
                .split(inside_container);

            (chunks[0], chunks[1])
        };

        // nvpmodel
        let rows = [Row::new(vec![
            Cell::new("Mode").style(Style::default().bold()),
            Cell::new(match &self.nvpmode {
                Some(nvpmode) => match &nvpmode.mode {
                    Some(mode) => format!("{}W", &mode.name),
                    None => " - ".to_string(),
                },
                None => " - ".to_string(),
            }),
        ])];

        let widths = [Constraint::Length(16), Constraint::Length(7)];

        let nvpmodel = Table::new(rows, widths).block(Block::default());

        // Power consumption
        let rows: Vec<Row> = self
            .channels
            .iter()
            .map(|channel| {
                Row::new(vec![
                    channel.name.to_owned(),
                    format!("{} mW", channel.power),
                ])
            })
            .collect();

        let widths = [Constraint::Length(16), Constraint::Length(7)];

        let power = Table::new(rows, widths)
            .header(Row::new(vec!["Channel", "Power"]).style(Style::new().bold()))
            .block(Block::default());

        frame.render_widget(container, block);
        frame.render_widget(nvpmodel, nvpmodel_block);
        frame.render_widget(power, power_consumption_block);
    }
}
