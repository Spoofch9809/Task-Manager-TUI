use std::{ io, time::Duration};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{ Block, Borders, Tabs},
    layout::{ Layout, Constraint, Direction},
    prelude::{ Style, Color},
    symbols::DOT,
    Terminal, 
    text::Line,
};
use crossterm::{
    event::{ self, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use sysinfo::{ System, SystemExt};
mod lib;
use lib::{render_cpu_info, render_battery_info,render_disk_info,render_memory_info,render_network_info,render_process_info,render_temperature_info};


#[derive(PartialEq)]
pub enum InfoCategory {
    CPU,
    Memory,
    Network,
    Process,
    Disk,
    Temperature,
    Battery,
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // Initialize running flag
    let mut running = true;
    let categories = vec![
    InfoCategory::CPU,
    InfoCategory::Memory,
    InfoCategory::Network,
    InfoCategory::Process,
    InfoCategory::Disk,
    InfoCategory::Temperature,
    InfoCategory::Battery,
];

    let titles = ["CPU", "Memory", "Network", "Process", "Disk", "Temp", "Battery"];
    let mut selected_category_index = 0; // Initialize with the first category

    loop {
        let mut sys = System::new_all();
        sys.refresh_all();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(80),
                        Constraint::Percentage(0),
                    ].as_ref()
                )
                .split(f.size());

            let titles = ["CPU", "Memory", "Network", "Process", "Disk", "Temp", "Battery"]
                .iter()
                .cloned()
                .map(Line::from)
                .collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().title("'' Menu ''- < , > to navigate ").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(DOT)
                .select(selected_category_index);

            f.render_widget(tabs, chunks[0]);

            // Render CPU information in the second block
            match categories[selected_category_index] {
                InfoCategory::CPU => {
                    render_cpu_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Memory => {
                    render_memory_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Network => {
                    render_network_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Process => {
                    render_process_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Disk => {
                    render_disk_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Temperature => {
                    render_temperature_info(&mut sys, f, chunks[1]);
                }
                InfoCategory::Battery => {
                    render_battery_info(&mut sys, f, chunks[1]);
                }
            }
        })?;

        // Handle keyboard events
        #[allow(unused_variables)]
        if event::poll(Duration::from_millis(1500)).unwrap() {
            if let Event::Key(KeyEvent { 
                code, 
                modifiers, 
                state, 
                kind 
            }) = event::read().unwrap() 
            {
                if true
                {
                    match code {
                        KeyCode::Char('q') => {
                            running = false;
                        }
                        KeyCode::Left => {
                            selected_category_index = (selected_category_index + titles.len() - 1) % titles.len();
                        }
                        KeyCode::Right => {
                            selected_category_index = (selected_category_index + 1) % titles.len();
                        }
                        _ => {}
                    }
                }
            }
        }

        // Exit the loop if running is false
        if !running {
            break;
        }
        }

    // Cleanup and exit
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    Ok(())
}
