use anyhow::Context;
use anyhow::Result;

use log::error;
use std::fs::File;
use std::io::Read;
use std::io::Seek;

use std::{ffi::CString, fs};

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Row, Table},
};

#[derive(Debug, Default)]
pub struct Disk {
    device_name: Option<String>,
    space: Option<DiskSpace>,
    io: Option<DiskIO>,
}

#[derive(Debug)]
pub struct DiskIO {
    file: File,
    stats: DiskIOStats,
    total_reads: f64,
    total_writes: f64,
}

#[derive(Debug, Default, Clone)]
pub struct DiskIOStats {
    reads: f64,
    writes: f64,
}

#[derive(Debug, Default)]
pub struct DiskSpace {
    total: f64,
    available: f64,
}

impl DiskIO {
    fn stats(device_name: &str) -> Result<Self> {
        let mut file = File::open("/proc/diskstats").context("Failed to open /proc/diskstats")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let mut stats: DiskIOStats = DiskIOStats::default();

        for line in buffer.lines() {
            let fields = line.split_whitespace().collect::<Vec<&str>>();
            if fields[2] == device_name {
                stats = DiskIOStats {
                    reads: fields[5].parse::<f64>()? * 512.0 / 1024.0 / 1024.0,
                    writes: fields[9].parse::<f64>()? * 512.0 / 1024.0 / 1024.0,
                };
                break;
            }
        }

        Ok(Self {
            file,
            stats,
            total_reads: 0.0,
            total_writes: 0.0,
        })
    }

    fn refresh(&mut self, device_name: &str) -> Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;

        let mut stats: DiskIOStats = self.stats.clone();

        for line in buffer.lines() {
            let fields = line.split_whitespace().collect::<Vec<&str>>();

            if fields[2] == device_name {
                stats = DiskIOStats {
                    reads: fields[5].parse::<f64>()? * 512.0 / 1024.0 / 1024.0,
                    writes: fields[9].parse::<f64>()? * 512.0 / 1024.0 / 1024.0,
                };
                break;
            }
        }

        let total_reads = stats.reads - self.stats.reads;
        let total_writes = stats.writes - self.stats.writes;

        self.stats = stats;
        self.total_reads = total_reads;
        self.total_writes = total_writes;

        Ok(())
    }
}

impl DiskSpace {
    fn stats() -> Result<Option<Self>> {
        let path = CString::new("/").unwrap();

        let mut statvfs: libc::statvfs = unsafe { std::mem::zeroed() };

        if unsafe { libc::statvfs(path.as_ptr(), &mut statvfs) } == 0 {
            let total =
                statvfs.f_blocks as f64 * statvfs.f_frsize as f64 / (1024.0 * 1024.0 * 1024.0);
            let available =
                statvfs.f_bavail as f64 * statvfs.f_frsize as f64 / (1024.0 * 1024.0 * 1024.0);

            return Ok(Some(Self { total, available }));
        }

        Ok(None)
    }
}

impl Disk {
    pub fn new() -> Self {
        match Disk::root_device_name() {
            Ok(Some(device_name)) => {
                let space = match DiskSpace::stats() {
                    Ok(space) => space,
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                };

                let io = DiskIO::stats(&device_name).map_or_else(
                    |e| {
                        error!("{}", e);
                        None
                    },
                    Some,
                );

                Self {
                    device_name: Some(device_name),
                    space,
                    io,
                }
            }
            Ok(None) => {
                error!("Can not find the root device name");
                Self {
                    device_name: None,
                    space: None,
                    io: None,
                }
            }
            Err(e) => {
                error!("{}", e);

                Self {
                    device_name: None,
                    space: None,
                    io: None,
                }
            }
        }
    }

    pub fn root_device_name() -> Result<Option<String>> {
        let buffer =
            fs::read_to_string("/proc/mounts").context("Failed to read from /proc/mounts")?;
        for line in buffer.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts[1] == "/" {
                return Ok(Some(parts[0].to_string()));
            }
        }
        Ok(None)
    }

    pub fn refresh(&mut self) {
        if let Ok(stats) = DiskSpace::stats() {
            self.space = stats;
        }

        if let Some(device_name) = &self.device_name {
            if let Some(io) = &mut self.io {
                io.refresh(device_name).ok();
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let mut rows: Vec<String> = Vec::new();
        let mut space_rows: Vec<String> = Vec::new();
        let mut io_rows: Vec<String> = Vec::new();

        if let Some(space) = &self.space {
            space_rows = vec![
                format!("{:.1}GB", space.total),
                format!("{:.1}GB", space.available),
                format!("{:.1}GB", space.total - space.available),
            ];
        }

        if let Some(io) = &self.io {
            io_rows = vec![
                format!("{:.1}MB/s", io.total_reads),
                format!("{:.1}MB/s", io.total_writes),
            ];
        }

        rows.append(&mut space_rows);
        rows.append(&mut io_rows);

        let rows: [Row; 1] = [Row::new(rows)];

        let widths = [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let disk = Table::new(rows, widths)
            .header(
                Row::new(vec!["Total", "Available", "Used", "Reads", "Writes"])
                    .style(Style::new().bold()),
            )
            .block(
                Block::default()
                    .title("Disk")
                    .title_style(Style::new().bold())
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL),
            );

        frame.render_widget(disk, block);
    }
}
