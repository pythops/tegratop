use anyhow::{Context, Result};
use log::error;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Padding, Row, Table},
};

use std::{
    fs::{self, File},
    io::{Read, Seek},
};

#[derive(Debug, Default)]
pub struct Memory {
    pub mem: Option<Mem>,
    pub emc: Option<EMC>,
}

#[derive(Debug)]
pub struct Mem {
    fd: File,
    pub total_ram: f64,
    pub used_ram: f64,
    pub total_swap: f64,
    pub used_swap: f64,
}

#[derive(Debug)]
pub struct EMC {
    max_frequency_fd: File,
    current_frequency: usize,
    max_frequency: usize,
}

impl EMC {
    fn new() -> Result<Self> {
        let current_frequency = fs::read_to_string("/sys/kernel/debug/clk/emc/clk_rate")
            .context("Failed to read from /sys/kernel/debug/clk/emc/clk_rate")?;

        let current_frequency = current_frequency.trim().parse::<usize>()? / 1_000_000;

        let mut max_frequency_fd = File::open("/sys/kernel/debug/clk/emc/clk_max_rate")
            .context("Failed to open /sys/kernel/debug/clk/emc/clk_max_rate")?;

        let mut buffer = String::new();
        max_frequency_fd.read_to_string(&mut buffer)?;
        let max_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

        Ok(Self {
            max_frequency_fd,
            current_frequency,
            max_frequency,
        })
    }

    fn refresh(&mut self) -> Result<()> {
        self.max_frequency_fd.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.max_frequency_fd.read_to_string(&mut buffer)?;
        self.max_frequency = buffer.trim().parse::<usize>()? / 1_000_000;

        let current_frequency = fs::read_to_string("/sys/kernel/debug/clk/emc/clk_rate")
            .context("Failed to read from /sys/kernel/debug/clk/emc/clk_rate")?;
        self.current_frequency = current_frequency.trim().parse::<usize>()? / 1_000_000;

        Ok(())
    }
}

impl Mem {
    pub fn new() -> Result<Self> {
        let mut memory_file = File::open("/proc/meminfo").context("Faile to open /proc/meminfo")?;
        let mut buffer = String::new();
        memory_file.read_to_string(&mut buffer)?;

        let mut total = 0.0;
        let mut free = 0.0;
        let mut buffered = 0.0;
        let mut cached = 0.0;
        let mut sreclaimable = 0.0;
        let mut shmem = 0.0;
        let mut total_swap = 0.0;
        let mut free_swap = 0.0;

        for line in buffer.lines() {
            let mut parts: Vec<&str> = line.split_whitespace().collect();
            parts.truncate(2);

            match parts.as_slice() {
                ["MemTotal:", v] => {
                    total = v.parse::<f64>()?;
                }
                ["MemFree:", v] => {
                    free = v.parse::<f64>()?;
                }
                ["Buffers:", v] => {
                    buffered = v.parse::<f64>()?;
                }
                ["Cached:", v] => {
                    cached = v.parse::<f64>()?;
                }
                ["Shmem:", v] => {
                    shmem = v.parse::<f64>()?;
                }
                ["SReclaimable:", v] => {
                    sreclaimable = v.parse::<f64>()?;
                }
                ["SwapTotal:", v] => {
                    total_swap = v.parse::<f64>()?;
                }
                ["SwapFree:", v] => {
                    free_swap = v.parse::<f64>()?;
                }

                _ => (),
            }
        }

        let total_used_memory = total - free;
        let cached_memory = cached + sreclaimable - shmem;

        let total_ram = (total / 1024.0).round();
        let used_ram = ((total_used_memory - (buffered + cached_memory)) / 1024.0).round();

        let total_swap = ((total_swap) / 1024.0).round();
        let used_swap = ((total_swap - free_swap) / 1024.0).round();

        Ok(Self {
            fd: memory_file,
            total_ram,
            used_ram,
            total_swap,
            used_swap,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.fd.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.fd.read_to_string(&mut buffer)?;

        let mut total = 0.0;
        let mut free = 0.0;
        let mut buffered = 0.0;
        let mut cached = 0.0;
        let mut sreclaimable = 0.0;
        let mut shmem = 0.0;
        let mut total_swap = 0.0;
        let mut free_swap = 0.0;

        for line in buffer.lines() {
            let mut parts: Vec<&str> = line.split_whitespace().collect();
            parts.truncate(2);

            match parts.as_slice() {
                ["MemTotal:", v] => {
                    total = v.parse::<f64>()?;
                }
                ["MemFree:", v] => {
                    free = v.parse::<f64>()?;
                }
                ["Buffers:", v] => {
                    buffered = v.parse::<f64>()?;
                }
                ["Cached:", v] => {
                    cached = v.parse::<f64>()?;
                }
                ["Shmem:", v] => {
                    shmem = v.parse::<f64>()?;
                }
                ["SReclaimable:", v] => {
                    sreclaimable = v.parse::<f64>()?;
                }
                ["SwapTotal:", v] => {
                    total_swap = v.parse::<f64>()?;
                }
                ["SwapFree:", v] => {
                    free_swap = v.parse::<f64>()?;
                }

                _ => (),
            }
        }

        let total_used_memory = total - free;
        let cached_memory = cached + sreclaimable - shmem;

        self.total_ram = (total / 1024.0).round();
        self.used_ram = ((total_used_memory - (buffered + cached_memory)) / 1024.0).round();

        self.total_swap = ((total_swap) / 1024.0).round();
        self.used_swap = ((total_swap - free_swap) / 1024.0).round();

        Ok(())
    }
}

impl Memory {
    pub fn new() -> Self {
        let emc = EMC::new().map_or_else(
            |e| {
                error!("{}", e);
                None
            },
            Some,
        );

        let mem = Mem::new().map_or_else(
            |e| {
                error!("{}", e);
                None
            },
            Some,
        );

        Self { mem, emc }
    }

    pub fn refresh(&mut self) {
        if let Some(mem) = &mut self.mem {
            if let Err(e) = mem.refresh() {
                error!("{}", e);
            }
        }

        if let Some(emc) = &mut self.emc {
            if let Err(e) = emc.refresh() {
                error!("{}", e);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let container = Block::default()
            .borders(Borders::ALL)
            .title("Memory")
            .padding(Padding::horizontal(1))
            .title_style(Style::new().bold());

        let inside_container = container.inner(block);

        let (ram_block, swap_block, emc_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                ])
                .split(inside_container);

            (chunks[0], chunks[1], chunks[2])
        };

        let ram = BarChart::default()
            .block(Block::default())
            .bar_width(1)
            .bar_gap(1)
            .group_gap(0)
            .direction(Direction::Horizontal)
            .data(
                BarGroup::default().bars(&[match &self.mem {
                    Some(mem) => Bar::default()
                        .label(Line::styled("RAM ", Style::default().bold()))
                        .value(mem.used_ram as u64)
                        .text_value(format!("{}MB / {}MB ", mem.used_ram, mem.total_ram)),

                    None => Bar::default()
                        .label(Line::styled("RAM ", Style::default().bold()))
                        .value(0),
                }]),
            )
            .max(match &self.mem {
                Some(mem) => mem.total_ram as u64,
                None => 0,
            });

        let swap = BarChart::default()
            .block(Block::default())
            .bar_width(1)
            .bar_gap(1)
            .group_gap(0)
            .direction(Direction::Horizontal)
            .data(
                BarGroup::default().bars(&[match &self.mem {
                    Some(mem) => Bar::default()
                        .label(Line::styled("Swap", Style::default().bold()))
                        .value(mem.used_swap as u64)
                        .text_value(format!("{}MB / {}MB ", mem.used_swap, mem.total_swap)),

                    None => Bar::default()
                        .label(Line::styled("Swap", Style::default().bold()))
                        .value(0),
                }]),
            )
            .max(match &self.mem {
                Some(mem) => mem.total_swap as u64,
                None => 0,
            });

        let emc = Table::new(
            [Row::new(vec![
                Cell::new("EMC").style(Style::default().bold()),
                Cell::new(match &self.emc {
                    Some(emc) => format!("{}MHz / {}MHz", emc.current_frequency, emc.max_frequency),
                    None => " - ".to_string(),
                }),
            ])],
            [Constraint::Length(4), Constraint::Length(20)],
        )
        .block(Block::default());

        frame.render_widget(container, block);
        frame.render_widget(ram, ram_block);
        frame.render_widget(swap, swap_block);
        frame.render_widget(emc, emc_block);
    }
}
