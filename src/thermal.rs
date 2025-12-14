use anyhow::{Context, Result};
use log::error;
use std::{
    fs::{self, File},
    io::{Read, Seek},
};

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Row, Table},
};

#[derive(Debug, Default)]
pub struct Thermal {
    pub sensors: Vec<Sensor>,
}
#[derive(Debug)]
pub struct Sensor {
    file: File,
    name: String,
    temperature: f32,
}

impl Sensor {
    pub fn refresh(&mut self) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;
        self.temperature = buffer.trim().parse::<f32>()? / 1000.0;
        Ok(())
    }
}

impl Thermal {
    pub fn new() -> Self {
        let sensors = match Thermal::init() {
            Ok(sensors) => sensors,
            Err(e) => {
                error!("{}", e);
                Vec::new()
            }
        };

        Self { sensors }
    }

    pub fn init() -> Result<Vec<Sensor>> {
        let mut sensors: Vec<Sensor> = Vec::new();
        let entries = fs::read_dir("/sys/devices/virtual/thermal/")
            .context("Failed to read from the directory /sys/devices/virtual/thermal")?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            if let Some(file_name) = entry_path.file_name()
                && let Some(name) = file_name.to_str()
                && name.starts_with("thermal_zone")
                && entry_path.is_dir()
            {
                let name = match fs::read_to_string(entry_path.join("type")) {
                    Ok(v) => v.split('-').next().unwrap().trim().to_string(),
                    Err(_) => continue,
                };

                let temperature_path = &entry_path.join("temp");
                let mut temperature_file = match File::open(temperature_path) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                };

                let mut buffer = String::new();
                if let Err(e) = temperature_file.read_to_string(&mut buffer) {
                    error!("{}", e);
                    continue;
                }

                let temperature = buffer.trim().parse::<f32>()? / 1000.0;

                let sensor = Sensor {
                    file: temperature_file,
                    name,
                    temperature,
                };
                sensors.push(sensor);
            }
        }

        Ok(sensors)
    }

    pub fn refresh(&mut self) {
        for sensor in &mut self.sensors {
            if let Err(e) = sensor.refresh() {
                error!("{}", e);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows: Vec<Row> = self
            .sensors
            .iter()
            .map(|sensor| {
                let temperature = match sensor.temperature {
                    t if t < -25.0 => " - ".to_string(),
                    t => format!("{:.1} C", t),
                };

                Row::new(vec![sensor.name.to_owned(), temperature])
            })
            .collect();

        let widths = [Constraint::Length(8), Constraint::Length(11)];

        let table = Table::new(rows, widths)
            .header(Row::new(vec!["Sensor", "Temperature"]).style(Style::new().bold()))
            .block(
                Block::default()
                    .title("Thermal")
                    .title_style(Style::new().bold())
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1)),
            );
        frame.render_widget(table, block);
    }
}
