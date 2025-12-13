use log::error;
use std::{fs, path::Path};

use anyhow::{Context, Result};

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table},
};
use regex::Regex;

#[derive(Debug, Default)]
pub struct Board {
    name: Option<String>,
    l4t: Option<String>,
    bios: Option<String>,
}

impl Board {
    fn name() -> Result<String> {
        let path = Path::new("/sys/firmware/devicetree/base/model");
        let name =
            fs::read_to_string(path).context(format!("Failed to read from {}", path.display()))?;

        Ok(name)
    }

    fn l4t() -> Result<Option<String>> {
        let path = Path::new("/etc/nv_tegra_release");
        let l4t_buffer =
            fs::read_to_string(path).context(format!("Failed to read from {}", path.display()))?;

        let re = Regex::new(r"R(\d+) \(release\), REVISION: (\d+\.\d+)")?;

        if let Some(captures) = re.captures(l4t_buffer.as_str()) {
            let release = captures.get(1).map_or("", |m| m.as_str());
            let revision = captures.get(2).map_or("", |m| m.as_str());
            return Ok(Some(format!("{}.{}", release, revision)));
        };

        Ok(None)
    }

    fn bios() -> Result<Option<String>> {
        let path = Path::new("/sys/class/dmi/id/bios_version");
        let buffer =
            fs::read_to_string(path).context(format!("Failed to read from {}", path.display()))?;
        let parts = buffer.split('-').next().map(|s| s.to_string());
        Ok(parts)
    }

    pub fn new() -> Self {
        let name = Board::name().map_or_else(
            |e| {
                error!("{}", e);
                None
            },
            Some,
        );

        let l4t = match Board::l4t() {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        let bios = match Board::bios() {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                None
            }
        };

        Self { name, l4t, bios }
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows: [Row; 3] = [
            Row::new(vec![
                Cell::new("Name").style(Style::default().bold()),
                Cell::new(match &self.name {
                    Some(name) => name,
                    None => " - ",
                }),
            ]),
            Row::new(vec![
                Cell::new("L4t").style(Style::default().bold()),
                Cell::new(match &self.l4t {
                    Some(l4t) => l4t,
                    None => " - ",
                }),
            ]),
            Row::new(vec![
                Cell::new("Firmware").style(Style::default().bold()),
                Cell::new(match &self.bios {
                    Some(v) => v,
                    None => " - ",
                }),
            ]),
        ];

        let widths = [Constraint::Length(10), Constraint::Fill(1)];

        let board = Table::new(rows, widths).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Board")
                .padding(Padding::horizontal(1))
                .title_style(Style::new().bold()),
        );
        frame.render_widget(board, block);
    }
}
