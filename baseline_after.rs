use std::collections::{HashMap, HashSet};

struct State {
    media_cache: HashMap<String, ()>,
    media_in_flight: HashSet<String>,
}

fn main() {
    let mut state = State {
        media_cache: HashMap::new(),
        media_in_flight: HashSet::new(),
    };

    let mut urls_to_fetch = Vec::new();
    let mut check_item = |url_str: &str| {
        if !state.media_cache.contains_key(url_str) && !state.media_in_flight.contains(url_str) {
            urls_to_fetch.push(url_str.to_string());
        }
    };

    // Simulate 10 duplicate diffs arriving before the first fetch completes
    for _ in 0..10 {
        check_item("mxc://matrix.org/avatar123");
    }

    let mut tasks_spawned = 0;
    for url_str in urls_to_fetch {
        if !state.media_in_flight.contains(&url_str) {
            state.media_in_flight.insert(url_str.clone());
            tasks_spawned += 1;
        }
    }

    println!("Tasks spawned (after optimization): {}", tasks_spawned);
}
