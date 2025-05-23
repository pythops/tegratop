use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Row, Table},
};
use std::fs;
use std::path::Path;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;

#[derive(EnumIter, Display, Debug, Default, PartialEq)]
pub enum HWName {
    #[default]
    APE,
    DLA,
    CVNAS,
    MSENC,
    NVENC,
    NVDEC,
    NVJPG,
    PVA,
    SE,
    VIC,
}

#[derive(Debug, Default)]
pub struct Engine {
    pub hws: Vec<HW>,
}

#[derive(Debug, Default, Display)]
pub enum HWState {
    #[default]
    Idle,
    Running,
}

#[derive(Debug, Default)]
pub struct HW {
    pub name: HWName,
    pub state: HWState,
    pub frequency: f64,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh(&mut self) {
        let mut hws: Vec<HW> = Vec::new();

        let stats_path = Path::new("/sys/kernel/debug/clk");

        for hw_name in HWName::iter() {
            let hw_stats_path = stats_path.join(hw_name.to_string().to_lowercase());

            if !hw_stats_path.exists() {
                continue;
            }

            let state = match fs::read_to_string(hw_stats_path.join("clk_enable_count")) {
                Ok(v) => match v.trim() {
                    "1" => HWState::Running,
                    _ => HWState::Idle,
                },
                Err(_) => continue,
            };

            let frequency = match fs::read_to_string(hw_stats_path.join("clk_rate")) {
                Ok(v) => v.trim().parse::<f64>().unwrap() / 1_000_000.0,
                Err(_) => continue,
            };

            let hw = HW {
                name: hw_name,
                state,
                frequency,
            };

            hws.push(hw);
        }

        self.hws = hws;
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let container = Block::default()
            .borders(Borders::ALL)
            .title("Engines")
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

        let (left_engines, right_engines) = self.hws.split_at(self.hws.len() / 2);

        let left_rows: Vec<Row> = left_engines
            .iter()
            .map(|hw| {
                Row::new(vec![hw.name.to_string(), hw.state.to_string(), {
                    match hw.state {
                        HWState::Idle => "-".to_string(),
                        HWState::Running => format!("{:.1} MHz", hw.frequency),
                    }
                }])
            })
            .collect();

        let right_rows: Vec<Row> = right_engines
            .iter()
            .map(|hw| {
                Row::new(vec![hw.name.to_string(), hw.state.to_string(), {
                    match hw.state {
                        HWState::Idle => "-".to_string(),
                        HWState::Running => format!("{:.1} MHz", hw.frequency),
                    }
                }])
            })
            .collect();

        let widths = [
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(10),
        ];

        let left_table = Table::new(left_rows, widths)
            .header(Row::new(vec!["Name", "State", "Frequency"]).style(Style::new().bold()))
            .block(Block::default());

        let right_table = Table::new(right_rows, widths)
            .header(Row::new(vec!["Name", "State", "Frequency"]).style(Style::new().bold()))
            .block(Block::default());

        frame.render_widget(container, block);
        frame.render_widget(left_table, left_block);
        frame.render_widget(right_table, right_block);
    }
}
