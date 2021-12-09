//! Structured querying for the Tendermint RPC event subscription system.
//!
//! See [`Query`] for details as to how to construct queries.
//!
//! [`Query`]: struct.Query.html

use crate::prelude::*;
use crate::Error;
use core::fmt;
use core::str::FromStr;
use tendermint_proto::serializers::timestamp;
use time::format_description::well_known::Rfc3339;
use time::macros::{format_description, offset};
use time::{Date, OffsetDateTime};

/// A structured query for use in interacting with the Tendermint RPC event
/// subscription system.
///
/// Allows for compile-time validation of queries.
///
/// See the [subscribe endpoint documentation] for more details.
///
/// ## Examples
///
/// ### Direct construction of queries
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
/// let query = Query::from(EventType::Tx).and_gte("tx.height", 100_u64);
/// assert_eq!("tm.event = 'Tx' AND tx.height >= 100", query.to_string());
/// ```
///
/// ### Query parsing
///
/// ```rust
/// use tendermint_rpc::query::{Query, EventType};
///
/// let query: Query = "tm.event = 'NewBlock'".parse().unwrap();
/// assert_eq!(query, Query::from(EventType::NewBlock));
///
/// let query: Query = "tm.event = 'Tx' AND tx.hash = 'XYZ'".parse().unwrap();
/// assert_eq!(query, Query::from(EventType::Tx).and_eq("tx.hash", "XYZ"));
///
/// let query: Query = "tm.event = 'Tx' AND tx.height >= 100".parse().unwrap();
/// assert_eq!(query, Query::from(EventType::Tx).and_gte("tx.height", 100_u64));
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
            conditions: vec![Condition::Eq(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> < <value>`
    pub fn lt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Lt(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> <= <value>`
    pub fn lte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Lte(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> > <value>`
    pub fn gt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Gt(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> >= <value>`
    pub fn gte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Gte(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> CONTAINS <value>` (assuming
    /// `key` contains a string, this tests whether `value` is a sub-string
    /// within it).
    pub fn contains(key: impl ToString, value: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Contains(key.to_string(), value.to_string())],
        }
    }

    /// Query constructor testing whether `<key> EXISTS`.
    pub fn exists(key: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Exists(key.to_string())],
        }
    }

    /// Add the condition `<key> = <value>` to the query.
    pub fn and_eq(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Eq(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> < <value>` to the query.
    pub fn and_lt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Lt(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> <= <value>` to the query.
    pub fn and_lte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Lte(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> > <value>` to the query.
    pub fn and_gt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Gt(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> >= <value>` to the query.
    pub fn and_gte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Gte(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> CONTAINS <value>` to the query.
    pub fn and_contains(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.conditions
            .push(Condition::Contains(key.to_string(), value.to_string()));
        self
    }

    /// Add the condition `<key> EXISTS` to the query.
    pub fn and_exists(mut self, key: impl ToString) -> Self {
        self.conditions.push(Condition::Exists(key.to_string()));
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

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = &self.event_type {
            write!(f, "tm.event = '{}'", t)?;

            if !self.conditions.is_empty() {
                write!(f, " AND ")?;
            }
        }

        join(f, " AND ", &self.conditions)?;

        Ok(())
    }
}

peg::parser! {
    grammar query_parser() for str {
        // Some or no whitespace.
        rule _() = quiet!{[' ']*}

        // At least some whitespace.
        rule __() = quiet!{[' ']+}

        rule string() -> &'input str
            = "'" s:$([^'\'']*) "'" { s }

        rule unsigned() -> u64
            = s:$(['0'..='9']+) {?
                u64::from_str(s)
                    .map_err(|_| "failed to parse as an unsigned integer")
            }

        rule signed() -> i64
            = s:$("-" ['1'..='9'] ['0'..='9']*) {?
                i64::from_str(s)
                    .map_err(|_| "failed to parse as a signed integer")
            }

        rule year() -> &'input str
            = $(['0'..='9']*<4>)

        rule month() -> &'input str
            = $(['0' | '1'] ['0'..='9'])

        rule day() -> &'input str
            = $(['0'..='3'] ['0'..='9'])

        rule date() -> &'input str
            = $(year() "-" month() "-" day())

        rule hour() -> &'input str
            = $(['0'..='2'] ['0'..='9'])

        rule min_sec() -> &'input str
            = $(['0'..='5'] ['0'..='9'])

        rule nanosec() -> &'input str
            = $("." ['0'..='9']+)

        rule time() -> &'input str
            = $(hour() ":" min_sec() ":" min_sec() nanosec()? "Z")

        rule datetime() -> &'input str
            = dt:$(date() "T" time()) { dt }

        rule float() -> f64
            = s:$("-"? ['0'..='9']+ "." ['0'..='9']+) {?
                f64::from_str(s)
                    .map_err(|_| "failed to parse as a 64-bit floating point number")
            }

        rule string_op() -> Operand
            = s:string() { Operand::String(s.to_owned()) }

        rule unsigned_op() -> Operand
            = u:unsigned() { Operand::Unsigned(u) }

        rule signed_op() -> Operand
            = s:signed() { Operand::Signed(s) }

        rule datetime_op() -> Operand
            = "TIME" __ dt:datetime() {?
                OffsetDateTime::parse(dt, &Rfc3339)
                    .map(|dt| Operand::DateTime(dt.to_offset(offset!(UTC))))
                    .map_err(|_| "failed to parse as RFC3339-compatible date/time")
            }

        rule date_op() -> Operand
            = "DATE" __ dt:date() {?
                let date = Date::parse(dt, &format_description!("[year]-[month]-[day]"))
                    .map_err(|_| "failed to parse as RFC3339-compatible date")?;
                Ok(Operand::Date(date))
            }

        rule float_op() -> Operand
            = f:float() { Operand::Float(f) }

        rule tag() -> &'input str
            = $(['a'..='z' | 'A'..='Z'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.']*)

        rule operand() -> Operand
            = datetime_op() / date_op() / string_op() / float_op() / signed_op() / unsigned_op()

        rule eq() -> Condition
            = t:tag() _ "=" _ op:operand() { Condition::Eq(t.to_owned(), op) }

        rule lte() -> Condition
            = t:tag() _ "<=" _ op:operand() { Condition::Lte(t.to_owned(), op) }

        rule lt() -> Condition
            = t:tag() _ "<" _ op:operand() { Condition::Lt(t.to_owned(), op) }

        rule gte() -> Condition
            = t:tag() _ ">=" _ op:operand() { Condition::Gte(t.to_owned(), op) }

        rule gt() -> Condition
            = t:tag() _ ">" _ op:operand() { Condition::Gt(t.to_owned(), op) }

        rule contains() -> Condition
            = t:tag() __ "CONTAINS" __ op:string() { Condition::Contains(t.to_owned(), op.to_owned()) }

        rule exists() -> Condition
            = t:tag() __ "EXISTS" { Condition::Exists(t.to_owned()) }

        rule event_type() -> Term
            = "tm.event" _ "=" _ "'" et:$("NewBlock" / "Tx") "'" {
                Term::EventType(EventType::from_str(et).unwrap())
            }

        rule condition() -> Term
            = c:(eq() / lte() / lt() / gte() / gt() / contains() / exists()) { Term::Condition(c) }

        rule term() -> Term
            = event_type() / condition()

        pub rule query() -> Vec<Term>
            = t:term() ** ( __ "AND" __ ) { t }
    }
}

/// A term in a query is either an event type or a general condition.
/// Exclusively used for query parsing.
#[derive(Debug)]
pub enum Term {
    EventType(EventType),
    Condition(Condition),
}

// Separate a list of terms into lists of each type of term.
fn separate_terms(terms: Vec<Term>) -> (Vec<EventType>, Vec<Condition>) {
    terms
        .into_iter()
        .fold((Vec::new(), Vec::new()), |mut v, t| {
            match t {
                Term::EventType(et) => v.0.push(et),
                Term::Condition(c) => v.1.push(c),
            }
            v
        })
}

impl FromStr for Query {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (event_types, conditions) = separate_terms(
            query_parser::query(s)
                .map_err(|e| Error::invalid_params(format!("failed to parse query: {}", e)))?,
        );
        if event_types.len() > 1 {
            return Err(Error::invalid_params(
                "tm.event can only be used once in a query".to_owned(),
            ));
        }
        Ok(Query {
            event_type: event_types.first().cloned(),
            conditions,
        })
    }
}

fn join<S, I>(f: &mut fmt::Formatter<'_>, separator: S, iterable: I) -> fmt::Result
where
    S: fmt::Display,
    I: IntoIterator,
    I::Item: fmt::Display,
{
    let mut iter = iterable.into_iter();
    if let Some(first) = iter.next() {
        write!(f, "{}", first)?;
    }

    for item in iter {
        write!(f, "{}{}", separator, item)?;
    }

    Ok(())
}

/// The types of Tendermint events for which we can query at present.
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    NewBlock,
    Tx,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::NewBlock => write!(f, "NewBlock"),
            EventType::Tx => write!(f, "Tx"),
        }
    }
}

impl FromStr for EventType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "NewBlock" => Ok(Self::NewBlock),
            "Tx" => Ok(Self::Tx),
            invalid => Err(Error::unrecognized_event_type(invalid.to_string())),
        }
    }
}

/// The different types of conditions supported by a [`Query`].
///
/// [`Query`]: struct.Query.html
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Equals
    Eq(String, Operand),
    /// Less than
    Lt(String, Operand),
    /// Less than or equal to
    Lte(String, Operand),
    /// Greater than
    Gt(String, Operand),
    /// Greater than or equal to
    Gte(String, Operand),
    /// Contains (to check if a key contains a certain sub-string)
    Contains(String, String),
    /// Exists (to check if a key exists)
    Exists(String),
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Eq(key, op) => write!(f, "{} = {}", key, op),
            Condition::Lt(key, op) => write!(f, "{} < {}", key, op),
            Condition::Lte(key, op) => write!(f, "{} <= {}", key, op),
            Condition::Gt(key, op) => write!(f, "{} > {}", key, op),
            Condition::Gte(key, op) => write!(f, "{} >= {}", key, op),
            Condition::Contains(key, op) => write!(f, "{} CONTAINS {}", key, escape(op)),
            Condition::Exists(key) => write!(f, "{} EXISTS", key),
        }
    }
}

/// A typed operand for use in an [`Condition`].
///
/// According to the [Tendermint RPC subscribe docs][tm-subscribe],
/// an operand can be a string, number, date or time. We differentiate here
/// between integer and floating point numbers.
///
/// [`Condition`]: enum.Condition.html
/// [tm-subscribe]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Date(Date),
    DateTime(OffsetDateTime),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::String(s) => write!(f, "{}", escape(s)),
            Operand::Signed(i) => write!(f, "{}", i),
            Operand::Unsigned(u) => write!(f, "{}", u),
            Operand::Float(h) => write!(f, "{}", h),
            Operand::Date(d) => {
                write!(f, "DATE ")?;
                fmt_date(*d, f)?;
                Ok(())
            }
            Operand::DateTime(dt) => {
                write!(f, "TIME ")?;
                timestamp::fmt_as_rfc3339_nanos(*dt, f)?;
                Ok(())
            }
        }
    }
}

fn fmt_date(d: Date, mut f: impl fmt::Write) -> fmt::Result {
    write!(f, "{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

impl From<String> for Operand {
    fn from(source: String) -> Self {
        Operand::String(source)
    }
}

impl From<char> for Operand {
    fn from(source: char) -> Self {
        Operand::String(source.to_string())
    }
}

impl From<&str> for Operand {
    fn from(source: &str) -> Self {
        Operand::String(source.to_string())
    }
}

impl From<i64> for Operand {
    fn from(source: i64) -> Self {
        Operand::Signed(source)
    }
}

impl From<i32> for Operand {
    fn from(source: i32) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<i16> for Operand {
    fn from(source: i16) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<i8> for Operand {
    fn from(source: i8) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<u64> for Operand {
    fn from(source: u64) -> Self {
        Operand::Unsigned(source)
    }
}

impl From<u32> for Operand {
    fn from(source: u32) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<u16> for Operand {
    fn from(source: u16) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<u8> for Operand {
    fn from(source: u8) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<usize> for Operand {
    fn from(source: usize) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<f64> for Operand {
    fn from(source: f64) -> Self {
        Operand::Float(source)
    }
}

impl From<f32> for Operand {
    fn from(source: f32) -> Self {
        Operand::Float(source as f64)
    }
}

impl From<Date> for Operand {
    fn from(source: Date) -> Self {
        Operand::Date(source)
    }
}

impl From<OffsetDateTime> for Operand {
    fn from(source: OffsetDateTime) -> Self {
        Operand::DateTime(source.to_offset(offset!(UTC)))
    }
}

/// Escape backslashes and single quotes within the given string with a backslash.
fn escape(s: &str) -> String {
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
    use time::macros::{date, datetime};

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
        let query = Query::eq("some_date", date!(2020 - 09 - 24));
        assert_eq!("some_date = DATE 2020-09-24", query.to_string());
    }

    #[test]
    fn date_time_condition() {
        let query = Query::eq("some_date_time", datetime!(2020-09-24 10:17:23 -04:00));
        assert_eq!(
            "some_date_time = TIME 2020-09-24T14:17:23Z",
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

    #[test]
    fn query_event_type_parsing() {
        // Test the empty query (that matches all possible events)
        let query = Query::from_str("").unwrap();
        assert_eq!(query, Query::default());

        // With just one event type
        let query = Query::from_str("tm.event='Tx'").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert!(query.conditions.is_empty());
        let query = Query::from_str("tm.event='NewBlock'").unwrap();
        assert_eq!(query.event_type, Some(EventType::NewBlock));
        assert!(query.conditions.is_empty());

        // One event type, with whitespace
        let query = Query::from_str("tm.event =  'NewBlock'").unwrap();
        assert_eq!(query.event_type, Some(EventType::NewBlock));
        assert!(query.conditions.is_empty());

        // Two event types are not allowed
        assert!(Query::from_str("tm.event='Tx' AND tm.event='NewBlock'").is_err());
    }

    #[test]
    fn query_string_term_parsing() {
        // Query with string term
        let query = Query::from_str("tm.event='Tx' AND transfer.sender='AddrA'").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Eq(
                "transfer.sender".to_owned(),
                Operand::String("AddrA".to_owned()),
            )]
        );
        // Query with string term, with extra whitespace
        let query = Query::from_str("tm.event = 'Tx' AND transfer.sender = 'AddrA'").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Eq(
                "transfer.sender".to_owned(),
                Operand::String("AddrA".to_owned()),
            )]
        );
    }

    #[test]
    fn query_unsigned_term_parsing() {
        let query = Query::from_str("tm.event = 'Tx' AND tx.height = 10").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Eq("tx.height".to_owned(), Operand::Unsigned(10))]
        );

        let query = Query::from_str("tm.event = 'Tx' AND tx.height <= 100").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Lte(
                "tx.height".to_owned(),
                Operand::Unsigned(100)
            )]
        );
    }

    #[test]
    fn query_signed_term_parsing() {
        let query = Query::from_str("tm.event = 'Tx' AND some.value = -1").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Eq("some.value".to_owned(), Operand::Signed(-1))]
        );

        let query = Query::from_str("tm.event = 'Tx' AND some.value <= -100").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Lte(
                "some.value".to_owned(),
                Operand::Signed(-100)
            )]
        );
    }

    #[test]
    fn query_date_parsing() {
        let query = Query::from_str("tm.event = 'Tx' AND some.date <= DATE 2022-02-03").unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Lte(
                "some.date".to_owned(),
                Operand::Date(date!(2022 - 2 - 3))
            )]
        );
    }

    #[test]
    fn query_datetime_parsing() {
        let query =
            Query::from_str("tm.event = 'Tx' AND some.datetime = TIME 2021-02-26T17:05:02.1495Z")
                .unwrap();
        assert_eq!(query.event_type, Some(EventType::Tx));
        assert_eq!(
            query.conditions,
            vec![Condition::Eq(
                "some.datetime".to_owned(),
                Operand::DateTime(datetime!(2021-2-26 17:05:02.149500000 UTC))
            )]
        )
    }

    #[test]
    fn query_float_parsing() {
        // Positive floating point number
        let query = Query::from_str("short.pi = 3.14159").unwrap();
        assert_eq!(query.conditions.len(), 1);
        match &query.conditions[0] {
            Condition::Eq(tag, op) => {
                assert_eq!(tag, "short.pi");
                match op {
                    Operand::Float(f) => {
                        assert!(floats_eq(*f, core::f64::consts::PI, 5));
                    }
                    _ => panic!("unexpected operand: {:?}", op),
                }
            }
            c => panic!("unexpected condition: {:?}", c),
        }

        // Negative floating point number
        let query = Query::from_str("short.pi = -3.14159").unwrap();
        assert_eq!(query.conditions.len(), 1);
        match &query.conditions[0] {
            Condition::Eq(tag, op) => {
                assert_eq!(tag, "short.pi");
                match op {
                    Operand::Float(f) => {
                        assert!(floats_eq(*f, -core::f64::consts::PI, 5));
                    }
                    _ => panic!("unexpected operand: {:?}", op),
                }
            }
            c => panic!("unexpected condition: {:?}", c),
        }
    }

    // From https://stackoverflow.com/a/41447964/1156132
    fn floats_eq(a: f64, b: f64, precision: u8) -> bool {
        let factor = 10.0f64.powi(precision as i32);
        let a = (a * factor).trunc();
        let b = (b * factor).trunc();
        a == b
    }

    #[test]
    fn query_conditions() {
        let query = Query::from_str("some.field = 'string'").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Eq(
                    "some.field".to_owned(),
                    Operand::String("string".to_owned())
                )]
            }
        );

        let query = Query::from_str("some.field < 5").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Lt("some.field".to_owned(), Operand::Unsigned(5),)]
            }
        );

        let query = Query::from_str("some.field <= 5").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Lte(
                    "some.field".to_owned(),
                    Operand::Unsigned(5),
                )]
            }
        );

        let query = Query::from_str("some.field > 5").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Gt("some.field".to_owned(), Operand::Unsigned(5),)]
            }
        );

        let query = Query::from_str("some.field >= 5").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Gte(
                    "some.field".to_owned(),
                    Operand::Unsigned(5),
                )]
            }
        );

        let query = Query::from_str("some.field CONTAINS 'inner'").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Contains(
                    "some.field".to_owned(),
                    "inner".to_owned()
                )]
            }
        );

        let query = Query::from_str("some.field EXISTS").unwrap();
        assert_eq!(
            query,
            Query {
                event_type: None,
                conditions: vec![Condition::Exists("some.field".to_owned())]
            }
        );
    }
}
