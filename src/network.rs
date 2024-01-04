use std::ffi::CStr;
use std::net::Ipv4Addr;

use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Row, Table},
    Frame,
};

#[derive(Debug, Default)]
pub struct Network {
    pub interfaces: Vec<Interface>,
}

#[derive(Debug, Default)]
pub struct Interface {
    name: String,
    ipv4: Option<Ipv4Addr>,
}

impl Network {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh(&mut self) {
        let mut interfaces: Vec<Interface> = vec![];
        unsafe {
            let mut ifap: *mut libc::ifaddrs = std::ptr::null_mut();

            if libc::getifaddrs(&mut ifap) == 0 {
                let mut ifa = ifap;

                while !ifa.is_null() {
                    let ifa_name = (*ifa).ifa_name;
                    let ifa_addr = (*ifa).ifa_addr;

                    if !ifa_name.is_null() && !ifa_addr.is_null() {
                        let cstr_name = CStr::from_ptr(ifa_name);
                        let interface_name = cstr_name.to_str().unwrap();

                        if !interfaces.iter().any(|i| i.name == interface_name) {
                            interfaces.push(Interface {
                                name: interface_name.to_string(),
                                ipv4: None,
                            });
                        }

                        if (*ifa_addr).sa_family == libc::AF_INET as u16 {
                            let sockaddr_in = ifa_addr as *const libc::sockaddr_in;
                            let ip_addr = (*sockaddr_in).sin_addr;

                            let ipv4_addr = Ipv4Addr::from(ip_addr.s_addr.to_be());

                            if let Some(index) =
                                interfaces.iter().position(|i| i.name == interface_name)
                            {
                                interfaces[index].ipv4 = Some(ipv4_addr);
                            }
                        }
                    }

                    ifa = (*ifa).ifa_next;
                }

                libc::freeifaddrs(ifap);
            }
        }

        interfaces.retain(|interface| interface.name != "lo");
        self.interfaces = interfaces;
    }

    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows: Vec<Row> = self
            .interfaces
            .iter()
            .map(|interface| {
                let ip = match interface.ipv4 {
                    Some(ip) => ip.to_string(),
                    None => "-".to_string(),
                };
                Row::new(vec![interface.name.to_owned(), ip])
            })
            .collect();

        let widths = [Constraint::Length(11), Constraint::Length(18)];

        let network = Table::new(rows, widths)
            .header(Row::new(vec!["Interface", "IPv4"]).style(Style::new().bold()))
            .block(
                Block::default()
                    .title("Network")
                    .title_style(Style::new().bold())
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL),
            );

        frame.render_widget(network, block);
    }
}
