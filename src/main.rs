use std::{collections::HashMap, ops::AddAssign};

mod apps;
mod processes;

fn main() {
    let processes = processes::get_processes().expect("Unable to get processes");
    let flatpak_map = apps::get_flakpak_apps().unwrap_or(HashMap::default());

    let mut totals = HashMap::new();

    for process in &processes {
        let app_name = apps::get_app_name(process, &processes, &flatpak_map);
        totals
            .entry(app_name)
            .or_insert(0.0)
            .add_assign(process.pcpu);
    }

    print_cpu_usage(totals);
}

fn print_cpu_usage(usage: HashMap<String, f32>) {
    let mut sorted_totals: Vec<_> = usage.iter().collect();
    sorted_totals.sort_by(|a, b| b.1.total_cmp(a.1));

    for process in sorted_totals {
        println!("{}\t{}", process.0, process.1);
    }
}
