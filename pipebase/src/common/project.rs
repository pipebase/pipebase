/// Project input to Self
pub trait Project<Rhs = Self> {
    fn project(rhs: &Rhs) -> Self;
}

impl Project<bool> for bool {
    fn project(from: &bool) -> bool {
        *from
    }
}

impl Project<i8> for i8 {
    fn project(from: &i8) -> i8 {
        *from
    }
}

impl Project<u8> for u8 {
    fn project(from: &u8) -> u8 {
        *from
    }
}

impl Project<i16> for i16 {
    fn project(from: &i16) -> i16 {
        *from
    }
}

impl Project<u16> for u16 {
    fn project(from: &u16) -> u16 {
        *from
    }
}

impl Project<i32> for i32 {
    fn project(from: &i32) -> i32 {
        *from
    }
}

impl Project<u32> for u32 {
    fn project(from: &u32) -> u32 {
        *from
    }
}

impl Project<isize> for isize {
    fn project(from: &isize) -> isize {
        *from
    }
}

impl Project<usize> for usize {
    fn project(from: &usize) -> usize {
        *from
    }
}

impl Project<i64> for i64 {
    fn project(from: &i64) -> i64 {
        *from
    }
}

impl Project<u64> for u64 {
    fn project(from: &u64) -> u64 {
        *from
    }
}

impl Project<i128> for i128 {
    fn project(from: &i128) -> i128 {
        *from
    }
}

impl Project<u128> for u128 {
    fn project(from: &u128) -> u128 {
        *from
    }
}

impl Project<f32> for f32 {
    fn project(from: &f32) -> f32 {
        *from
    }
}

impl Project<f64> for f64 {
    fn project(from: &f64) -> f64 {
        *from
    }
}

impl Project<char> for char {
    fn project(from: &char) -> char {
        *from
    }
}

impl Project<String> for String {
    fn project(from: &String) -> String {
        from.clone()
    }
}

impl<T, U: Project<T>> Project<Option<T>> for Option<U> {
    fn project(from: &Option<T>) -> Option<U> {
        match from {
            Some(t) => Some(U::project(t)),
            None => None,
        }
    }
}

/// Project fixed size array
impl<T, U, const N: usize> Project<[T; N]> for [U; N]
where
    U: Project<T> + Default + Copy,
{
    fn project(from: &[T; N]) -> [U; N] {
        let mut to: [U; N] = [U::default(); N];
        for i in 0..N {
            to[i] = U::project(&from[i])
        }
        to
    }
}

/// Project vec
impl<T, U> Project<Vec<T>> for Vec<U>
where
    U: Project<T>,
{
    fn project(from: &Vec<T>) -> Vec<U> {
        let to: Vec<U> = from.into_iter().map(|item| U::project(item)).collect();
        to
    }
}

#[cfg(test)]
mod project_tests {

    use crate::*;

    #[derive(Debug)]
    struct IntegerRecord {
        pub r0: i32,
        pub r1: i32,
    }

    #[derive(Debug, Project)]
    #[project(input = "self::IntegerRecord")]
    struct SwappedIntegerRecord {
        #[project(from = "r1")]
        pub r0: i32,
        #[project(from = "r0")]
        pub r1: i32,
    }

    #[test]
    fn test_reverse() {
        let origin = IntegerRecord { r0: 0, r1: 1 };
        let swapped: SwappedIntegerRecord = Project::project(&origin);
        assert_eq!(1, swapped.r0);
        assert_eq!(0, swapped.r1);
    }

    #[derive(Debug, Project)]
    #[project(input = "IntegerRecord")]
    struct IntegerRecordSumPlusOne {
        #[project(alias = "r", expr = "let mut s = r.r0 + r.r1; s + 1")]
        pub s: i32,
    }

    #[test]
    fn test_sum_plus_one() {
        let origin = IntegerRecord { r0: 1, r1: 1 };
        let sum = IntegerRecordSumPlusOne::project(&origin);
        assert_eq!(3, sum.s);
    }

    struct KeyValueRecord {
        key: String,
        value: i32,
    }

    #[derive(Debug, Project)]
    #[project(input = "KeyValueRecord")]
    struct SwappedKeyValueRecord {
        #[project(from = "value")]
        key: i32,
        #[project(from = "key")]
        value: String,
        #[project(from = "key")]
        value_copy: String,
    }

    #[test]
    pub fn test_swap_key_value_record() {
        let origin = KeyValueRecord {
            key: "foo".to_owned(),
            value: 1,
        };
        let swapped: SwappedKeyValueRecord = Project::project(&origin);
        assert_eq!(origin.key, swapped.value);
        assert_eq!(origin.key, swapped.value_copy);
        assert_eq!(origin.value, swapped.key);
    }
}

/// Move and project input to Self
/// Note that with move project, each field can only be used no more than once
pub trait MoveProject<Rhs = Self> {
    fn move_project(rhs: Rhs) -> Self;
}

impl MoveProject<bool> for bool {
    fn move_project(from: bool) -> bool {
        from
    }
}

impl MoveProject<i8> for i8 {
    fn move_project(from: i8) -> i8 {
        from
    }
}

impl MoveProject<u8> for u8 {
    fn move_project(from: u8) -> u8 {
        from
    }
}

impl MoveProject<i16> for i16 {
    fn move_project(from: i16) -> i16 {
        from
    }
}

impl MoveProject<u16> for u16 {
    fn move_project(from: u16) -> u16 {
        from
    }
}

impl MoveProject<i32> for i32 {
    fn move_project(from: i32) -> i32 {
        from
    }
}

impl MoveProject<u32> for u32 {
    fn move_project(from: u32) -> u32 {
        from
    }
}

impl MoveProject<isize> for isize {
    fn move_project(from: isize) -> isize {
        from
    }
}

impl MoveProject<usize> for usize {
    fn move_project(from: usize) -> usize {
        from
    }
}

impl MoveProject<i64> for i64 {
    fn move_project(from: i64) -> i64 {
        from
    }
}

impl MoveProject<u64> for u64 {
    fn move_project(from: u64) -> u64 {
        from
    }
}

impl MoveProject<i128> for i128 {
    fn move_project(from: i128) -> i128 {
        from
    }
}

impl MoveProject<u128> for u128 {
    fn move_project(from: u128) -> u128 {
        from
    }
}

impl MoveProject<f32> for f32 {
    fn move_project(from: f32) -> f32 {
        from
    }
}

impl MoveProject<f64> for f64 {
    fn move_project(from: f64) -> f64 {
        from
    }
}

impl MoveProject<char> for char {
    fn move_project(from: char) -> char {
        from
    }
}

impl MoveProject<String> for String {
    fn move_project(from: String) -> String {
        from
    }
}

/// Move project Vec
impl<T, U> MoveProject<Vec<T>> for Vec<U>
where
    U: MoveProject<T>,
{
    fn move_project(from: Vec<T>) -> Vec<U> {
        let to: Vec<U> = from.into_iter().map(|item| U::move_project(item)).collect();
        to
    }
}

#[cfg(test)]
mod move_project_tests {

    use crate::*;

    #[derive(Debug)]
    struct IntegerRecord {
        pub r0: i32,
        pub r1: i32,
    }

    #[derive(Debug, MoveProject)]
    #[project(input = "self::IntegerRecord")]
    struct SwappedIntegerRecord {
        #[project(from = "r1")]
        pub r0: i32,
        #[project(from = "r0")]
        pub r1: i32,
    }

    #[test]
    fn test_reverse() {
        let origin = IntegerRecord { r0: 0, r1: 1 };
        let swapped: SwappedIntegerRecord = MoveProject::move_project(origin);
        assert_eq!(1, swapped.r0);
        assert_eq!(0, swapped.r1);
    }

    #[derive(Debug, MoveProject)]
    #[project(input = "IntegerRecord")]
    struct IntegerRecordSumPlusOne {
        #[project(alias = "r", expr = "let mut s = r.r0 + r.r1; s + 1")]
        pub s: i32,
    }

    #[test]
    fn test_sum_plus_one() {
        let origin = IntegerRecord { r0: 1, r1: 1 };
        let sum: IntegerRecordSumPlusOne = MoveProject::move_project(origin);
        assert_eq!(3, sum.s);
    }

    #[derive(Clone)]
    struct KeyValueRecord {
        key: String,
        value: i32,
    }

    #[derive(Debug, MoveProject)]
    #[project(input = "KeyValueRecord")]
    struct SwappedKeyValueRecord {
        #[project(from = "value")]
        key: i32,
        #[project(from = "key")]
        value: String,
    }

    #[test]
    pub fn test_swap_key_value_record() {
        let origin = KeyValueRecord {
            key: "foo".to_owned(),
            value: 1,
        };
        let swapped: SwappedKeyValueRecord = MoveProject::move_project(origin.to_owned());
        assert_eq!(origin.key, swapped.value);
        assert_eq!(origin.value, swapped.key);
    }
}
