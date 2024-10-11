use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let cpu_block_length = (&app.cpu.cores.len() + 1) as u16;
    let thermal_block_length = app.thermal.sensors.len();
    let engines_block_length = app.engine.hws.len();
    let power_block_length = app.power.channels.len();
    let engine_block_length = app.engine.hws.len() as u16;

    let container_length = std::cmp::max(
        power_block_length,
        std::cmp::max(engines_block_length, thermal_block_length),
    ) as u16
        + 3;

    let (
        board_block,
        cpu_block,
        memory_block,
        gpu_block,
        system_block,
        fan_block,
        disk_block,
        engine_block,
        container_block,
    ) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4),
                    Constraint::Length(cpu_block_length),
                    Constraint::Length(7),                   // memory
                    Constraint::Length(3),                   // gpu
                    Constraint::Length(5),                   // system
                    Constraint::Length(4),                   // fan
                    Constraint::Length(4),                   // disk
                    Constraint::Length(engine_block_length), // network
                    Constraint::Length(container_length),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(frame.area());
        (
            chunks[0], chunks[1], chunks[2], chunks[3], chunks[4], chunks[5], chunks[6], chunks[7],
            chunks[8],
        )
    };

    let (network_block, thermal_block, power_block) = {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(container_block);
        (chunks[0], chunks[1], chunks[2])
    };

    app.cpu.render(frame, cpu_block);
    app.memory.render(frame, memory_block);
    app.network.render(frame, network_block);
    app.gpu.render(frame, gpu_block);
    app.engine.render(frame, engine_block);
    app.thermal.render(frame, thermal_block);
    app.fan.render(frame, fan_block);
    app.disk.render(frame, disk_block);
    app.power.render(frame, power_block);
    app.system.render(frame, system_block);
    app.board.render(frame, board_block);
}
