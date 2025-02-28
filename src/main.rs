mod apps;
mod power;
mod processes;

fn main() {
    let power_usage = power::get_app_power_usages();
    let mut sorted_usage: Vec<_> = power_usage.iter().collect();
    sorted_usage.sort_by(|a, b| b.1.total_cmp(a.1));

    for process in sorted_usage {
        println!("{}\t{}", process.0, process.1);
    }
}
