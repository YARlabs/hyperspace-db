use ratatui::{layout::*, widgets::*, Frame, style::{Style, Color}};

pub struct App {
    pub memory_percent: u16,
}

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    let title = Paragraph::new("HyperspaceDB :: Poincare Edition")
        .block(Block::default().borders(Borders::ALL).title("Dashboard"));
    f.render_widget(title, chunks[0]);

    // QPS and Memory Usage Gauge
    let gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(app.memory_percent);
    f.render_widget(gauge, chunks[1]);
}
