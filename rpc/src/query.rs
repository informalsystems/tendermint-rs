//! Structured querying for the Tendermint RPC event subscription system.
//!
//! See [`Query`] for details as to how to construct queries.
//!
//! [`Query`]: struct.Query.html

use chrono::{Date, DateTime, FixedOffset, Utc};

/// A structured query for use in interacting with the Tendermint RPC event
/// subscription system.
///
/// Allows for compile-time validation of queries.
///
/// See the [subscribe endpoint documentation] for more details.
///
/// ## Examples
///
/// ```rust
/// use tendermint_rpc::query::{Query, EventType};
///
/// let query = Query::from(EventType::NewBlock);
/// assert_eq!("tm.event = 'NewBlock'", query.to_string());
///
/// let query = Query::from(EventType::Tx).and_eq("tx.hash", "XYZ");
/// assert_eq!("tm.event = 'Tx' AND tx.hash = 'XYZ'", query.to_string());
///
/// let query = Query::from(EventType::Tx).and_gte("tx.height", 100_i64);
/// assert_eq!("tm.event = 'Tx' AND tx.height >= 100", query.to_string());
/// ```
///
/// [subscribe endpoint documentation]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    // We can only have at most one event type at present in a query.
    event_type: Option<EventType>,
    // We can have zero or more additional conditions associated with a query.
    // Conditions are currently exclusively joined by logical ANDs.
    conditions: Vec<Condition>,
}

impl Query {
    /// Query constructor testing whether `<key> = <value>`
    pub fn eq(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(key.to_string(), Operation::Eq(value.into()))],
        }
    }

    /// Query constructor testing whether `<key> < <value>`
    pub fn lt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(key.to_string(), Operation::Lt(value.into()))],
        }
    }

    /// Query constructor testing whether `<key> <= <value>`
    pub fn lte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(
                key.to_string(),
                Operation::Lte(value.into()),
            )],
        }
    }

    /// Query constructor testing whether `<key> > <value>`
    pub fn gt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(key.to_string(), Operation::Gt(value.into()))],
        }
    }

    /// Query constructor testing whether `<key> >= <value>`
    pub fn gte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(
                key.to_string(),
                Operation::Gte(value.into()),
            )],
        }
    }

    /// Query constructor testing whether `<key> CONTAINS <value>` (assuming
    /// `key` contains a string, this tests whether `value` is a sub-string
    /// within it).
    pub fn contains(key: impl ToString, value: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(
                key.to_string(),
                Operation::Contains(value.to_string()),
            )],
        }
    }

    /// Query constructor testing whether `<key> EXISTS`.
    pub fn exists(key: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(key.to_string(), Operation::Exists)],
        }
    }

    pub fn and_eq(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::new(key.to_string(), Operation::Eq(value.into())));
        self
    }

    pub fn and_lt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::new(key.to_string(), Operation::Lt(value.into())));
        self
    }

    pub fn and_lte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions.push(Condition::new(
            key.to_string(),
            Operation::Lte(value.into()),
        ));
        self
    }

    pub fn and_gt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::new(key.to_string(), Operation::Gt(value.into())));
        self
    }

    pub fn and_gte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions.push(Condition::new(
            key.to_string(),
            Operation::Gte(value.into()),
        ));
        self
    }

    pub fn and_contains(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.conditions.push(Condition::new(
            key.to_string(),
            Operation::Contains(value.to_string()),
        ));
        self
    }

    pub fn and_exists(mut self, key: impl ToString) -> Self {
        self.conditions
            .push(Condition::new(key.to_string(), Operation::Exists));
        self
    }
}

impl Default for Query {
    /// An empty query matches any set of events. See [these docs].
    ///
    /// [these docs]: https://godoc.org/github.com/tendermint/tendermint/libs/pubsub/query#Empty
    fn default() -> Self {
        Self {
            event_type: None,
            conditions: Vec::new(),
        }
    }
}

impl From<EventType> for Query {
    fn from(t: EventType) -> Self {
        Self {
            event_type: Some(t),
            conditions: Vec::new(),
        }
    }
}

impl ToString for Query {
    fn to_string(&self) -> String {
        let mut conditions: Vec<String> = Vec::new();
        if let Some(t) = &self.event_type {
            conditions.push(format!("tm.event = '{}'", t.to_string()));
        }
        self.conditions
            .iter()
            .for_each(|c| conditions.push(c.to_string()));
        conditions.join(" AND ")
    }
}

/// The types of Tendermint events for which we can query at present.
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    NewBlock,
    Tx,
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match self {
            EventType::NewBlock => "NewBlock",
            EventType::Tx => "Tx",
        }
        .to_string()
    }
}

/// A `Condition` takes the form of `<key> <operation>` in a [`Query`].
///
/// See [`Operation`] for the types of operations supported.
///
/// [`Query`]: struct.Query.html
/// [`Operation`]: enum.Operation.html
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    key: String,
    op: Operation,
}

impl Condition {
    fn new(key: String, op: Operation) -> Self {
        Self { key, op }
    }
}

impl ToString for Condition {
    fn to_string(&self) -> String {
        format!("{} {}", self.key, self.op.to_string())
    }
}

/// The different types of operations supported by a [`Condition`] in a
/// [`Query`].
///
/// [`Condition`]: struct.Condition.html
/// [`Query`]: struct.Query.html
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Equals
    Eq(Operand),
    /// Less than
    Lt(Operand),
    /// Less than or equal to
    Lte(Operand),
    /// Greater than
    Gt(Operand),
    /// Greater than or equal to
    Gte(Operand),
    /// Contains (to check if a string contains a certain sub-string)
    Contains(String),
    /// Exists
    Exists,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::Eq(op) => format!("= {}", op.to_string()),
            Operation::Lt(op) => format!("< {}", op.to_string()),
            Operation::Lte(op) => format!("<= {}", op.to_string()),
            Operation::Gt(op) => format!("> {}", op.to_string()),
            Operation::Gte(op) => format!(">= {}", op.to_string()),
            Operation::Contains(op) => format!("CONTAINS {}", single_quote_string(op.clone())),
            Operation::Exists => "EXISTS".to_string(),
        }
    }
}

/// A typed operand for use in an [`Operation`].
///
/// According to the [Tendermint RPC subscribe docs][tm-subscribe],
/// an operand can be a string, number, date or time. We differentiate here
/// between integer and floating point numbers.
///
/// [`Operation`]: enum.Operation.html
/// [tm-subscribe]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Date(Date<Utc>),
    DateTime(DateTime<Utc>),
}

impl ToString for Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::String(s) => single_quote_string(s.clone()),
            Operand::Signed(i) => format!("{}", i),
            Operand::Unsigned(u) => format!("{}", u),
            Operand::Float(f) => format!("{}", f),
            Operand::Date(d) => single_quote_string(d.format("%Y-%m-%d").to_string()),
            Operand::DateTime(dt) => single_quote_string(dt.to_rfc3339()),
        }
    }
}

impl Into<Operand> for String {
    fn into(self) -> Operand {
        Operand::String(self)
    }
}

impl Into<Operand> for char {
    fn into(self) -> Operand {
        Operand::String(self.to_string())
    }
}

impl Into<Operand> for &str {
    fn into(self) -> Operand {
        Operand::String(self.to_string())
    }
}

impl Into<Operand> for i64 {
    fn into(self) -> Operand {
        Operand::Signed(self)
    }
}

impl Into<Operand> for i32 {
    fn into(self) -> Operand {
        Operand::Signed(self as i64)
    }
}

impl Into<Operand> for i16 {
    fn into(self) -> Operand {
        Operand::Signed(self as i64)
    }
}

impl Into<Operand> for i8 {
    fn into(self) -> Operand {
        Operand::Signed(self as i64)
    }
}

impl Into<Operand> for u64 {
    fn into(self) -> Operand {
        Operand::Unsigned(self)
    }
}

impl Into<Operand> for u32 {
    fn into(self) -> Operand {
        Operand::Unsigned(self as u64)
    }
}

impl Into<Operand> for u16 {
    fn into(self) -> Operand {
        Operand::Unsigned(self as u64)
    }
}

impl Into<Operand> for u8 {
    fn into(self) -> Operand {
        Operand::Unsigned(self as u64)
    }
}

impl Into<Operand> for usize {
    fn into(self) -> Operand {
        Operand::Unsigned(self as u64)
    }
}

impl Into<Operand> for f64 {
    fn into(self) -> Operand {
        Operand::Float(self)
    }
}

impl Into<Operand> for f32 {
    fn into(self) -> Operand {
        Operand::Float(self as f64)
    }
}

impl Into<Operand> for Date<Utc> {
    fn into(self) -> Operand {
        Operand::Date(self)
    }
}

impl Into<Operand> for DateTime<Utc> {
    fn into(self) -> Operand {
        Operand::DateTime(self)
    }
}

impl Into<Operand> for DateTime<FixedOffset> {
    fn into(self) -> Operand {
        Operand::DateTime(self.into())
    }
}

fn single_quote_string(s: String) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch == '\\' || ch == '\'' {
            result.push('\\');
        }
        result.push(ch);
    }
    format!("'{}'", result)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn empty_query() {
        let query = Query::default();
        assert_eq!("", query.to_string());
    }

    #[test]
    fn simple_event_type() {
        let query = Query::from(EventType::NewBlock);
        assert_eq!("tm.event = 'NewBlock'", query.to_string());

        let query = Query::from(EventType::Tx);
        assert_eq!("tm.event = 'Tx'", query.to_string());
    }

    #[test]
    fn simple_condition() {
        let query = Query::eq("key", "value");
        assert_eq!("key = 'value'", query.to_string());

        let query = Query::eq("key", 'v');
        assert_eq!("key = 'v'", query.to_string());

        let query = Query::eq("key", "'value'");
        assert_eq!("key = '\\'value\\''", query.to_string());

        let query = Query::eq("key", "\\'value'");
        assert_eq!("key = '\\\\\\'value\\''", query.to_string());

        let query = Query::lt("key", 42_i64);
        assert_eq!("key < 42", query.to_string());

        let query = Query::lt("key", 42_u64);
        assert_eq!("key < 42", query.to_string());

        let query = Query::lte("key", 42_i64);
        assert_eq!("key <= 42", query.to_string());

        let query = Query::gt("key", 42_i64);
        assert_eq!("key > 42", query.to_string());

        let query = Query::gte("key", 42_i64);
        assert_eq!("key >= 42", query.to_string());

        let query = Query::eq("key", 42_u8);
        assert_eq!("key = 42", query.to_string());

        let query = Query::contains("key", "some-substring");
        assert_eq!("key CONTAINS 'some-substring'", query.to_string());

        let query = Query::exists("key");
        assert_eq!("key EXISTS", query.to_string());
    }

    #[test]
    fn date_condition() {
        let query = Query::eq(
            "some_date",
            Date::from_utc(NaiveDate::from_ymd(2020, 9, 24), Utc),
        );
        assert_eq!("some_date = '2020-09-24'", query.to_string());
    }

    #[test]
    fn date_time_condition() {
        let query = Query::eq(
            "some_date_time",
            DateTime::parse_from_rfc3339("2020-09-24T10:17:23-04:00").unwrap(),
        );
        assert_eq!(
            "some_date_time = '2020-09-24T14:17:23+00:00'",
            query.to_string()
        );
    }

    #[test]
    fn complex_query() {
        let query = Query::from(EventType::Tx).and_eq("tx.height", 3_i64);
        assert_eq!("tm.event = 'Tx' AND tx.height = 3", query.to_string());

        let query = Query::from(EventType::Tx)
            .and_lte("tx.height", 100_i64)
            .and_eq("transfer.sender", "AddrA");
        assert_eq!(
            "tm.event = 'Tx' AND tx.height <= 100 AND transfer.sender = 'AddrA'",
            query.to_string()
        );

        let query = Query::from(EventType::Tx)
            .and_lte("tx.height", 100_i64)
            .and_contains("meta.attr", "some-substring");
        assert_eq!(
            "tm.event = 'Tx' AND tx.height <= 100 AND meta.attr CONTAINS 'some-substring'",
            query.to_string()
        );
    }
}
