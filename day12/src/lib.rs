use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moon {
    position: (i32, i32, i32),
    velocity: (i32, i32, i32),
}

impl Moon {
    fn new(position: (i32, i32, i32)) -> Self {
        Self {
            position,
            velocity: (0, 0, 0),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MoonParseError;

impl fmt::Display for MoonParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse moon")
    }
}

impl Error for MoonParseError {}

impl FromStr for Moon {
    type Err = MoonParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s[1..s.len() - 1].split(", ");

        let x_part = parts.next().ok_or(MoonParseError)?;
        let x = x_part[2..].parse().map_err(|_| MoonParseError)?;

        let y_part = parts.next().ok_or(MoonParseError)?;
        let y = y_part[2..].parse().map_err(|_| MoonParseError)?;

        let z_part = parts.next().ok_or(MoonParseError)?;
        let z = z_part[2..].parse().map_err(|_| MoonParseError)?;

        Ok(Moon::new((x, y, z)))
    }
}

impl Moon {
    fn delta_velocity(a: i32, b: i32) -> i32 {
        if a > b {
            -1
        } else if a < b {
            1
        } else {
            0
        }
    }

    fn apply_gravity(&mut self, position: (i32, i32, i32)) {
        self.velocity.0 += Self::delta_velocity(self.position.0, position.0);
        self.velocity.1 += Self::delta_velocity(self.position.1, position.1);
        self.velocity.2 += Self::delta_velocity(self.position.2, position.2);
    }

    fn apply_velocity(&mut self) {
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;
        self.position.2 += self.velocity.2;
    }

    fn potential_energy(&self) -> i32 {
        self.position.0.abs() + self.position.1.abs() + self.position.2.abs()
    }

    fn kinetic_energy(&self) -> i32 {
        self.velocity.0.abs() + self.velocity.1.abs() + self.velocity.2.abs()
    }

    fn total_energy(&self) -> i32 {
        self.potential_energy() * self.kinetic_energy()
    }
}

pub struct System {
    moons: Vec<Moon>,
}

impl System {
    pub fn new(moons: &[Moon]) -> Self {
        Self {
            moons: moons.to_vec(),
        }
    }

    pub fn step(&mut self) {
        for i in 0..self.moons.len() {
            for j in i + 1..self.moons.len() {
                let i_position = self.moons[i].position;
                let j_position = self.moons[j].position;

                self.moons[i].apply_gravity(j_position);
                self.moons[j].apply_gravity(i_position);
            }
        }

        self.moons.iter_mut().for_each(|moon| moon.apply_velocity());
    }

    pub fn steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn total_energy(&self) -> i32 {
        self.moons.iter().map(Moon::total_energy).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_moon() {
        let position = "<x=-1, y=0, z=2>";

        assert_eq!(Ok(Moon::new((-1, 0, 2))), position.parse::<Moon>());
    }

    #[test]
    fn steps() {
        let moons = vec![
            Moon::new((-1, 0, 2)),
            Moon::new((2, -10, -7)),
            Moon::new((4, -8, 8)),
            Moon::new((3, 5, -1)),
        ];

        let mut system = System::new(&moons);
        system.steps(10);

        let expected = vec![
            Moon {
                position: (2, 1, -3),
                velocity: (-3, -2, 1),
            },
            Moon {
                position: (1, -8, 0),
                velocity: (-1, 1, 3),
            },
            Moon {
                position: (3, -6, 1),
                velocity: (3, 2, -3),
            },
            Moon {
                position: (2, 0, 4),
                velocity: (1, -1, -1),
            },
        ];

        assert_eq!(expected, system.moons);
    }

    #[test]
    fn energy() {
        let moons = vec![
            Moon::new((-1, 0, 2)),
            Moon::new((2, -10, -7)),
            Moon::new((4, -8, 8)),
            Moon::new((3, 5, -1)),
        ];

        let mut system = System::new(&moons);
        system.steps(10);

        assert_eq!(179, system.total_energy());
    }

    #[test]
    fn name() {
        let moons = vec![
            Moon::new((-8, -10, 0)),
            Moon::new((5, 5, 10)),
            Moon::new((2, -7, 3)),
            Moon::new((9, -8, -3)),
        ];

        let mut system = System::new(&moons);
        system.steps(100);

        assert_eq!(1940, system.total_energy());
    }
}
