// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=ad7391366c8408d8cb0097f1f867808f

use sqlx::arguments::Arguments;
use sqlx::{encode::Encode, Database, Type};


/// As of right now, you can only use this with the `query` function because
/// it's the only function that has a `bind_all` method to pass the arguments struct.
///
/// ```
/// let (query, arguments) = QueryBuilder::new()
///     .append("SELECT * FROM someTable")
///     .condition("ID = ?")
///     .bind("123abc")
///     .append("ORDER BY id")
///     .into_query_and_arguments();
///
/// let mut cursor = sqlx::query(&query)
///     .bind_all(arguments)
///     .fetch(&mut connection);
/// ```
#[derive(Debug, Clone)]
pub struct QueryBuilder<DB: Database> {
    query: String,
    arguments: DB::Arguments,
    has_where_statement: bool,
}

impl<DB: Database> QueryBuilder<DB> {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            arguments: DB::Arguments::default(),
            has_where_statement: false,
        }
    }

    pub fn into_query_and_arguments(self) -> (String, DB::Arguments) {
        (self.query, self.arguments)
    }

    pub fn append(mut self, text: &str) -> Self {
        self.query.push_str(text);
        self.query.push(' ');
        self
    }

    /// Adds a WHERE clause or appends to an existing WHERE with AND
    pub fn condition(mut self, condition: &str) -> Self {
        if self.has_where_statement {
            self.query.push_str("AND ");
        } else {
            self.query.push_str("WHERE ");
            self.has_where_statement = true;
        }

        self.query.push_str(condition);
        self.query.push(' ');

        self
    }

    pub fn multi_condition<T>(mut self, condition: &str, values: &[T]) -> Self
        where T: Type<DB> + Encode<DB>
    {
        let mut full_condition = String::from("(");
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                full_condition.push_str("OR ")
            }
            full_condition.push_str(condition);
            full_condition.push(' ');
            self.arguments.add(value);
        }
        full_condition.push_str(")");

        self.condition(&full_condition)
    }

    pub fn bind<T>(mut self, value: T) -> Self
        where
            T: Type<DB> + Encode<DB>,
    {
        self.arguments.add(value);
        self
    }

    pub fn bind_slice<T>(mut self, values: &[T]) -> Self
        where T: Type<DB> + Encode<DB>
    {
        for value in values {
            self.arguments.add(value);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_query() -> anyhow::Result<()> {
        let expected = "\
            SELECT * FROM someTable \
            WHERE ID = ? \
            ORDER BY id \
        ";

        let (actual, _) = QueryBuilder::new()
            .append("SELECT * FROM someTable")
            .condition("ID = ?")
            .bind("123abc")
            .append("ORDER BY id")
            .into_query_and_arguments();

        assert_eq!(expected, actual);

        Ok(())
    }
}
