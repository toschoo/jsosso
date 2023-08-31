/// The json macro provides a domain-specific language
/// to define Json structures in a Json-like syntax.
///
/// Examples:
/// ```
/// use jsosso::*;
///
/// assert_eq!(
///     json!([
///         true, false, null, "hello", "world"
///     ]),
///     Json::Array(vec![
///         Json::Boolean(true),
///         Json::Boolean(false),
///         Json::Null,
///         Json::String("hello".to_string()),
///         Json::String("world".to_string())
///     ])
/// );
///
/// assert_eq!(
///     json!({
///         "Sensor": 4711,
///         "Measurement": 31.123
///     }),
///     Json::Object(Box::new(vec![
///         ("Sensor".to_string(), Json::Number(4711.0)),
///         ("Measurement".to_string(), Json::Number(31.123))
///     ].into_iter().collect()))
/// );
/// ```
///
/// Note that signed numbers need parentheses around them, e.g.:
/// ```
/// use jsosso::*;
///
/// assert_eq!(
///     json!({
///         "Sensor": 4711,
///         "Measurement": (-31.123)
///     }),
///     Json::Object(Box::new(vec![
///         ("Sensor".to_string(), Json::Number(4711.0)),
///         ("Measurement".to_string(), Json::Number(-31.123))
///     ].into_iter().collect()))
/// );
/// ```
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };
    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![
            $( json!($element) ),*
        ])
    };
    ({ $( $key:tt : $value:tt ),* }) => {
        $crate::Json::Object(Box::new(vec![
            $( ($key.to_string(), json!($value)) ),*
          ].into_iter().collect())
        )
    };
    ( $other:tt ) => {
        $crate::Json::from($other)
    };
}
