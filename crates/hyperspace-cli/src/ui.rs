use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::{Block, Borders, Gauge, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::app::{App, CurrentTab};

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
    let titles = vec!["Overview [1]", "Storage [2]", "Admin [3]"];
    let tabs = Tabs::new(titles)
        .select(app.current_tab as usize)
        .block(Block::default().title("HyperspaceDB Mission Control").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Content
    match app.current_tab {
        CurrentTab::Overview => draw_overview(f, app, chunks[1]),
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
    f.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)), chunks[2]);
}

fn draw_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Compression
            Constraint::Length(3), // Stats Row 1
            Constraint::Length(3), // Config Row
            Constraint::Min(1),    // Queue status
        ])
        .split(area);

    // 1. Compression Ratio
    let ratio = app.stats.compression_ratio;
    let raw = app.stats.raw_data_size_mb;
    let actual = app.stats.actual_storage_mb;
    let saved = raw - actual;
    
    let label = format!("Compression: {:.2}x (Saved {:.2} MB)", ratio, saved);
    let saved_percent = if raw > 0.0 { (raw - actual) / raw } else { 0.0 };
    
    let gauge = Gauge::default()
        .block(Block::default().title(label).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(saved_percent.clamp(0.0, 1.0))
        .label(format!("{:.1}% Space Saved", saved_percent * 100.0));

    f.render_widget(gauge, chunks[0]);
    
    // 2. Stats Row
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)])
        .split(chunks[1]);
        
    let count_text = Paragraph::new(format!("{}", app.stats.indexed_vectors))
        .block(Block::default().title("Indexed Vectors").borders(Borders::ALL));
    f.render_widget(count_text, stats_layout[0]);
    
    let active_text = Paragraph::new(format!("{}", app.stats.active_segments))
        .block(Block::default().title("Segments").borders(Borders::ALL));
    f.render_widget(active_text, stats_layout[1]);
    
    let soft_text = Paragraph::new(format!("{}", app.stats.soft_deleted))
        .block(Block::default().title("Deleted").borders(Borders::ALL).style(Style::default().fg(Color::Red)));
    f.render_widget(soft_text, stats_layout[2]);
    
    // 3. Config Row (Dynamic Tuning Parameters)
    let config_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)])
        .split(chunks[2]);
        
    let ef_s = Paragraph::new(format!("{}", app.stats.ef_search))
        .block(Block::default().title("ef_search").borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
    f.render_widget(ef_s, config_layout[0]);
    
    let ef_c = Paragraph::new(format!("{}", app.stats.ef_construction))
        .block(Block::default().title("ef_construction").borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
    f.render_widget(ef_c, config_layout[1]);
    
    // Queue Size (Pending Indexing)
    let queue_color = if app.stats.index_queue_size > 100 { Color::Yellow } else { Color::Green };
    let queue_text = Paragraph::new(format!("{}", app.stats.index_queue_size))
        .block(Block::default().title("Index Queue").borders(Borders::ALL).style(Style::default().fg(queue_color)));
    f.render_widget(queue_text, config_layout[2]);
}

fn draw_storage(f: &mut Frame, app: &App, area: Rect) {
     let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(area);
        
     let info = format!(
         "Storage Mode: ScalarI8 (8-bit Quantization)\n\
          Raw Size (f64): {:.2} MB\n\
          Actual Size:    {:.2} MB\n\
          \n\
          Segments: {}\n\
          (Vacuum feature coming in v1.6)",
          app.stats.raw_data_size_mb,
          app.stats.actual_storage_mb,
          app.stats.active_segments
     );
     
     let p = Paragraph::new(info)
        .block(Block::default().title("Storage Inspector").borders(Borders::ALL));
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
    let p_controls = Paragraph::new(controls)
        .block(Block::default().title("Controls").borders(Borders::ALL));
    f.render_widget(p_controls, chunks[0]);
    
    // Logs
    let logs: Vec<Line> = app.logs.iter().rev().map(|s| Line::from(s.as_str())).collect();
    let p_logs = Paragraph::new(logs)
        .block(Block::default().title("System Logs").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(p_logs, chunks[1]);
}
