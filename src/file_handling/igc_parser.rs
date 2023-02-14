fn add_two_numbers(a: u8, b: u8) -> u8 {
    a+b
}

#[cfg(test)]

#[test]
fn two_plus_two_is_four() {
    assert_eq!(add_two_numbers(2, 2), 4)
}