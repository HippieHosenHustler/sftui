use std::{
    io::{self, Stdout},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, ListState, Paragraph},
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

enum InputEventResult {
    Continue,
    Quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("cannot run in raw mode");

    let (tx, rx) = mpsc::channel();

    start_polling_thread(tx);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut metadata_list_state = ListState::default();
    metadata_list_state.select(Some(0));

    loop {
        render_ui(&mut terminal)?;

        let input_result = handle_input(&rx)?;
        match input_result {
            InputEventResult::Quit => {
                disable_raw_mode()?;
                terminal.show_cursor()?;
                break;
            }
            InputEventResult::Continue => {}
        }
    }

    Ok(())
}

fn handle_input(
    rx: &Receiver<Event<KeyEvent>>,
) -> Result<InputEventResult, Box<dyn std::error::Error>> {
    match rx.recv()? {
        Event::Input(event) => match event.code {
            KeyCode::Char('q') => Ok(InputEventResult::Quit),
            _ => Ok(InputEventResult::Continue),
        },
        Event::Tick => Ok(InputEventResult::Continue),
    }
}

fn render_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|rect| {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(2)].as_ref())
            .split(size);

        let hello_world = Paragraph::new("Hello World!")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("SFUI")
                    .border_type(BorderType::Rounded),
            );
        rect.render_widget(hello_world, chunks[0]);
    })?;
    Ok(())
}

fn start_polling_thread(tx: Sender<Event<KeyEvent>>) {
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Polling is broken") {
                if let CEvent::Key(key) = event::read().expect("Cannot read events") {
                    tx.send(Event::Input(key)).expect("Cannot send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });
}
