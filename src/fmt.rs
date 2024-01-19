use number_prefix::{Amounts, NumberPrefix};

pub fn readable_amount<T: Amounts + std::fmt::Display>(amount: T) -> String {
    match NumberPrefix::decimal(amount) {
        NumberPrefix::Standalone(bytes) => {
            format!("{}", bytes)
        }
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{:.1}{}", n, prefix.symbol())
        }
    }
}
