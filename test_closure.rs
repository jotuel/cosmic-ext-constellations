use std::collections::{HashMap, HashSet};

struct State {
    cache: HashMap<String, ()>,
    in_flight: HashSet<String>,
    matrix: Option<String>,
}

fn main() {
    let mut state = State {
        cache: HashMap::new(),
        in_flight: HashSet::new(),
        matrix: Some("a".into()),
    };

    let mut tasks = Vec::new();
    let cache = &state.cache;
    let in_flight = &mut state.in_flight;
    let matrix = &state.matrix;

    let mut check_item = |item: &str| {
        if !cache.contains_key(item) && !in_flight.contains(item) {
            in_flight.insert(item.to_string());
            if let Some(m) = matrix {
                tasks.push(m.clone());
            }
        }
    };

    let items = vec!["a", "b", "a"];
    items.iter().for_each(|i| check_item(i));

    println!("{:?}", tasks);
}
