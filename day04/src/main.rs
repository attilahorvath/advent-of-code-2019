use day04::*;

const PASSWORD_RANGE: std::ops::RangeInclusive<i32> = 158_126..=624_574;

fn main() {
    let valid_passwords = PASSWORD_RANGE
        .filter(|password| validate_password(&password.to_string()))
        .count();

    println!("Valid passwords: {}", valid_passwords);
}
