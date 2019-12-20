use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::ops::Mul;
use std::str::FromStr;

const ORE_RESERVES: u64 = 1_000_000_000_000;

#[derive(Debug, PartialEq)]
pub struct ReactionParseError;

impl fmt::Display for ReactionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse reaction")
    }
}

impl Error for ReactionParseError {}

#[derive(Clone, Debug, PartialEq)]
struct Term {
    quantity: u64,
    chemical: String,
}

impl Term {
    fn new(quantity: u64, chemical: &str) -> Self {
        Self {
            quantity,
            chemical: chemical.to_string(),
        }
    }
}

impl FromStr for Term {
    type Err = ReactionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(" ");

        let quantity = parts
            .next()
            .ok_or(ReactionParseError)?
            .clone()
            .parse()
            .map_err(|_| ReactionParseError)?;

        let chemical = parts.next().ok_or(ReactionParseError)?;

        Ok(Term::new(quantity, chemical))
    }
}

impl Mul<u64> for &Term {
    type Output = Term;

    fn mul(self, rhs: u64) -> Self::Output {
        Term::new(self.quantity * rhs, &self.chemical)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    inputs: Vec<Term>,
    output: Term,
}

impl Reaction {
    fn new(inputs: Vec<Term>, output: Term) -> Self {
        Self { inputs, output }
    }
}

impl FromStr for Reaction {
    type Err = ReactionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(" => ");

        let inputs = parts
            .next()
            .ok_or(ReactionParseError)?
            .split(", ")
            .map(str::parse::<Term>)
            .collect::<Result<Vec<_>, ReactionParseError>>()?;

        let output = parts.next().ok_or(ReactionParseError)?.parse()?;

        Ok(Reaction::new(inputs, output))
    }
}

pub struct Reactor {
    reactions: HashMap<String, Reaction>,
}

impl Reactor {
    pub fn new(reactions: Vec<Reaction>) -> Self {
        let reactions = reactions
            .into_iter()
            .map(|reaction| (reaction.output.chemical.clone(), reaction))
            .collect();

        Self { reactions }
    }

    fn ore_cost(&self, term: Term, inventory: &mut HashMap<String, u64>) -> u64 {
        if term.chemical == "ORE" {
            return term.quantity;
        }

        let leftover = inventory.entry(term.chemical.clone()).or_insert(0);

        if *leftover >= term.quantity {
            *leftover -= term.quantity;

            return 0;
        }

        let needed = term.quantity - *leftover;

        *leftover = 0;

        let reaction = &self.reactions[&term.chemical];
        let produced = reaction.output.quantity;
        let rounds = (needed as f64 / produced as f64).ceil() as u64;

        if produced * rounds > needed {
            *leftover += produced * rounds - needed;
        }

        reaction
            .inputs
            .iter()
            .map(|input| self.ore_cost(input * rounds, inventory))
            .sum()
    }

    pub fn fuel_cost(&self) -> u64 {
        self.ore_cost(Term::new(1, "FUEL"), &mut HashMap::new())
    }

    fn find_max_fuel(&self, min: u64, max: u64) -> u64 {
        let midpoint = min + (max - min) / 2;

        if midpoint == min {
            return min;
        }

        let ore_needed = self.ore_cost(Term::new(midpoint, "FUEL"), &mut HashMap::new());

        if ore_needed > ORE_RESERVES {
            self.find_max_fuel(min, midpoint)
        } else {
            self.find_max_fuel(midpoint, max)
        }
    }

    pub fn max_fuel(&self) -> u64 {
        let min = ORE_RESERVES / self.ore_cost(Term::new(1, "FUEL"), &mut HashMap::new());
        let max = min * 10;

        self.find_max_fuel(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reaction() {
        let input = "1 A, 2 B, 3 C => 4 D";
        let reaction = Reaction::new(
            vec![Term::new(1, "A"), Term::new(2, "B"), Term::new(3, "C")],
            Term::new(4, "D"),
        );

        assert_eq!(Ok(reaction), input.parse());
    }

    #[test]
    fn simple_case() {
        let reactions = vec![
            Reaction::new(vec![Term::new(10, "ORE")], Term::new(10, "A")),
            Reaction::new(vec![Term::new(1, "ORE")], Term::new(1, "B")),
            Reaction::new(
                vec![Term::new(7, "A"), Term::new(1, "B")],
                Term::new(1, "C"),
            ),
            Reaction::new(
                vec![Term::new(7, "A"), Term::new(1, "C")],
                Term::new(1, "D"),
            ),
            Reaction::new(
                vec![Term::new(7, "A"), Term::new(1, "D")],
                Term::new(1, "E"),
            ),
            Reaction::new(
                vec![Term::new(7, "A"), Term::new(1, "E")],
                Term::new(1, "FUEL"),
            ),
        ];

        assert_eq!(31, Reactor::new(reactions).fuel_cost());
    }

    #[test]
    fn medium_case() {
        let reactions = vec![
            Reaction::new(vec![Term::new(9, "ORE")], Term::new(2, "A")),
            Reaction::new(vec![Term::new(8, "ORE")], Term::new(3, "B")),
            Reaction::new(vec![Term::new(7, "ORE")], Term::new(5, "C")),
            Reaction::new(
                vec![Term::new(3, "A"), Term::new(4, "B")],
                Term::new(1, "AB"),
            ),
            Reaction::new(
                vec![Term::new(5, "B"), Term::new(7, "C")],
                Term::new(1, "BC"),
            ),
            Reaction::new(
                vec![Term::new(4, "C"), Term::new(1, "A")],
                Term::new(1, "CA"),
            ),
            Reaction::new(
                vec![Term::new(2, "AB"), Term::new(3, "BC"), Term::new(4, "CA")],
                Term::new(1, "FUEL"),
            ),
        ];

        assert_eq!(165, Reactor::new(reactions).fuel_cost());
    }

    #[test]
    fn large_case_1() {
        let reactions = "157 ORE => 5 NZVS\n\
                         165 ORE => 6 DCFZ\n\
                         44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL\n\
                         12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ\n\
                         179 ORE => 7 PSHF\n\
                         177 ORE => 5 HKGWZ\n\
                         7 DCFZ, 7 PSHF => 2 XJWVT\n\
                         165 ORE => 2 GPVTF\n\
                         3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT";

        let reactions = reactions
            .lines()
            .map(|reaction| reaction.parse().unwrap())
            .collect::<Vec<_>>();

        let reactor = Reactor::new(reactions);

        assert_eq!(13_312, reactor.fuel_cost());
        assert_eq!(82_892_753, reactor.max_fuel());
    }

    #[test]
    fn large_case_2() {
        let reactions = "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG\n\
                         17 NVRVD, 3 JNWZP => 8 VPVL\n\
                         53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL\n\
                         22 VJHF, 37 MNCFX => 5 FWMGM\n\
                         139 ORE => 4 NVRVD\n\
                         144 ORE => 7 JNWZP\n\
                         5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC\n\
                         5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV\n\
                         145 ORE => 6 MNCFX\n\
                         1 NVRVD => 8 CXFTF\n\
                         1 VJHF, 6 MNCFX => 4 RFSQX\n\
                         176 ORE => 6 VJHF";

        let reactions = reactions
            .lines()
            .map(|reaction| reaction.parse().unwrap())
            .collect::<Vec<_>>();

        let reactor = Reactor::new(reactions);

        assert_eq!(180_697, reactor.fuel_cost());
        assert_eq!(5_586_022, reactor.max_fuel());
    }

    #[test]
    fn large_case_3() {
        let reactions = "171 ORE => 8 CNZTR\n\
                         7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL\n\
                         114 ORE => 4 BHXH\n\
                         14 VRPVC => 6 BMBT\n\
                         6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL\n\
                         6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT\n\
                         15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW\n\
                         13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW\n\
                         5 BMBT => 4 WPTQ\n\
                         189 ORE => 9 KTJDG\n\
                         1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP\n\
                         12 VRPVC, 27 CNZTR => 2 XDBXC\n\
                         15 KTJDG, 12 BHXH => 5 XCVML\n\
                         3 BHXH, 2 VRPVC => 7 MZWV\n\
                         121 ORE => 7 VRPVC\n\
                         7 XCVML => 6 RJRHP\n\
                         5 BHXH, 4 VRPVC => 5 LTCX";

        let reactions = reactions
            .lines()
            .map(|reaction| reaction.parse().unwrap())
            .collect::<Vec<_>>();

        let reactor = Reactor::new(reactions);

        assert_eq!(2_210_736, reactor.fuel_cost());
        assert_eq!(460_664, reactor.max_fuel());
    }
}
