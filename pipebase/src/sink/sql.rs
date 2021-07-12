// export data using SQL, render SQL statement with data fields
pub trait Sql {
    fn sql(&self) -> String;
}
