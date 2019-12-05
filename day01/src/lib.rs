pub fn fuel_for_module(mass: i32) -> i32 {
    mass / 3 - 2
}

pub fn total_fuel_for_module(mass: i32) -> i32 {
    let mut total = 0;
    let mut fuel = fuel_for_module(mass);

    while fuel >= 0 {
        total += fuel;
        fuel = fuel_for_module(fuel);
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_for_module_with_mass_12() {
        assert_eq!(2, fuel_for_module(12));
    }

    #[test]
    fn fuel_for_module_with_mass_14() {
        assert_eq!(2, fuel_for_module(14));
    }

    #[test]
    fn fuel_for_module_with_mass_1969() {
        assert_eq!(654, fuel_for_module(1969));
    }

    #[test]
    fn fuel_for_module_with_mass_100756() {
        assert_eq!(33583, fuel_for_module(100756));
    }

    #[test]
    fn total_fuel_for_module_with_mass_14() {
        assert_eq!(2, total_fuel_for_module(14));
    }

    #[test]
    fn total_fuel_for_module_with_mass_1969() {
        assert_eq!(966, total_fuel_for_module(1969));
    }

    #[test]
    fn total_fuel_for_module_with_mass_100756() {
        assert_eq!(50346, total_fuel_for_module(100756));
    }
}
