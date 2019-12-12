use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Asteroid {
    x: i32,
    y: i32,
}

impl Asteroid {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn vector_to(&self, other: &Asteroid) -> (f64, f64) {
        ((other.x - self.x) as f64, (other.y - self.y) as f64)
    }

    fn angle_between(&self, other: &Asteroid) -> f64 {
        let d = self.vector_to(other);

        let angle = d.1.atan2(d.0) + PI * 1.5;

        angle - 2.0 * PI * ((angle + PI) / (2.0 * PI)).floor()
    }

    fn distance_to(&self, other: &Asteroid) -> f64 {
        let d = self.vector_to(other);

        (d.0.powf(2.0) + d.1.powf(2.0)).sqrt()
    }
}

impl fmt::Display for Asteroid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.x * 100 + self.y)
    }
}

pub struct Vaporize<'a> {
    angles: Vec<(&'a Asteroid, f64, f64)>,
    index: usize,
}

impl<'a> Vaporize<'a> {
    fn new(angles: &[(&'a Asteroid, f64, f64)]) -> Self {
        Self {
            angles: angles.to_vec(),
            index: 0,
        }
    }
}

impl<'a> Iterator for Vaporize<'a> {
    type Item = &'a Asteroid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.angles.is_empty() {
            return None;
        }

        let angle = self.angles.remove(self.index);

        self.index = self
            .angles
            .iter()
            .map(|&(_, a, _)| a)
            .position(|a| a > angle.1)
            .unwrap_or(0);

        Some(angle.0)
    }
}

pub struct Map {
    asteroids: Vec<Asteroid>,
}

impl Map {
    fn new(asteroids: &[Asteroid]) -> Self {
        Self {
            asteroids: asteroids.to_vec(),
        }
    }

    fn angles(&self, source: &Asteroid) -> Vec<(&Asteroid, f64, f64)> {
        let mut angles = self
            .asteroids
            .iter()
            .filter(|&asteroid| asteroid != source)
            .map(|asteroid| {
                (
                    asteroid,
                    source.angle_between(asteroid),
                    source.distance_to(asteroid),
                )
            })
            .collect::<Vec<_>>();

        angles.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap()
                .then(a.2.partial_cmp(&b.2).unwrap())
        });

        angles
    }

    pub fn vaporize(&self, source: &Asteroid) -> Vaporize {
        Vaporize::new(&self.angles(source))
    }

    pub fn best_location(&self) -> (&Asteroid, usize) {
        self.asteroids
            .iter()
            .map(|asteroid| {
                let mut angles = self.angles(asteroid);
                angles.dedup_by_key(|(_, angle, _)| *angle);
                (asteroid, angles.len())
            })
            .max_by_key(|&(_, asteroids_detected)| asteroids_detected)
            .unwrap_or((&self.asteroids[0], 0))
    }
}

impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut asteroids = vec![];

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if c == '#' {
                    asteroids.push(Asteroid::new(x as i32, y as i32));
                }
            }
        }

        Ok(Map::new(&asteroids))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_example() {
        let input = ".#..#\n\
                     .....\n\
                     #####\n\
                     ....#\n\
                     ...##";

        let map = input.parse::<Map>().unwrap();

        assert_eq!((&Asteroid::new(3, 4), 8), map.best_location());
    }

    #[test]
    fn medium_example_1() {
        let input = "......#.#.\n\
                     #..#.#....\n\
                     ..#######.\n\
                     .#.#.###..\n\
                     .#..#.....\n\
                     ..#....#.#\n\
                     #..#....#.\n\
                     .##.#..###\n\
                     ##...#..#.\n\
                     .#....####";

        let map = input.parse::<Map>().unwrap();

        assert_eq!((&Asteroid::new(5, 8), 33), map.best_location());
    }

    #[test]
    fn medium_example_2() {
        let input = "#.#...#.#.\n\
                     .###....#.\n\
                     .#....#...\n\
                     ##.#.#.#.#\n\
                     ....#.#.#.\n\
                     .##..###.#\n\
                     ..#...##..\n\
                     ..##....##\n\
                     ......#...\n\
                     .####.###.";

        let map = input.parse::<Map>().unwrap();

        assert_eq!((&Asteroid::new(1, 2), 35), map.best_location());
    }

    #[test]
    fn medium_example_3() {
        let input = ".#..#..###\n\
                     ####.###.#\n\
                     ....###.#.\n\
                     ..###.##.#\n\
                     ##.##.#.#.\n\
                     ....###..#\n\
                     ..#.#..#.#\n\
                     #..#.#.###\n\
                     .##...##.#\n\
                     .....#.#..";

        let map = input.parse::<Map>().unwrap();

        assert_eq!((&Asteroid::new(6, 3), 41), map.best_location());
    }

    #[test]
    fn large_example() {
        let input = ".#..##.###...#######\n\
                     ##.############..##.\n\
                     .#.######.########.#\n\
                     .###.#######.####.#.\n\
                     #####.##.#.##.###.##\n\
                     ..#####..#.#########\n\
                     ####################\n\
                     #.####....###.#.#.##\n\
                     ##.#################\n\
                     #####.##.###..####..\n\
                     ..######..##.#######\n\
                     ####.##.####...##..#\n\
                     .#####..#.######.###\n\
                     ##...#.##########...\n\
                     #.##########.#######\n\
                     .####.#.###.###.#.##\n\
                     ....##.##.###..#####\n\
                     .#.#.###########.###\n\
                     #.#.#.#####.####.###\n\
                     ###.##.####.##.#..##";

        let map = input.parse::<Map>().unwrap();

        assert_eq!((&Asteroid::new(11, 13), 210), map.best_location());
    }

    #[test]
    fn vaporize_small_example() {
        let input = ".#....#####...#..\n\
                     ##...##.#####..##\n\
                     ##...#...#.#####.\n\
                     ..#.....#...###..\n\
                     ..#.#.....#....##";

        let map = input.parse::<Map>().unwrap();
        let source = map.best_location().0;

        let mut vaporize = map.vaporize(source);

        assert_eq!(Some(&Asteroid::new(8, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(9, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(9, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(10, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(9, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(11, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(12, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(11, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(15, 1)), vaporize.next());

        assert_eq!(Some(&Asteroid::new(12, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(13, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(14, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(15, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(12, 3)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(16, 4)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(15, 4)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(10, 4)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(4, 4)), vaporize.next());

        assert_eq!(Some(&Asteroid::new(2, 4)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(2, 3)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(0, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(1, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(0, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(1, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(5, 2)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(1, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(5, 1)), vaporize.next());

        assert_eq!(Some(&Asteroid::new(6, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(6, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(7, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(8, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(10, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(14, 0)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(16, 1)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(13, 3)), vaporize.next());
        assert_eq!(Some(&Asteroid::new(14, 3)), vaporize.next());

        assert_eq!(None, vaporize.next());
    }

    #[test]
    fn vaporize_large_example() {
        let input = ".#..##.###...#######\n\
                     ##.############..##.\n\
                     .#.######.########.#\n\
                     .###.#######.####.#.\n\
                     #####.##.#.##.###.##\n\
                     ..#####..#.#########\n\
                     ####################\n\
                     #.####....###.#.#.##\n\
                     ##.#################\n\
                     #####.##.###..####..\n\
                     ..######..##.#######\n\
                     ####.##.####...##..#\n\
                     .#####..#.######.###\n\
                     ##...#.##########...\n\
                     #.##########.#######\n\
                     .####.#.###.###.#.##\n\
                     ....##.##.###..#####\n\
                     .#.#.###########.###\n\
                     #.#.#.#####.####.###\n\
                     ###.##.####.##.#..##";

        let map = input.parse::<Map>().unwrap();
        let source = map.best_location().0;

        assert_eq!(Some(&Asteroid::new(11, 12)), map.vaporize(source).nth(0));
        assert_eq!(Some(&Asteroid::new(12, 1)), map.vaporize(source).nth(1));
        assert_eq!(Some(&Asteroid::new(12, 2)), map.vaporize(source).nth(2));
        assert_eq!(Some(&Asteroid::new(12, 8)), map.vaporize(source).nth(9));
        assert_eq!(Some(&Asteroid::new(16, 0)), map.vaporize(source).nth(19));
        assert_eq!(Some(&Asteroid::new(16, 9)), map.vaporize(source).nth(49));
        assert_eq!(Some(&Asteroid::new(10, 16)), map.vaporize(source).nth(99));
        assert_eq!(Some(&Asteroid::new(9, 6)), map.vaporize(source).nth(198));
        assert_eq!(Some(&Asteroid::new(8, 2)), map.vaporize(source).nth(199));
        assert_eq!(Some(&Asteroid::new(10, 9)), map.vaporize(source).nth(200));
        assert_eq!(Some(&Asteroid::new(11, 1)), map.vaporize(source).nth(298));

        assert_eq!(None, map.vaporize(source).nth(299));
    }
}
