use std::fs;
use std::process;
use std::{io::Error, process::Command};

#[derive(Clone)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub pcpu: f32,
    pub cgroup: String,
}

pub fn get_processes() -> Result<Vec<Process>, Error> {
    let output = Command::new("ps")
        .arg("-eo")
        .arg("pid,ppid,pcpu")
        .arg("--no-header")
        .output()?;
    let output_string = String::from_utf8(output.stdout)
        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

    let my_pid = process::id();

    // Parse the output string (each line is a process - PID and PCPU separated by a space)
    let processes: Vec<Process> = output_string
        .lines()
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let pid = parts[0].parse().unwrap();
            let ppid = parts[1].parse().unwrap();
            let pcpu = parts[2].parse().unwrap();
            let cgroup = get_cgroup(pid).unwrap_or(String::from(""));
            Process {
                pid,
                ppid,
                pcpu,
                cgroup,
            }
        })
        .collect();

    let filtered_processes: Vec<Process> = processes
        .iter()
        .filter(|process| !is_subprocess(my_pid, process.pid, &processes))
        .cloned()
        .collect();

    Ok(filtered_processes)
}

fn get_cgroup(pid: u32) -> Result<String, Error> {
    return fs::read_to_string(format!("/proc/{}/cgroup", pid));
}

pub fn get_parent<'a>(process: &Process, processes: &'a [Process]) -> Option<&'a Process> {
    return processes.iter().find(|current| current.pid == process.ppid);
}

fn is_subprocess(pid: u32, potential_child_pid: u32, processes: &[Process]) -> bool {
    let process = processes
        .iter()
        .find(|process| process.pid == potential_child_pid);

    if process.is_none() {
        return false;
    }

    let mut current = process.unwrap();

    while current.pid != pid {
        let parent = get_parent(current, processes);
        if parent.is_none() {
            return false;
        }
        current = parent.unwrap();
    }

    return current.pid == pid;
}

pub fn get_process_name(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("comm=")
        .output();

    if output.is_err() {
        return None;
    }

    let output_string = String::from_utf8(output.unwrap().stdout);

    if output_string.is_err() {
        return None;
    }

    return Some(output_string.unwrap());
}
