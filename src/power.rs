use std::{collections::HashMap, ops::AddAssign};

use crate::{apps, processes};

pub fn get_app_power_usages() -> HashMap<String, f32> {
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

    return totals;
}
