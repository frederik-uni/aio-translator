use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::Cursor,
};

pub fn read_csv() -> Result<Vec<HashMap<String, Option<String>>>, Box<dyn Error>> {
    let file = include_bytes!("map.csv");
    let mut rdr = csv::Reader::from_reader(Cursor::new(file));
    let mut people = Vec::new();

    for result in rdr.deserialize() {
        let person: HashMap<String, Option<String>> = result?;
        people.push(person);
    }

    Ok(people)
}

fn write_csv(items: &[HashMap<String, Option<String>>]) -> String {
    let v = items
        .iter()
        .flat_map(|v| {
            v.iter()
                .flat_map(|v| match v.1 {
                    Some(s) => vec![s, v.0],
                    None => vec![v.0],
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if v.iter().any(|v| v.contains(",")) {
        panic!("invalid ,")
    }
    let keys = items
        .iter()
        .flat_map(|v| v.keys().collect::<Vec<_>>())
        .collect::<HashSet<_>>();
    let mut keys = keys.into_iter().collect::<Vec<_>>();
    keys.sort();
    let mut keys_sorted: Vec<&str> = vec!["name", "639-1", "639-2/B", "639-2/T", "639-3"];
    for key in keys {
        if !keys_sorted.contains(&key.as_str()) {
            keys_sorted.push(key);
        }
    }
    let mut str = vec![];
    for item in items {
        let mut items = vec![];
        for key in &keys_sorted {
            let item = item
                .get(*key)
                .cloned()
                .unwrap_or_default()
                .unwrap_or_default();
            items.push(item);
        }
        str.push(items.join(","));
    }
    str.sort();
    str.insert(0, keys_sorted.join(","));
    str.join("\n")
}
