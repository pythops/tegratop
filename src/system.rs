use anyhow::Context;
use log::error;
use std::fs::File;
use std::io::Read;
use std::io::Seek;

use anyhow::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table},
};

#[derive(Debug, Default)]
pub struct System {
    loadavg: Option<Loadavg>,
    uptime: Option<Uptime>,
}

#[derive(Debug)]
pub struct Uptime {
    file: File,
    value: String,
}

impl Uptime {
    fn new() -> Result<Self> {
        let mut file = File::open("/proc/uptime").context("Failed to open /proc/uptime")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let uptime_in_seconds = buffer.trim().split('.').next().unwrap().parse::<f64>()?;

        let days = (uptime_in_seconds / (60.0 * 60.0 * 24.0)) as u64;
        let hours = ((uptime_in_seconds / (60.0 * 60.0)) % 24.0) as u64;
        let minutes = ((uptime_in_seconds / 60.0) % 60.0) as u64;
        let seconds = (uptime_in_seconds % 60.0) as u64;

        let value = format!("{}days, {}h, {}min, {}s", days, hours, minutes, seconds);

        Ok(Self { file, value })
    }

    fn refresh(&mut self) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;

        let uptime_in_seconds = buffer.trim().split('.').next().unwrap().parse::<f64>()?;

        let days = (uptime_in_seconds / (60.0 * 60.0 * 24.0)) as u64;
        let hours = ((uptime_in_seconds / (60.0 * 60.0)) % 24.0) as u64;
        let minutes = ((uptime_in_seconds / 60.0) % 60.0) as u64;
        let seconds = (uptime_in_seconds % 60.0) as u64;
        self.value = format!("{}days, {}h, {}min, {}s", days, hours, minutes, seconds);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Loadavg {
    file: File,
    value: [f64; 3],
}

impl Loadavg {
    fn new() -> Result<Self> {
        let mut file = File::open("/proc/loadavg").context("Failed to open /proc/loadavg")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let loadavg_values: Vec<&str> = buffer.split_whitespace().collect();

        let load_avg_1min: f64 = loadavg_values[0].parse()?;
        let load_avg_10min: f64 = loadavg_values[1].parse()?;
        let load_avg_15min: f64 = loadavg_values[2].parse()?;

        Ok(Self {
            file,
            value: [load_avg_1min, load_avg_10min, load_avg_15min],
        })
    }

    fn refresh(&mut self) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer).unwrap();

        let loadavg_values: Vec<&str> = buffer.split_whitespace().collect();

        let load_avg_1min: f64 = loadavg_values[0].parse()?;
        let load_avg_10min: f64 = loadavg_values[1].parse()?;
        let load_avg_15min: f64 = loadavg_values[2].parse()?;

        self.value = [load_avg_1min, load_avg_10min, load_avg_15min];
        Ok(())
    }
}

impl System {
    pub fn new() -> Self {
        let loadavg = Loadavg::new().map_or_else(
            |e| {
                error!("{}", e);
                None
            },
            Some,
        );

        let uptime = Uptime::new().map_or_else(
            |e| {
                error!("{}", e);
                None
            },
            Some,
        );

        Self { loadavg, uptime }
    }

    pub fn refresh(&mut self) {
        if let Some(load_avg) = &mut self.loadavg {
            if let Err(e) = load_avg.refresh() {
                error!("{}", e);
            }
        };

        if let Some(uptime) = &mut self.uptime {
            if let Err(e) = uptime.refresh() {
                error!("{}", e);
            }
        };
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows: [Row; 3] = [
            Row::new(vec![
                Cell::new("Load avg").style(Style::default().bold()),
                Cell::new(match &self.loadavg {
                    Some(loadavg) => format!(
                        "{:.2} {:.2} {:.2}",
                        loadavg.value[0], loadavg.value[1], loadavg.value[2]
                    ),
                    None => " - ".to_string(),
                }),
            ]),
            Row::new(vec!["", ""]),
            Row::new(vec![
                Cell::new("Uptime").style(Style::default().bold()),
                Cell::new(match &self.uptime {
                    Some(uptime) => uptime.value.to_string(),
                    None => " - ".to_string(),
                }),
            ]),
        ];

        let widths = [Constraint::Length(10), Constraint::Fill(1)];

        let system = Table::new(rows, widths).block(
            Block::default()
                .title("System")
                .title_style(Style::new().bold())
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        );
        frame.render_widget(system, block);
    }
}
