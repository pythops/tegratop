use std::error;

use crate::{
    board::Board, cpu::CPU, disk::Disk, engine::Engine, fan::Fan, gpu::GPU, memory::Memory,
    network::Network, power::Power, system::System, thermal::Thermal,
};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub board: Board,
    pub cpu: CPU,
    pub disk: Disk,
    pub engine: Engine,
    pub fan: Fan,
    pub gpu: GPU,
    pub memory: Memory,
    pub network: Network,
    pub power: Power,
    pub system: System,
    pub thermal: Thermal,
    pub running: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            board: Board::new(),
            cpu: CPU::new(),
            disk: Disk::new(),
            engine: Engine::new(),
            fan: Fan::new(),
            gpu: GPU::new(),
            memory: Memory::new(),
            network: Network::new(),
            power: Power::new(),
            system: System::new(),
            thermal: Thermal::new(),
            running: true,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        self.cpu.refresh();
        self.disk.refresh();
        self.engine.refresh();
        self.fan.refresh();
        self.gpu.refresh();
        self.memory.refresh();
        self.network.refresh();
        self.power.refresh();
        self.system.refresh();
        self.thermal.refresh();
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
