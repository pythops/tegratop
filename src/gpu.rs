use anyhow::{Context, Result};
use log::error;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Padding, Row, Table},
    Frame,
};
use std::{
    fs::{self, File},
    io::{Read, Seek},
};

#[derive(Debug, Default)]
pub struct GPU {
    pub load: Option<GPULoad>,
    pub frequency: Option<GPUFrequency>,
}

#[derive(Debug)]
pub struct GPULoad {
    file: File,
    load: f64,
}

impl GPULoad {
    fn new() -> Result<Option<Self>> {
        let gpu_names = ["gv11b", "gp10b", "ga10b", "gpu"];

        let entries = fs::read_dir("/sys/class/devfreq")
            .context("Failed to read from the directory /sys/class/devfreq")?;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if gpu_names.iter().any(|gpu_name| name_str.contains(gpu_name)) {
                        // Load
                        let gpu_load_path = path.join("device/load");
                        let mut file = File::open(&gpu_load_path)
                            .context(format!("Failed to open {}", gpu_load_path.display()))?;
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)?;

                        let load = (buffer.trim().parse::<f64>()? / 10.0).round();

                        return Ok(Some(GPULoad { file, load }));
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
        self.load = (buffer.trim().parse::<f64>()? / 10.0).round();
        Ok(())
    }
}

#[derive(Debug)]
pub struct GPUFrequency {
    current_frequency_file: File,
    max_frequency_file: File,
    current_frequency: usize,
    max_frequency: usize,
}

impl GPUFrequency {
    fn new() -> Result<Option<Self>> {
        let gpu_names = ["gv11b", "gp10b", "ga10b", "gpu"];

        let entries = fs::read_dir("/sys/class/devfreq")
            .context("Failed to read from the directory /sys/class/devfreq")?;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if gpu_names.iter().any(|gpu_name| name_str.contains(gpu_name)) {
                        // Current freq

                        let current_frequency_path = path.join("cur_freq");
                        let mut current_frequency_file = File::open(&current_frequency_path)
                            .context(format!(
                                "Failed to open {}",
                                current_frequency_path.display()
                            ))?;
                        let mut buffer = String::new();
                        current_frequency_file.read_to_string(&mut buffer)?;
                        let current_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

                        // Max freq

                        let max_frequency_path = path.join("max_freq");
                        let mut max_frequency_file = File::open(&max_frequency_path)
                            .context(format!("Failed to open {}", max_frequency_path.display()))?;

                        let mut buffer = String::new();
                        max_frequency_file.read_to_string(&mut buffer)?;
                        let max_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

                        return Ok(Some(GPUFrequency {
                            current_frequency_file,
                            max_frequency_file,
                            current_frequency,
                            max_frequency,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    fn refresh(&mut self) -> Result<()> {
        self.current_frequency_file
            .seek(std::io::SeekFrom::Start(0))?;
        self.max_frequency_file.seek(std::io::SeekFrom::Start(0))?;

        // Current freq

        let mut buffer = String::new();
        self.current_frequency_file.read_to_string(&mut buffer)?;
        self.current_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

        // Max freq

        let mut buffer = String::new();
        self.max_frequency_file.read_to_string(&mut buffer)?;
        self.max_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

        Ok(())
    }
}

impl GPU {
    pub fn new() -> Self {
        let load = match GPULoad::new() {
            Ok(load) => load,

            Err(e) => {
                error!("{}", e);
                None
            }
        };

        let frequency = match GPUFrequency::new() {
            Ok(frequency) => frequency,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        Self { load, frequency }
    }

    pub fn refresh(&mut self) {
        if let Some(load) = &mut self.load {
            if let Err(e) = load.refresh() {
                error!("{}", e);
            }
        }

        if let Some(frequency) = &mut self.frequency {
            if let Err(e) = frequency.refresh() {
                error!("{}", e);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let container = Block::default()
            .borders(Borders::ALL)
            .title("GPU")
            .padding(Padding::horizontal(1))
            .title_style(Style::new().bold());

        let inside_container = container.inner(block);

        let (left_block, right_block) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(inside_container);

            (chunks[0], chunks[1])
        };

        let frequency = Table::new(
            [Row::new(vec![
                Cell::new("Freq").style(Style::default().bold()),
                Cell::new(match &self.frequency {
                    Some(frequency) => format!(
                        "{}MHz / {}MHz",
                        frequency.current_frequency, frequency.max_frequency
                    ),
                    None => " - ".to_string(),
                }),
            ])],
            [Constraint::Length(4), Constraint::Length(17)],
        )
        .block(Block::default());

        let load = BarChart::default()
            .block(Block::default())
            .bar_width(1)
            .bar_gap(1)
            .group_gap(0)
            .direction(Direction::Horizontal)
            .data(
                BarGroup::default().bars(&[Bar::default()
                    .label(Line::styled("Load", Style::default().bold()))
                    .value(match &self.load {
                        Some(load) => load.load.round() as u64,
                        None => 0,
                    })
                    .text_value(match &self.load {
                        Some(load) => format!("{:.1}% ", load.load),
                        None => " - ".to_string(),
                    })]),
            )
            .max(100);

        frame.render_widget(container, block);
        frame.render_widget(frequency, left_block);
        frame.render_widget(load, right_block);
    }
}
