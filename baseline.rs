use std::collections::{HashMap, HashSet};

struct State {
    media_cache: HashMap<String, ()>,
}

fn main() {
    let state = State {
        media_cache: HashMap::new(),
    };

    let mut tasks_spawned = 0;

    // Simulate check_item closure being called 10 times with the same URL
    let mut check_item = |url_str: &str| {
        if !state.media_cache.contains_key(url_str) {
            tasks_spawned += 1;
        }
    };

    for _ in 0..10 {
        check_item("mxc://matrix.org/avatar123");
    }

    println!("Tasks spawned (before optimization): {}", tasks_spawned);
}
