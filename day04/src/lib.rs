pub fn validate_password(password: &str) -> bool {
    let mut last_digit = '\0';
    let mut group_size = 1;
    let mut doubles = false;

    for digit in password.chars() {
        if digit < last_digit {
            return false;
        }

        if digit == last_digit {
            group_size += 1;
        } else {
            if group_size == 2 {
                doubles = true;
            }

            group_size = 1;
        }

        last_digit = digit;
    }

    if group_size == 2 {
        doubles = true;
    }

    doubles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_password() {
        assert_eq!(true, validate_password("112233"));
    }

    #[test]
    fn decreasing_digits() {
        assert_eq!(false, validate_password("223450"));
    }

    #[test]
    fn no_doubles() {
        assert_eq!(false, validate_password("123789"));
    }

    #[test]
    fn group_too_long() {
        assert_eq!(false, validate_password("123444"));
    }

    #[test]
    fn multiple_groups() {
        assert_eq!(true, validate_password("111122"));
    }
}
