/// Filter object itself
pub trait Filter<Rhs = Self> {
    fn filter(rhs: &Rhs) -> bool;
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;

    #[derive(Clone, Debug, Filter)]
    #[filter(alias = "r", predicate = "r.r0 + r.r1 < 1")]
    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[test]
    fn test_filter() {
        let mut r = Record { r0: 1, r1: 1 };
        assert!(!Record::filter(&r));
        r.r0 = 0;
        r.r1 = 0;
        assert!(Record::filter(&r));
    }
}
