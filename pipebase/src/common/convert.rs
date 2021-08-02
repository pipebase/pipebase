pub trait Convert<Rhs = Self> {
    fn convert(rhs: Rhs) -> Self;
}

/// Convert Vec
impl<T, U> Convert<Vec<T>> for Vec<U>
where
    U: Convert<T>,
{
    fn convert(input: Vec<T>) -> Vec<U> {
        let to: Vec<U> = input.into_iter().map(U::convert).collect();
        to
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;

    #[derive(Debug)]
    struct Record {
        key: String,
        value: i32,
    }

    #[derive(Convert)]
    #[convert(input = "Record")]
    struct SwappedRecord {
        #[convert(from = "value")]
        key: i32,
        #[convert(from = "key")]
        value: String,
    }

    #[test]
    fn test_swap() {
        let origin = Record {
            key: "foo".to_owned(),
            value: 1,
        };
        let swapped: SwappedRecord = SwappedRecord::convert(origin);
        assert_eq!(String::from("foo"), swapped.value);
        assert_eq!(1, swapped.key);
    }
}
