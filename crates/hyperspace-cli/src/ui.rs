use crate::app::{App, CurrentTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(f.size());

    // Tabs
    let titles = vec![
        "Overview [1]",
        "Collections [2]",
        "Storage [3]",
        "Admin [4]",
    ];
    let tabs = Tabs::new(titles)
        .select(app.current_tab as usize)
        .block(
            Block::default()
                .title("HyperspaceDB Mission Control")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Content
    match app.current_tab {
        CurrentTab::Overview => draw_overview(f, app, chunks[1]),
        CurrentTab::Collections => draw_collections(f, app, chunks[1]),
        CurrentTab::Storage => draw_storage(f, app, chunks[1]),
        CurrentTab::Admin => draw_admin(f, app, chunks[1]),
    }

    // Footer
    let footer = Line::from(vec![
        Span::raw("Press "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to switch tabs, "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to quit."),
    ]);
    f.render_widget(
        Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );
}

fn draw_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Stats Row 1
            Constraint::Length(3), // Stats Row 2
            Constraint::Min(1),    // Empty
        ])
        .split(area);

    // 1. Stats Row
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let count_text = Paragraph::new(format!("{}", app.stats.total_vectors)).block(
        Block::default()
            .title("Total Vectors")
            .borders(Borders::ALL),
    );
    f.render_widget(count_text, stats_layout[0]);

    let cols_text = Paragraph::new(format!("{}", app.stats.total_collections))
        .block(Block::default().title("Collections").borders(Borders::ALL));
    f.render_widget(cols_text, stats_layout[1]);

    // 2. Stats Row 2
    let stats_layout2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let mem_text = Paragraph::new(format!("{:.2} MB", app.stats.total_memory_mb))
        .block(Block::default().title("Memory Usage").borders(Borders::ALL));
    f.render_widget(mem_text, stats_layout2[0]);

    let qps_text = Paragraph::new(format!("{:.2}", app.stats.qps))
        .block(Block::default().title("QPS").borders(Borders::ALL));
    f.render_widget(qps_text, stats_layout2[1]);
}

fn draw_storage(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(area);

    let info = format!(
        "Storage Mode: Multi-Collection\n\
          Total Vectors: {}\n\
          Total Memory: {:.2} MB\n\
          (Detailed storage stats moved to Dashboard)",
        app.stats.total_vectors, app.stats.total_memory_mb
    );

    let p = Paragraph::new(info).block(
        Block::default()
            .title("Storage Inspector")
            .borders(Borders::ALL),
    );
    f.render_widget(p, chunks[0]);
}

fn draw_admin(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Controls
            Constraint::Min(1),    // Logs
        ])
        .split(area);

    let controls = "Actions: [S]napshot  [V]acuum";
    let p_controls =
        Paragraph::new(controls).block(Block::default().title("Controls").borders(Borders::ALL));
    f.render_widget(p_controls, chunks[0]);

    // Logs
    let logs: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .map(|s| Line::from(s.as_str()))
        .collect();
    let p_logs = Paragraph::new(logs)
        .block(Block::default().title("System Logs").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(p_logs, chunks[1]);
}

fn draw_collections(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(area);

    let items: Vec<Line> = app
        .collections_list
        .iter()
        .map(|c| Line::from(Span::raw(c)))
        .collect();

    let list = Paragraph::new(items).block(
        Block::default()
            .title("Active Collections")
            .borders(Borders::ALL),
    );

    f.render_widget(list, chunks[0]);
}
