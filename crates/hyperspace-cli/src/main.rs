mod ui;
use ui::{ui, App};
use std::{error::Error, io};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hyperspace_proto::hyperspace::database_client::DatabaseClient; // Fix import path
use hyperspace_proto::hyperspace::MonitorRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Connect gRPC
    // Note: Use http scheme for tonic local connection usually
    let mut client = DatabaseClient::connect("http://[::1]:50051").await?;
    let mut stream = client.monitor(MonitorRequest {}).await?.into_inner();

    // 2. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut node_count: u64 = 0;

    // 3. UI Loop
    loop {
        terminal.draw(|f| {
             let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
                .split(f.size());

             let title = Paragraph::new("HyperspaceDB :: Poincaré Monitor")
                .block(Block::default().borders(Borders::ALL));
             f.render_widget(title, chunks[0]);

            let info = Paragraph::new(format!(
                "Nodes: {}\nStatus: Online\nMode: HNSW+SIMD", 
                node_count
            ))
            .block(Block::default().title("Stats").borders(Borders::ALL));
            f.render_widget(info, chunks[1]);
        })?;

        tokio::select! {
            msg = stream.message() => {
                match msg {
                    Ok(Some(stats)) => {
                        node_count = stats.indexed_vectors;
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            _ = async {
                 if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        if key.code == KeyCode::Char('q') { return Some(()); }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                None
            } => {
                 // Check return from async block? 
                 // Actually this poll logic is blocking, might be choppy.
                 // Better to use a separate thread for input ideally, but for demo:
                 // The poll above returns immediately. 
            }
        }
        
        // Manual check for exit since 'select' branch return isn't propagating break easily
        if crossterm::event::poll(std::time::Duration::from_millis(0))? {
             if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
    }

    // Restore
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
}
