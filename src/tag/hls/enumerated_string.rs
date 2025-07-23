use crate::{error::UnrecognizedEnumerationError, utils::AsStaticStr};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    marker::PhantomData,
    str::Split,
    usize,
};

/// Provides a forward compatible wrapper for enumerated string values.
///
/// The intent is that all cases of an enumerated string are captured within `T`; however, in case a
/// new value is added to HLS and this library has not been updated to support it yet, this enum
/// also supports an `Unknown` case that contains a custom string. In this way, parsing won't break
/// as new cases are added to the specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnumeratedString<'a, T>
where
    T: Clone + Copy + PartialEq + Debug + Display,
{
    Known(T),
    Unknown(&'a str),
}
impl<T> EnumeratedString<'_, T>
where
    T: Clone + Copy + PartialEq + Debug + Display,
{
    /// A convenience method for getting the known value. This may be most helpful when chaining on
    /// an already optional attribute.
    pub fn known(&self) -> Option<&T> {
        match self {
            Self::Known(t) => Some(t),
            Self::Unknown(_) => None,
        }
    }
}
// Display is needed for writing mutated values to output
impl<T> Display for EnumeratedString<'_, T>
where
    T: Clone + Copy + PartialEq + Debug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known(t) => Display::fmt(&t, f),
            Self::Unknown(s) => Display::fmt(&s, f),
        }
    }
}
// Convenience From implementation for where T has implemented TryFrom
impl<'a, T> From<&'a str> for EnumeratedString<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>
        + Clone
        + Copy
        + PartialEq
        + Debug
        + Display,
{
    fn from(value: &'a str) -> Self {
        match T::try_from(value) {
            Ok(known) => Self::Known(known),
            Err(_) => Self::Unknown(value),
        }
    }
}
// Convenience Into implementation for where Cow has From T
impl<'a, T> From<EnumeratedString<'a, T>> for Cow<'a, str>
where
    T: Into<Cow<'a, str>> + Clone + Copy + PartialEq + Debug + Display,
{
    fn from(value: EnumeratedString<'a, T>) -> Self {
        match value {
            EnumeratedString::Known(t) => t.into(),
            EnumeratedString::Unknown(s) => Cow::Borrowed(s),
        }
    }
}
// If T is AsStaticStr then EnumeratedString can have an as_str method
impl<'a, T> EnumeratedString<'a, T>
where
    T: Clone + Copy + PartialEq + Debug + Display + AsStaticStr,
{
    pub fn as_str(&self) -> &'a str {
        match self {
            EnumeratedString::Unknown(s) => s,
            EnumeratedString::Known(t) => t.as_str(),
        }
    }
}

/// Provides a forward compatible wrapper for enumerated string lists.
///
/// [`EnumeratedString`] makes sure that any new enum cases added to the specification will not
/// break parsing. This type extends this concept to lists and provides support for mixed lists of
/// known and unknown values.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    inner: Cow<'a, str>,
    t: PhantomData<T>,
}
impl<'a, T> EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    pub fn contains(&self, value: impl Into<EnumeratedString<'a, T>>) -> bool {
        let value = value.into();
        let value_str = match value {
            EnumeratedString::Unknown(s) => s,
            EnumeratedString::Known(t) => t.as_str(),
        };
        self.inner
            .split(',')
            .find(|item| *item == value_str)
            .is_some()
    }

    pub fn insert<Item: Into<EnumeratedString<'a, T>>>(&mut self, value: Item) -> bool {
        let value = value.into();
        if self.contains(value) {
            return false;
        }
        let mut new_inner = std::mem::take(&mut self.inner).to_string();
        if !new_inner.is_empty() {
            new_inner.push_str(",");
        }
        new_inner.push_str(value.as_str());
        self.inner = Cow::Owned(new_inner);
        true
    }

    pub fn remove<Item: Into<EnumeratedString<'a, T>>>(&mut self, value: Item) -> bool {
        let value = value.into();
        if !self.contains(value) {
            return false;
        }
        let value = value.as_str();
        let mut new_inner = String::new();
        let old_inner = std::mem::take(&mut self.inner).to_string();
        let mut iter = old_inner.split(',');
        match iter.next() {
            Some(v) if v == value => {
                if let Some(next) = iter.next() {
                    new_inner.push_str(next);
                    while let Some(item) = iter.next() {
                        new_inner.push_str(",");
                        new_inner.push_str(item);
                    }
                }
            }
            Some(v) => {
                new_inner.push_str(v);
                while let Some(item) = iter.next() {
                    if item != value {
                        new_inner.push_str(",");
                        new_inner.push_str(item);
                    }
                }
            }
            None => (), // This isn't really possible since we check for contains above
        }
        std::mem::swap(&mut self.inner, &mut Cow::Owned(new_inner));
        true
    }

    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    pub fn to_owned<'b>(&self) -> EnumeratedStringList<'b, T> {
        EnumeratedStringList::from(self.to_string())
    }
}
impl<'a, T> AsRef<str> for EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl<'a, T> From<String> for EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    fn from(value: String) -> Self {
        Self {
            inner: Cow::Owned(value),
            t: PhantomData::<T>,
        }
    }
}
impl<'a, T> From<&'a str> for EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    fn from(value: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
            t: PhantomData::<T>,
        }
    }
}
impl<'a, T> From<EnumeratedStringList<'a, T>> for Cow<'a, str>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
{
    fn from(value: EnumeratedStringList<'a, T>) -> Self {
        value.inner
    }
}
impl<'a, T, S, const N: usize> From<[S; N]> for EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
    S: Into<EnumeratedString<'a, T>>,
{
    fn from(value: [S; N]) -> Self {
        let mut list = EnumeratedStringList::from("");
        for item in value {
            list.insert(item.into());
        }
        list
    }
}
impl<'a, T, S> From<Vec<S>> for EnumeratedStringList<'a, T>
where
    T: AsStaticStr + Clone + Copy + PartialEq + Debug + Display,
    S: Into<EnumeratedString<'a, T>>,
{
    fn from(value: Vec<S>) -> Self {
        let mut list = EnumeratedStringList::from("");
        for item in value {
            list.insert(item.into());
        }
        list
    }
}

impl<'a, T> EnumeratedStringList<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>
        + AsStaticStr
        + Clone
        + Copy
        + PartialEq
        + Debug
        + Display,
{
    pub fn iter(&'a self) -> EnumeratedStringListIter<'a, T> {
        EnumeratedStringListIter {
            inner: self.inner.split(','),
            t: PhantomData::<T>,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumeratedStringListIter<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>
        + AsStaticStr
        + Clone
        + Copy
        + PartialEq
        + Debug
        + Display,
{
    inner: Split<'a, char>,
    t: PhantomData<T>,
}
impl<'a, T> Iterator for EnumeratedStringListIter<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>
        + AsStaticStr
        + Clone
        + Copy
        + PartialEq
        + Debug
        + Display,
{
    type Item = EnumeratedString<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| EnumeratedString::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestEnum {
        One,
        Two,
        Three,
    }
    impl<'a> TryFrom<&'a str> for TestEnum {
        type Error = UnrecognizedEnumerationError<'a>;
        fn try_from(value: &'a str) -> Result<Self, Self::Error> {
            match value {
                "ONE" => Ok(Self::One),
                "TWO" => Ok(Self::Two),
                "THREE" => Ok(Self::Three),
                _ => Err(UnrecognizedEnumerationError::new(value)),
            }
        }
    }
    impl Display for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }
    impl AsStaticStr for TestEnum {
        fn as_str(&self) -> &'static str {
            match self {
                TestEnum::One => "ONE",
                TestEnum::Two => "TWO",
                TestEnum::Three => "THREE",
            }
        }
    }
    impl From<TestEnum> for Cow<'_, str> {
        fn from(value: TestEnum) -> Self {
            Cow::Borrowed(value.as_str())
        }
    }
    impl From<TestEnum> for EnumeratedString<'_, TestEnum> {
        fn from(value: TestEnum) -> Self {
            Self::Known(value)
        }
    }

    #[test]
    fn enumerated_string_fmt_correctly() {
        assert_eq!("ONE", format!("{}", EnumeratedString::Known(TestEnum::One)));
        assert_eq!("TWO", format!("{}", EnumeratedString::Known(TestEnum::Two)));
        assert_eq!(
            "THREE",
            format!("{}", EnumeratedString::Known(TestEnum::Three))
        );
        assert_eq!(
            "CUSTOM",
            format!("{}", EnumeratedString::<TestEnum>::Unknown("CUSTOM"))
        );
    }

    #[test]
    fn enumerated_string_from_any_str() {
        assert_eq!(
            EnumeratedString::Known(TestEnum::One),
            EnumeratedString::from("ONE")
        );
        assert_eq!(
            EnumeratedString::Known(TestEnum::Two),
            EnumeratedString::from("TWO")
        );
        assert_eq!(
            EnumeratedString::Known(TestEnum::Three),
            EnumeratedString::from("THREE")
        );
        assert_eq!(
            EnumeratedString::<TestEnum>::Unknown("CUSTOM"),
            EnumeratedString::from("CUSTOM")
        );
    }

    #[test]
    fn enumerated_string_list_contains_true_when_value_in_list() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.contains(TestEnum::One), "list should contain ONE");
        assert!(list.contains(TestEnum::Two), "list should contain TWO");
    }

    #[test]
    fn enumerated_string_list_contains_false_when_value_not_in_list() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };
        assert!(
            !list.contains(TestEnum::Three),
            "list should not contain THREE"
        );
        assert!(!list.contains(EnumeratedString::Unknown("UNKNOWN")));
    }

    #[test]
    fn enumerated_string_iter_provides_all_values() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO,CUSTOM"),
            t: PhantomData::<TestEnum>,
        };
        let mut count = 0;
        for item in list.iter() {
            count += 1;
            match count {
                1 => assert_eq!(EnumeratedString::Known(TestEnum::One), item),
                2 => assert_eq!(EnumeratedString::Known(TestEnum::Two), item),
                3 => assert_eq!(EnumeratedString::Unknown("CUSTOM"), item),
                n => panic!("unexpected item number {n} in iterable"),
            }
        }
    }

    #[test]
    fn enumerated_string_iter_insert_adds_new_value_and_returns_true_if_not_already_present() {
        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.insert(TestEnum::Three), "should have inserted THREE");
        assert_eq!(Cow::<str>::Owned("ONE,TWO,THREE".to_string()), list.inner);

        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed(""),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.insert(TestEnum::Three), "should have inserted THREE");
        assert_eq!(Cow::<str>::Owned("THREE".to_string()), list.inner);
    }

    #[test]
    fn enumerated_string_iter_insert_doew_not_add_new_value_and_returns_false_if_already_present() {
        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };

        assert!(!list.insert(TestEnum::Two), "should not have inserted TWO");
        assert_eq!(Cow::Borrowed("ONE,TWO"), list.inner);
    }

    #[test]
    fn enumerated_string_iter_remove_removes_value_and_returns_true_if_present() {
        // Run the test a few times with comma in different places
        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.remove(TestEnum::Two), "should have removed TWO");
        assert_eq!(Cow::<str>::Owned("ONE".to_string()), list.inner);

        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO,THREE"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.remove(TestEnum::Two), "should have removed TWO");
        assert_eq!(Cow::<str>::Owned("ONE,THREE".to_string()), list.inner);

        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("TWO,THREE"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.remove(TestEnum::Two), "should have removed TWO");
        assert_eq!(Cow::<str>::Owned("THREE".to_string()), list.inner);

        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("TWO"),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.remove(TestEnum::Two), "should have removed TWO");
        assert_eq!(Cow::<str>::Owned("".to_string()), list.inner);
    }

    #[test]
    fn enumerated_string_iter_remove_does_not_remove_value_and_returns_false_if_not_present() {
        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };

        assert!(
            !list.remove(TestEnum::Three),
            "should not have removed THREE"
        );
        assert_eq!(Cow::Borrowed("ONE,TWO"), list.inner);
    }
}
