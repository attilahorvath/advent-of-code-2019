use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Entry {
    id: String,
    parent_id: String,
}

impl Entry {
    pub fn new(id: &str, parent_id: &str) -> Self {
        Self {
            id: id.to_string(),
            parent_id: parent_id.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EntryParseError;

impl Error for EntryParseError {}

impl fmt::Display for EntryParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse entry")
    }
}

impl FromStr for Entry {
    type Err = EntryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(")").collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(EntryParseError);
        }

        Ok(Entry::new(parts[1], parts[0]))
    }
}

struct Orbits<'a> {
    map: &'a Map,
    id: &'a str,
}

impl<'a> Orbits<'a> {
    fn new(map: &'a Map, id: &'a str) -> Self {
        Self { map, id }
    }
}

impl<'a> Iterator for Orbits<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.id == "COM" {
            return None;
        }

        let entry = &self.map.entries[self.id];
        self.id = &entry.parent_id;

        Some(self.id)
    }
}

pub struct Map {
    entries: HashMap<String, Entry>,
}

impl Map {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|entry| (entry.id.clone(), entry))
                .collect(),
        }
    }

    pub fn total_orbits(&self) -> usize {
        self.entries
            .keys()
            .map(|id| self.orbits_of(id).count())
            .sum()
    }

    fn orbits_of(&self, id: &str) -> Orbits {
        let id = match self.entries.get(id) {
            Some(entry) => &entry.id,
            None => "COM",
        };

        Orbits::new(&self, id)
    }

    fn common_orbit(&self, a: &str, b: &str) -> &str {
        let ancestors = self.orbits_of(a).collect::<Vec<_>>();

        self.orbits_of(b)
            .find(|orbit| ancestors.contains(orbit))
            .unwrap_or("COM")
    }

    fn transfers_until(&self, source: &str, destination: &str) -> usize {
        self.orbits_of(source)
            .take_while(|&orbit| orbit != destination)
            .count()
    }

    pub fn transfers_needed(&self) -> usize {
        let common = self.common_orbit("YOU", "SAN");

        self.transfers_until("YOU", common) + self.transfers_until("SAN", common)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> Vec<Entry> {
        vec![
            Entry::new("B", "COM"),
            Entry::new("C", "B"),
            Entry::new("D", "C"),
            Entry::new("E", "D"),
            Entry::new("F", "E"),
            Entry::new("G", "B"),
            Entry::new("H", "G"),
            Entry::new("I", "D"),
            Entry::new("J", "E"),
            Entry::new("K", "J"),
            Entry::new("L", "K"),
        ]
    }

    fn test_entries_with_transfers() -> Vec<Entry> {
        let mut entries = test_entries();

        entries.push(Entry::new("YOU", "K"));
        entries.push(Entry::new("SAN", "I"));

        entries
    }

    #[test]
    fn parse_entry() {
        let entry = "COM)B".parse::<Entry>();

        assert_eq!(
            Ok(Entry {
                id: String::from("B"),
                parent_id: String::from("COM"),
            }),
            entry
        );
    }

    #[test]
    fn count_orbits_of_d() {
        let map = Map::new(test_entries());

        assert_eq!(3, map.orbits_of("D").count());
    }

    #[test]
    fn count_orbits_of_l() {
        let map = Map::new(test_entries());

        assert_eq!(7, map.orbits_of("L").count());
    }

    #[test]
    fn count_orbits_of_com() {
        let map = Map::new(test_entries());

        assert_eq!(0, map.orbits_of("COM").count());
    }

    #[test]
    fn total_orbits() {
        let map = Map::new(test_entries());

        assert_eq!(42, map.total_orbits());
    }

    #[test]
    fn common_orbit() {
        let map = Map::new(test_entries_with_transfers());

        assert_eq!("D", map.common_orbit("YOU", "SAN"));
    }

    #[test]
    fn transfers_until() {
        let map = Map::new(test_entries_with_transfers());

        assert_eq!(3, map.transfers_until("YOU", "D"));
        assert_eq!(1, map.transfers_until("SAN", "D"));
    }

    #[test]
    fn transfers_needed() {
        let map = Map::new(test_entries_with_transfers());

        assert_eq!(4, map.transfers_needed());
    }
}
