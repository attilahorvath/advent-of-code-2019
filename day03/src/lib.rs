use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Segment {
    Horizontal(i32, i32, i32),
    Vertical(i32, i32, i32),
}

impl Segment {
    fn end(&self) -> (i32, i32) {
        match *self {
            Segment::Horizontal(y, _x1, x2) => (x2, y),
            Segment::Vertical(x, _y1, y2) => (x, y2),
        }
    }

    fn range(&self) -> std::ops::RangeInclusive<i32> {
        match *self {
            Segment::Horizontal(_y, x1, x2) => {
                if x1 < x2 {
                    (x1..=x2)
                } else {
                    (x2..=x1)
                }
            }
            Segment::Vertical(_x, y1, y2) => {
                if y1 < y2 {
                    (y1..=y2)
                } else {
                    (y2..=y1)
                }
            }
        }
    }

    fn length(&self) -> i32 {
        self.range().end() - self.range().start()
    }

    fn intersection_with(&self, wire: &Self) -> Option<((i32, i32), i32)> {
        match *self {
            Segment::Horizontal(y, x_start, _) => match *wire {
                Segment::Horizontal(_, _, _) => None,
                Segment::Vertical(x, y_start, _) => {
                    if self.range().contains(&x) && wire.range().contains(&y) {
                        Some(((x, y), (x - x_start).abs() + (y - y_start).abs()))
                    } else {
                        None
                    }
                }
            },
            Segment::Vertical(x, y_start, _) => match *wire {
                Segment::Horizontal(y, x_start, _) => {
                    if self.range().contains(&y) && wire.range().contains(&x) {
                        Some(((x, y), (x - x_start).abs() + (y - y_start).abs()))
                    } else {
                        None
                    }
                }
                Segment::Vertical(_, _, _) => None,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Wire {
    segments: Vec<Segment>,
}

impl Wire {
    fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    fn intersections_with(&self, wire: &Self) -> Vec<((i32, i32), i32)> {
        let mut intersections = vec![];
        let mut a_steps = 0;

        for a in &self.segments {
            let mut b_steps = 0;

            for b in &wire.segments {
                if let Some((intersection, steps)) = a.intersection_with(b) {
                    if intersection.0 != 0 || intersection.1 != 0 {
                        let total_steps = a_steps + b_steps + steps;

                        intersections.push((intersection, total_steps));
                    }
                }

                b_steps += b.length();
            }

            a_steps += a.length();
        }

        intersections
    }

    pub fn closest_intersection_with(&self, wire: &Self) -> Option<i32> {
        let intersections = self.intersections_with(wire);

        intersections
            .iter()
            .map(|intersection| (intersection.0).0.abs() + (intersection.0).1.abs())
            .min()
    }

    pub fn fewest_steps_with(&self, wire: &Self) -> Option<i32> {
        let intersections = self.intersections_with(wire);

        intersections
            .iter()
            .map(|intersection| intersection.1)
            .min()
    }
}

#[derive(Debug, PartialEq)]
pub struct WireParseError;

impl FromStr for Wire {
    type Err = WireParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut segments = vec![];
        let mut start = (0, 0);

        for elem in s.split(",") {
            let (direction, distance) = elem.split_at(1);
            let distance = distance.parse::<i32>().map_err(|_| WireParseError)?;

            let segment = match direction {
                "U" => Segment::Vertical(start.0, start.1, start.1 - distance),
                "D" => Segment::Vertical(start.0, start.1, start.1 + distance),
                "L" => Segment::Horizontal(start.1, start.0, start.0 - distance),
                "R" => Segment::Horizontal(start.1, start.0, start.0 + distance),
                _ => return Err(WireParseError),
            };

            segments.push(segment);

            start = segment.end();
        }

        Ok(Wire::new(segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_first_wire() {
        let wire = Wire::new(vec![
            Segment::Horizontal(0, 0, 8),
            Segment::Vertical(8, 0, -5),
            Segment::Horizontal(-5, 8, 3),
            Segment::Vertical(3, -5, -2),
        ]);

        assert_eq!(Ok(wire), "R8,U5,L5,D3".parse::<Wire>());
    }

    #[test]
    fn parse_second_wire() {
        let wire = Wire::new(vec![
            Segment::Vertical(0, 0, -7),
            Segment::Horizontal(-7, 0, 6),
            Segment::Vertical(6, -7, -3),
            Segment::Horizontal(-3, 6, 2),
        ]);

        assert_eq!(Ok(wire), "U7,R6,D4,L4".parse::<Wire>());
    }

    #[test]
    fn intersections() {
        let wire1 = "R8,U5,L5,D3".parse::<Wire>().unwrap();
        let wire2 = "U7,R6,D4,L4".parse::<Wire>().unwrap();

        assert_eq!(
            vec![((6, -5), 30), ((3, -3), 40)],
            wire1.intersections_with(&wire2)
        );
    }

    #[test]
    fn closest_intersection_1() {
        let wire1 = "R8,U5,L5,D3".parse::<Wire>().unwrap();
        let wire2 = "U7,R6,D4,L4".parse::<Wire>().unwrap();

        assert_eq!(Some(6), wire1.closest_intersection_with(&wire2));
    }

    #[test]
    fn closest_intersection_2() {
        let wire1 = "R75,D30,R83,U83,L12,D49,R71,U7,L72"
            .parse::<Wire>()
            .unwrap();

        let wire2 = "U62,R66,U55,R34,D71,R55,D58,R83".parse::<Wire>().unwrap();

        assert_eq!(Some(159), wire1.closest_intersection_with(&wire2));
    }

    #[test]
    fn closest_intersection_3() {
        let wire1 = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51"
            .parse::<Wire>()
            .unwrap();

        let wire2 = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            .parse::<Wire>()
            .unwrap();

        assert_eq!(Some(135), wire1.closest_intersection_with(&wire2));
    }

    #[test]
    fn fewest_steps_1() {
        let wire1 = "R8,U5,L5,D3".parse::<Wire>().unwrap();
        let wire2 = "U7,R6,D4,L4".parse::<Wire>().unwrap();

        assert_eq!(Some(30), wire1.fewest_steps_with(&wire2));
    }

    #[test]
    fn fewest_steps_2() {
        let wire1 = "R75,D30,R83,U83,L12,D49,R71,U7,L72"
            .parse::<Wire>()
            .unwrap();

        let wire2 = "U62,R66,U55,R34,D71,R55,D58,R83".parse::<Wire>().unwrap();

        assert_eq!(Some(610), wire1.fewest_steps_with(&wire2));
    }

    #[test]
    fn fewest_steps_3() {
        let wire1 = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51"
            .parse::<Wire>()
            .unwrap();

        let wire2 = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            .parse::<Wire>()
            .unwrap();

        assert_eq!(Some(410), wire1.fewest_steps_with(&wire2));
    }
}
