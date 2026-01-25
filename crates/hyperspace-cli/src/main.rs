mod app;
mod ui;

use std::error::Error;
use std::io;
use std::time::Duration;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hyperspace_proto::hyperspace::database_client::DatabaseClient;
use hyperspace_proto::hyperspace::{MonitorRequest, Empty, SystemStats};
use app::{App, CurrentTab};
use ui::ui;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Network
    let mut client = DatabaseClient::connect("http://[::1]:50051").await?;
    
    // Start Monitor Stream
    let mut monitor_stream = client.monitor(MonitorRequest {}).await?.into_inner();

    // Channel for Async -> Sync UI
    let (tx, mut rx) = tokio::sync::mpsc::channel::<SystemStats>(10);

    // Background Task: Network Listener
    tokio::spawn(async move {
        while let Ok(Some(stats)) = monitor_stream.message().await {
            if tx.send(stats).await.is_err() {
                break;
            }
        }
    });

    // 2. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Create App State
    let mut app = App::new();

    // 4. Run UI Loop
    let res = run_app(&mut terminal, &mut app, &mut rx, client.clone()).await;

    // 5. Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: &mut tokio::sync::mpsc::Receiver<SystemStats>,
    mut client: DatabaseClient<Channel>, 
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Process network updates (Non-blocking)
        if let Ok(stats) = rx.try_recv() {
            app.stats = stats;
        }

        // Process Input (Blocking with timeout)
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Tab => app.next_tab(), 
                    KeyCode::Char('1') => app.current_tab = CurrentTab::Overview,
                    KeyCode::Char('2') => app.current_tab = CurrentTab::Storage,
                    KeyCode::Char('3') => app.current_tab = CurrentTab::Admin,
                    KeyCode::Char('s') => {
                        let mut c = client.clone();
                        tokio::spawn(async move {
                            let _ = c.trigger_snapshot(Empty{}).await;
                        });
                        app.logs.push("Snapshot triggered...".to_string());
                    },
                    KeyCode::Char('v') => {
                        let mut c = client.clone();
                        tokio::spawn(async move {
                            let _ = c.trigger_vacuum(Empty{}).await;
                        });
                        app.logs.push("Vacuum triggered...".to_string());
                    },
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
