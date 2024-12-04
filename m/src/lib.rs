use std::time::Duration;
use ratatui::{
    backend::Backend,
    widgets::{ Block, Borders, Paragraph, Gauge, Clear},
    layout::{ Layout, Constraint, Direction},
    prelude::{ Style, Rect, Color, Span},
    text::Line,
    Frame,
};
use sysinfo::{ NetworkExt, CpuExt, ProcessExt, System, SystemExt, ComponentExt, DiskExt};
use battery::units::time::second;
use std::collections::BTreeMap;
use humantime;
// --------------------------------------------------------------
pub fn render_cpu_info<B: Backend>(sys: &mut System, f: &mut Frame<B>, area: Rect) {
    sys.refresh_cpu();
    let mut cpu_info_text = String::new();
    let mut chart = vec![];

    for (idx, cpu) in sys.cpus().iter().enumerate() {
        let cpu_usage = cpu.cpu_usage();
        cpu_info_text.push_str(&format!("CPU {}: {:.2}%\n", idx, cpu_usage));

        // Define Colors
        let low_cpu_color = Color::White;
        let medium_cpu_color = Color::LightYellow;
        let high_cpu_color = Color::LightRed;
        let extremely_high_cpu_color = Color::Red;

        let color = if cpu_usage < 40.0 {
            low_cpu_color
        } else if cpu_usage < 60.0 {
            medium_cpu_color
        } else if cpu_usage < 80.0 {
            high_cpu_color
        } else {
            extremely_high_cpu_color
        };
        let cpu_value = cpu_usage as usize;
        let bar = "█".repeat(cpu_value);
        let empty = "░".repeat(100 - cpu_value);
        let chart_str = format!("[{}{}]", bar, empty);

        let chart_line = Line::from(vec![Span::styled(chart_str, Style::default().fg(color))]);
            chart.push(chart_line);
    }
    let cpu_info_widget = Paragraph::new(cpu_info_text)
        .block(Block::default().title("'' CPU Usage ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    let cpu_chart_widget = Paragraph::new(chart)
        .block(Block::default().title("'' CPU Chart ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    let cpu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);
    f.render_widget(cpu_info_widget, cpu_layout[0]);
    f.render_widget(cpu_chart_widget, cpu_layout[1]);
}
// --------------------------------------------------------------
pub fn render_memory_info<B: Backend>(sys: &mut System, f: &mut Frame<B>, area: Rect) {
    sys.refresh_memory();
    let total_memory = sys.total_memory() as f64;
    let used_memory = (sys.used_memory() as f64) / 1024.0 / 1024.0 / 1024.0; // Convert KB to GB
    let free_memory = (sys.free_memory() as f64) / 1024.0 / 1024.0 / 1024.0; // Convert KB to GB

    let memory_info_text = format!(
        "Total Memory: {:.2} GB\nUsed Memory: {:.2} GB\nFree Memory: {:.2} GB",
        total_memory / (1024.0 * 1024.0 * 1024.0),
        used_memory,
        free_memory
    );

    let gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .ratio(sys.used_memory() as f64 / sys.total_memory() as f64);
    let memory_info_widget = Paragraph::new(memory_info_text)
        .block(Block::default().title("Memory Info").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(6)].as_ref())
        .split(area);

    f.render_widget(memory_info_widget, memory_layout[0]);
    f.render_widget(gauge, memory_layout[1]);
}
// --------------------------------------------------------------
pub fn render_network_info<B: Backend>(sys: &mut System, f: &mut Frame<B>, area: Rect) {  
    sys.refresh_networks();
    // Use BTree to sort out the interface_name
    let mut btree = BTreeMap::new();

    for (interface_name, data) in sys.networks() {
        btree.insert(interface_name, data);
    }
    let mut network_info_text = String::new();
    for (interface_name, data) in btree{
        network_info_text.push_str(&format!("Interface: {}\n", interface_name));
        network_info_text.push_str(&format!("  Received: {} bytes\n", data.received()));
        network_info_text.push_str(&format!("  Transmitted: {} bytes\n", data.transmitted()));
    } 
    // Clear the area before rendering the updated content
    f.render_widget(Clear, area);

    // Create a Scroll widget to wrap the network information
    let network_info_widget = Paragraph::new(network_info_text)
        .block(Block::default().title("'' Network Info''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(network_info_widget, area);
}
// --------------------------------------------------------------
pub fn render_process_info<B: Backend>(sys: &mut System, f: &mut Frame<B>, area: Rect) {
    sys.refresh_processes();
    let mut process_info_text = String::new();

    // Use BTree to sort out the interface_name
    let mut btree = BTreeMap::new();

    for (pid, process) in sys.processes() {
        btree.insert(pid, process);
    }

    for (pid, process) in btree {
        process_info_text.push_str(&format!("PID: {}\n", pid));
        process_info_text.push_str(&format!("  Name: {}\n", process.name()));
        process_info_text.push_str(&format!("  Status: {:?}\n", process.status()));
        process_info_text.push_str(&format!("  CPU Usage: {:.2}%\n", process.cpu_usage()));
        let memory_gb = process.memory() as f64 / (1024.0 * 1024.0); // Convert KB to GB
        process_info_text.push_str(&format!("  Memory Usage: {:.2} GB\n", memory_gb));
    }

    // Clear the area before rendering the updated content
    f.render_widget(Clear, area);

    let process_info_widget = Paragraph::new(process_info_text)
        .block(Block::default().title("'' Process Info ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(process_info_widget, area);
}
// --------------------------------------------------------------
pub fn render_disk_info<B: Backend>(sys: &mut System, f: &mut Frame<B>, area: Rect) {
    sys.refresh_disks();
    let mut disk_info_text = String::new();
    let disk_info = sys.disks();

    // Function to convert bytes to gigabytes
    fn bytes_to_gb(bytes: u64) -> f64 {
        let bytes_per_gb = 1_073_741_824; // 1 GB = 1,073,741,824 bytes
        let gb = bytes as f64 / bytes_per_gb as f64;
        gb
    }

    for disk in disk_info {
        disk_info_text.push_str(&format!("Disk({:?})\n", disk.name()));
        disk_info_text.push_str(&format!("  [FS: {:?}]\n", disk.file_system().iter().map(|c| *c as char).collect::<Vec<_>>()));
        disk_info_text.push_str(&format!("  [Type: {:?}]\n", disk.kind()));
        disk_info_text.push_str(&format!("  [removeable: {}\n", if disk.is_removable() { "yes" } else { "no" }));

        let mounted_on_gb = bytes_to_gb(disk.available_space());
        let total_gb = bytes_to_gb(disk.total_space());
        
        disk_info_text.push_str(&format!("  mounted on {:?}: {:.2}/{:.2} GB]\n", disk.mount_point(), mounted_on_gb, total_gb));
    }
    let disk_info_widget = Paragraph::new(disk_info_text)
        .block(Block::default().title("'' Disk Info ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(disk_info_widget, area)
}
// --------------------------------------------------------------
pub fn render_temperature_info<B: Backend>(sys: &System, f: &mut Frame<B>, area: Rect) {
    let temp_comp = sys.components();
    let mut all_temp = vec![];
    let mut chart = vec![];

    for component in temp_comp {
            let temperature = component.temperature();
            let temp_str = format!("{:<20}: {:.2} °C", component.label(), temperature);
            all_temp.push(Line::from(vec![Span::raw(temp_str)]));

            // Define temperature ranges and their corresponding colors
            let low_temp_color = Color::White;
            let medium_temp_color = Color::LightYellow;
            let high_temp_color = Color::LightRed;
            let extremely_high_temp_color = Color::Red;

            let color = if temperature < 40.0 {
                low_temp_color
            } else if temperature < 60.0 {
                medium_temp_color
            } else if temperature < 80.0 {
                high_temp_color
            } else {
                extremely_high_temp_color
            };

            // Create a colored horizontal bar chart based on the temperature level
            let temp_value = temperature as usize;
            let bar = "█".repeat(temp_value);
            let empty = "░".repeat(100 - temp_value);
            let chart_str = format!("[{}{}]", bar, empty);

            let chart_line = Line::from(vec![Span::styled(chart_str, Style::default().fg(color))]);
            chart.push(chart_line);
        }

    let temperature_info_widget = Paragraph::new(all_temp)
        .block(Block::default().title("'' Temperature Info ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    let temperature_chart_widget = Paragraph::new(chart)
        .block(Block::default().title("'' Temperature Chart ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    let temperature_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(6)].as_ref())
        .split(area);
    f.render_widget(temperature_info_widget, temperature_layout[0]);
    f.render_widget(temperature_chart_widget, temperature_layout[1]);
}
// --------------------------------------------------------------
pub fn render_battery_info<B: Backend>(_sys: &mut System, f: &mut Frame<B>, area: Rect) {
    let manager = battery::Manager::new().expect("Failed to create battery manager");
    let mut battery_info_text = String::new();
    let mut battery_data = Vec::new(); // Vector to store battery percentage data

    for (idx, maybe_battery) in manager.batteries().expect("Failed to get batteries").enumerate() {
        match maybe_battery {
            Ok(battery) => {
                // convert to percentage
                let battery_percentage = battery.state_of_charge() * 100.0;

                // Check if battery model is null
                let model_info = match battery.model() {
                    Some(model) => model,
                    None => "Unknown Model",
                };

                // Check if time to full charge is null
                let time_to_full_info = match battery.time_to_full() {
                    Some(duration) => humantime::format_duration(Duration::from_secs(duration.get::<second>() as u64)).to_string(),
                    None => "Unknown".to_string(),
                };
        
                battery_info_text.push_str(&format!("Battery #{}\n", idx));
                battery_info_text.push_str(&format!("  Vendor: {:?}\n", battery.vendor()));
                battery_info_text.push_str(&format!("  Model: {:?}\n", model_info));
                battery_info_text.push_str(&format!("  State: {:?}\n", battery.state()));
                battery_info_text.push_str(&format!("  Battery Percentage: {:.0?}%\n", battery_percentage));
                battery_info_text.push_str(&format!("  Time to full charge: {}\n", time_to_full_info));

                // Collect battery percentage data
                battery_data.push((idx.to_string(), battery_percentage));

            }
            Err(err) => {
                eprintln!("Error getting battery info: {:?}", err);
            }
        }
    }
    let battery_info_widget = Paragraph::new(battery_info_text)
        .block(Block::default().title("'' Battery Info ''").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(battery_info_widget, area);
}