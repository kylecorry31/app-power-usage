use dirs;
use processes::Process;
use regex::Regex;
use std::{collections::HashMap, fs, io::Error, process::Command};

use crate::processes;

pub fn get_app_name(
    process: &Process,
    processes: &[Process],
    flatpak_map: &HashMap<String, String>,
) -> String {
    let app_slice_regex = Regex::new(r".*app\.slice/(app-.+)-[0-9]+\.scope").unwrap();

    if process.cgroup.ends_with("flatpak-session-helper.service") {
        return get_flatpak_session_helper_root(process, processes);
    }

    let app_slice_match = app_slice_regex.captures(&process.cgroup);
    if app_slice_match.is_none() {
        return String::from("System");
    }

    let mut app = String::from(app_slice_match.unwrap().get(1).unwrap().as_str());
    if app.starts_with("app-flatpak-") {
        app = app.replace("app-flatpak-", "");
        if flatpak_map.contains_key(app.as_str()) {
            return String::from(flatpak_map.get(app.as_str()).unwrap());
        }
    } else {
        app = app.replace("app-gnome-", "").replace("\\x2d", "-");
        app = get_gnome_app_name(&app).unwrap_or(String::from("System"));
    }

    return app;
}

pub fn get_flakpak_apps() -> Result<HashMap<String, String>, Error> {
    let output = Command::new("flatpak").arg("list").output()?;

    let output_string = String::from_utf8(output.stdout)
        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut map = HashMap::new();

    for line in output_string.lines() {
        let mut parts = line.split('\t');
        let (name, id) = (parts.next(), parts.next());
        if let (Some(name), Some(id)) = (name, id) {
            map.insert(id.to_string(), name.to_string());
        }
    }

    Ok(map)
}

fn get_flatpak_session_helper_root(process: &Process, processes: &[Process]) -> String {
    let roots = ["systemd", "bwrap"];
    let mut current = process;
    let mut parent = processes::get_parent(process, processes);
    let mut parent_name = processes::get_process_name(parent.map(|it| it.pid).unwrap_or(0));
    while parent.is_some() && !roots.contains(&parent_name.unwrap_or_default().as_str()) {
        current = parent.unwrap();
        parent = processes::get_parent(current, processes);
        parent_name = processes::get_process_name(parent.map(|it| it.pid).unwrap_or(0));
    }

    return processes::get_process_name(current.pid).unwrap_or(String::from("System"));
}

fn get_gnome_app_name(id: &str) -> Option<String> {
    let expanded_user_path = get_user_path();

    let possible_paths = [
        format!("/usr/share/applications/{}.desktop", id),
        format!(
            "{}/.local/share/applications/{}.desktop",
            expanded_user_path, id
        ),
        format!("/var/lib/flatpak/exports/share/applications/{}.desktop", id),
    ];

    let path = possible_paths
        .iter()
        .find(|path| fs::exists(path).unwrap_or(false))?;

    let desktop_contents = fs::read_to_string(path).unwrap_or(String::from(""));

    let re = Regex::new(r"Name=(.*)").unwrap();
    return Some(String::from(
        re.captures(&desktop_contents)?.get(1)?.as_str(),
    ));
}

fn get_user_path() -> String {
    let dir = dirs::home_dir();
    if dir.is_none() {
        return String::from("~");
    }

    return String::from(dir.unwrap().to_str().unwrap());
}
