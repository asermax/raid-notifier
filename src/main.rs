use std::fs;
use std::thread;
use std::time::Duration;
use regex::Regex;
use notify_rust::Notification;

const SLEEP_DURATION: u64 = 60 * 15;
const MDSTAT_PATH: &str = "/proc/mdstat";

struct Raid {
    name: String,
    all_devices: u8,
    current_devices: u8,
}

impl Raid {
    fn is_failing(&self) -> bool {
        self.current_devices < self.all_devices
    }
}

impl std::fmt::Display for Raid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({}/{})", self.name, self.current_devices, self.all_devices)
    }
}

impl PartialEq for Raid {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn main() {
    let mut failing_raids: Vec<Raid> = Vec::new();
    let sleep_duration = Duration::from_secs(SLEEP_DURATION);
    let first_line_re = Regex::new("Personalities.*").unwrap();
    let raid_re = Regex::new(r"(?m)(.+?) :[\w\W]*?\[(\d)/(\d)]").unwrap();

    loop {
        // we want to repeat this checking every N seconds and notify if necessary
        thread::sleep(sleep_duration);
        let mut should_notify = false;

        // read the mdstat proc content and parse the current raids
        let mdstat_contents = fs::read_to_string(MDSTAT_PATH)
            .expect("Failed to read mdstat");
        let mdstat_contents = first_line_re.replace(&mdstat_contents, "");

        for raid_capture in raid_re.captures_iter(&mdstat_contents) {
            let raid = Raid {
                name: String::from(&raid_capture[1]),
                all_devices: raid_capture[2].parse::<u8>().unwrap(),
                current_devices: raid_capture[3].parse::<u8>().unwrap(),
            };

            if raid.is_failing() && !failing_raids.contains(&raid) {
                // we want to make sure the raid wasn't already failing already before we show
                // a new notification
                should_notify = true;
                failing_raids.push(raid);
            } else if !raid.is_failing() && failing_raids.contains(&raid) {
                // and we also want to make sure we notify if it got fixed and failed again
                failing_raids.retain(|x| x.name != raid.name);
            }
        }

        if should_notify {
            notify(&failing_raids);
        }
    }
}

fn notify(failing_raids: &Vec<Raid>) {
    let raid_list: Vec<String> = failing_raids
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let raid_string = raid_list.join(" ,");

    Notification::new()
        .summary("One or more raids are failing")
        .body(&format!("The following raids are failing: {}", raid_string))
        .icon("drive-harddisk")
        .show().unwrap();
}
