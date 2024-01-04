use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tegratop::app::{App, AppResult};
use tegratop::event::{Event, EventHandler};
use tegratop::handler::handle_key_events;
use tegratop::tracing::Tracing;
use tegratop::tui::Tui;

fn main() -> AppResult<()> {
    Tracing::init()?;

    let mut app = App::new();

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1_000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }
    tui.exit()?;
    Ok(())
}
