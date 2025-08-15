use crate::{error::UnrecognizedEnumerationError, utils::AsStaticCow};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    marker::PhantomData,
    str::SplitTerminator,
};

/// Provides a forward compatible wrapper for enumerated string values.
///
/// The intent is that all cases of an enumerated string are captured within `T`; however, in case a
/// new value is added to HLS and this library has not been updated to support it yet, this enum
/// also supports an `Unknown` case that contains a custom string. In this way, parsing won't break
/// as new cases are added to the specification.
///
/// Note, that as long as `T` implements `Into<Cow<str>>`, then `EnumeratedString<T>` also
/// implements `Into<Cow<str>>`. This is a convenience for when setting values on known tags. Also
/// note that most library implementations of `T` will also implement
/// `From<T> for EnumeratedString<T>`, which is another convenience for all methods that take some
/// `impl Into<EnumeratedString<T>>`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnumeratedString<'a, T> {
    /// The value is known to the library and provided by `T`.
    Known(T),
    /// The value is unknown to the library but a reference to the original data is provided.
    Unknown(&'a str),
}
/// Used to provide a convenience accessor on `Option<EnumeratedString<T>>` for the `Known(T)` case.
/// For example:
/// ```
/// # use quick_m3u8::tag::hls::{EnumeratedString, VideoRange};
/// use quick_m3u8::tag::hls::GetKnown;
///
/// let some_enumerated_string: Option<EnumeratedString<VideoRange>> = Some(
///     EnumeratedString::Known(VideoRange::Pq)
/// );
/// let some_video_range = some_enumerated_string.known();
/// assert_eq!(Some(VideoRange::Pq), some_video_range);
/// ```
pub trait GetKnown<T> {
    /// A convenience method for getting the known value. This may be most helpful when chaining on
    /// an already optional attribute.
    fn known(self) -> Option<T>;
}
impl<T> GetKnown<T> for Option<EnumeratedString<'_, T>> {
    fn known(self) -> Option<T> {
        match self {
            Some(EnumeratedString::Known(t)) => Some(t),
            Some(EnumeratedString::Unknown(_)) | None => todo!(),
        }
    }
}
impl<T> EnumeratedString<'_, T> {
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
    T: Display,
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
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>,
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
    T: Into<Cow<'a, str>>,
{
    fn from(value: EnumeratedString<'a, T>) -> Self {
        match value {
            EnumeratedString::Known(t) => t.into(),
            EnumeratedString::Unknown(s) => Cow::Borrowed(s),
        }
    }
}
// If T is AsStaticCow then EnumeratedString can have an as_cow method
impl<'a, T> EnumeratedString<'a, T>
where
    T: AsStaticCow,
{
    /// Provides the inner data as a [`std::borrow::Cow`].
    pub fn as_cow(&self) -> Cow<'a, str> {
        match self {
            EnumeratedString::Unknown(s) => Cow::Borrowed(s),
            EnumeratedString::Known(t) => t.as_cow(),
        }
    }
}

/// Provides a forward compatible wrapper for enumerated string lists.
///
/// [`EnumeratedString`] makes sure that any new enum cases added to the specification will not
/// break parsing. This type extends this concept to lists and provides support for mixed lists of
/// known and unknown values.
///
/// Enumerated string lists are defined in [4.2. Attribute Lists] as:
/// > * enumerated-string-list: a quoted-string containing a comma-separated list of
/// >   enumerated-strings from a set that is explicitly defined by the AttributeName. Each
/// >   enumerated-string in the list is a string consisting of characters valid in an
/// >   enumerated-string. The list SHOULD NOT repeat any enumerated-string. To support forward
/// >   compatibility, clients MUST ignore any unrecognized enumerated-strings in an
/// >   enumerated-string-list.
///
/// The library aims to provide a pseudo-set-like API to working with enumerated-string-lists. You
/// can check for presence of values with [`Self::contains`], if the list is empty with
/// [`Self::is_empty`], add items via [`Self::insert`], and remove via [`Self::remove`]. We also
/// provide an iterator through the members via the [`Self::iter`] method.
///
/// To make lists more convenient to use, most methods where they are needed accept
/// `impl Into<Cow<str>>` as the type, and all library provided `T` allow `EnumeratedStringList<T>`
/// to be `impl Into<Cow<str>>`. Also, `EnumeratedStringList` has implemented `From` on several
/// types that are likely to be used when constructing a list, such as `[S; N]` or `Vec<S>` (where
/// `S` is `Into<EnumeratedString<T>>`). This allows for less boilerplate, and we can sometimes
/// avoid ever even seeing the `EnumeratedString` type, such as the below example of constructing a
/// list of [`crate::tag::hls::Cue`]:
/// ```
/// # use quick_m3u8::tag::hls::EnumeratedStringList;
/// # use quick_m3u8::tag::hls::Cue;
/// let list = EnumeratedStringList::from([Cue::Pre, Cue::Once]);
/// assert_eq!(2, list.iter().count());
/// ```
///
/// Another benefit of this approach is that we are not allocating onto the heap here (as we would
/// if we were using `Vec`), and so this abstraction has little cost over working with the `&str`
/// directly, but provides many convenience properties.
///
/// [4.2. Attribute Lists]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
#[derive(Debug, Clone, PartialEq)]
pub struct EnumeratedStringList<'a, T> {
    inner: Cow<'a, str>,
    t: PhantomData<T>,
}
impl<'a, T> EnumeratedStringList<'a, T>
where
    T: AsStaticCow + Copy + Display,
{
    /// Indicates whether a value is contained within the string list. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::EnumeratedStringList;
    /// use quick_m3u8::tag::hls::Cue;
    ///
    /// let list = EnumeratedStringList::from([Cue::Pre, Cue::Once]);
    /// assert!(list.contains(Cue::Pre));
    /// assert!(list.contains(Cue::Once));
    /// assert!(!list.contains(Cue::Post));
    /// assert!(!list.contains("UNKNOWN"));
    /// ```
    /// Note that most library types used with `EnumeratedString` support converting between
    /// `Cow<str>`, themselves, and `EnumeratedString<T>`. Also, there are many methods that take as
    /// parameter type `impl Into<EnumeratedString<T>>` or `impl Into<Cow<str>>`. These together aim
    /// to make it convenient (less boilerplate) to use enumerations in many places, as can be seen
    /// in the example above, where we did not have to write `EnumeratedString::Known(Cue::Pre)`,
    /// etc. This is also why we were able to use a string slice (`&str`) directly with the
    /// `"UNKNOWN"` example, since `From<str> for EnumeratedString<T>` where `T: TryFrom<&str>`.
    ///
    /// This would be equivalent:
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, EnumeratedString};
    /// # use quick_m3u8::tag::hls::Cue;
    /// # let list = EnumeratedStringList::from([Cue::Pre, Cue::Once]);
    /// assert!(list.contains(EnumeratedString::Known(Cue::Pre)));
    /// assert!(list.contains(EnumeratedString::Known(Cue::Once)));
    /// assert!(!list.contains(EnumeratedString::Known(Cue::Post)));
    /// assert!(!list.contains(EnumeratedString::Unknown("UNKNOWN")));
    /// ```
    ///
    /// And also this:
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, EnumeratedString};
    /// # use quick_m3u8::tag::hls::Cue;
    /// # let list = EnumeratedStringList::from([Cue::Pre, Cue::Once]);
    /// assert!(list.contains("PRE"));
    /// assert!(list.contains("ONCE"));
    /// assert!(!list.contains("POST"));
    /// assert!(!list.contains("UNKNOWN"));
    /// ```
    pub fn contains(&self, value: impl Into<EnumeratedString<'a, T>>) -> bool {
        let value = value.into();
        let value_str = match value {
            EnumeratedString::Unknown(s) => s,
            EnumeratedString::Known(t) => &t.as_cow(),
        };
        self.inner.split(',').any(|item| item == value_str)
    }

    /// Indicates whether the list is empty (i.e. empty string). For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let list = EnumeratedStringList::<Cue>::from("");
    /// assert!(list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Inserts an item into the list. Returns true if the insertion was successful.
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from([Cue::Pre]);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    ///
    /// let insert_succeeded = list.insert(Cue::Once);
    /// assert_eq!(2, list.iter().count());
    /// assert!(insert_succeeded);
    /// assert!(list.contains(Cue::Pre));
    /// assert!(list.contains(Cue::Once));
    /// ```
    /// If an item is already in the list then it does not insert again.
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from([Cue::Pre]);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    ///
    /// let insert_succeeded = list.insert(Cue::Pre);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    /// assert!(!insert_succeeded);
    /// ```
    /// Note that since `value` is any `Into<EnumeratedString<T>>`, and the library provides many
    /// convenience conversions between types, we can also insert unknown values by using `&str`
    /// directly:
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from([Cue::Pre]);
    /// list.insert("UNKNOWN");
    /// assert_eq!(2, list.iter().count());
    /// assert!(list.contains("UNKNOWN"));
    /// ```
    pub fn insert<Item: Into<EnumeratedString<'a, T>>>(&mut self, value: Item) -> bool {
        let value = value.into();
        if self.contains(value) {
            return false;
        }
        let mut new_inner = std::mem::take(&mut self.inner).to_string();
        if !new_inner.is_empty() {
            new_inner.push(',');
        }
        new_inner.push_str(&value.as_cow());
        self.inner = Cow::Owned(new_inner);
        true
    }

    /// Removes an item from the list. Returns true if an item was removed.
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from([Cue::Pre]);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    ///
    /// let remove_succeeded = list.remove(Cue::Pre);
    /// assert_eq!(0, list.iter().count());
    /// assert!(remove_succeeded);
    /// ```
    /// If an item is not in the list then it does not remove anything.
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from([Cue::Pre]);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    ///
    /// let remove_succeeded = list.remove(Cue::Once);
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    /// assert!(!remove_succeeded);
    /// ```
    /// Note that since `value` is any `Into<EnumeratedString<T>>`, and the library provides many
    /// convenience conversions between types, we can also insert unknown values by using `&str`
    /// directly:
    /// ```
    /// # use quick_m3u8::tag::hls::{EnumeratedStringList, Cue};
    /// let mut list = EnumeratedStringList::from(["PRE", "UNKNOWN"]);
    /// assert_eq!(2, list.iter().count());
    /// list.remove("UNKNOWN");
    /// assert_eq!(1, list.iter().count());
    /// assert!(list.contains(Cue::Pre));
    /// ```
    pub fn remove<Item: Into<EnumeratedString<'a, T>>>(&mut self, value: Item) -> bool {
        let value = value.into();
        if !self.contains(value) {
            return false;
        }
        let value = &value.as_cow();
        let mut new_inner = String::new();
        let old_inner = std::mem::take(&mut self.inner).to_string();
        let mut iter = old_inner.split(',');
        match iter.next() {
            Some(v) if v == value => {
                if let Some(next) = iter.next() {
                    new_inner.push_str(next);
                    for item in iter.by_ref() {
                        new_inner.push(',');
                        new_inner.push_str(item);
                    }
                }
            }
            Some(v) => {
                new_inner.push_str(v);
                for item in iter {
                    if item != value {
                        new_inner.push(',');
                        new_inner.push_str(item);
                    }
                }
            }
            None => (), // This isn't really possible since we check for contains above
        }
        self.inner = Cow::Owned(new_inner);
        true
    }

    /// This overrides the default `to_owned` provided as part of `#[derive(Clone)]`.
    ///
    /// The reason this exists is to provide better lifetime semantics by completely breaking ties
    /// to the reference data. This is done by converting the inner into an owned String.
    ///
    /// This method is important as otherwise it won't be possible to take a value from a tag,
    /// mutate the list, and then set it back onto the tag. Providing this method makes that
    /// possible. For example:
    /// ```
    /// # use quick_m3u8::{date_time, tag::hls::{Daterange, Cue, EnumeratedStringList}};
    /// let mut daterange = Daterange::builder()
    ///     .with_id("id")
    ///     .with_start_date(date_time!(2025-08-03 T 00:49:12.000 -05:00))
    ///     .with_cue(EnumeratedStringList::from([Cue::Pre]))
    ///     .finish();
    /// let mut cue = daterange.cue().expect("should be defined");
    /// cue.insert(Cue::Once);
    /// daterange.set_cue(cue.to_owned());
    /// ```
    pub fn to_owned<'b>(&self) -> EnumeratedStringList<'b, T> {
        EnumeratedStringList::from(self.to_string())
    }
}

impl<T> Display for EnumeratedStringList<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl<'a, T> AsRef<str> for EnumeratedStringList<'a, T> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl<'a, T> From<String> for EnumeratedStringList<'a, T> {
    fn from(value: String) -> Self {
        Self {
            inner: Cow::Owned(value),
            t: PhantomData::<T>,
        }
    }
}
impl<'a, T> From<&'a str> for EnumeratedStringList<'a, T> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
            t: PhantomData::<T>,
        }
    }
}
impl<'a, T> From<EnumeratedStringList<'a, T>> for Cow<'a, str> {
    fn from(value: EnumeratedStringList<'a, T>) -> Self {
        value.inner
    }
}
impl<'a, T, S, const N: usize> From<[S; N]> for EnumeratedStringList<'a, T>
where
    T: AsStaticCow + Copy + Display,
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
    T: AsStaticCow + Copy + Display,
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
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>,
{
    /// Creates an [`std::iter::Iterator`] of the [`EnumeratedString`] contents.
    pub fn iter(&'a self) -> EnumeratedStringListIter<'a, T> {
        EnumeratedStringListIter {
            inner: self.inner.split_terminator(','),
            t: PhantomData::<T>,
        }
    }
}

/// An [`std::iter::Iterator`] implementation to allow for iterating through items of an
/// [`EnumeratedStringList`].
#[derive(Debug, Clone)]
pub struct EnumeratedStringListIter<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>,
{
    inner: SplitTerminator<'a, char>,
    t: PhantomData<T>,
}
impl<'a, T> Iterator for EnumeratedStringListIter<'a, T>
where
    T: TryFrom<&'a str, Error = UnrecognizedEnumerationError<'a>>,
{
    type Item = EnumeratedString<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(EnumeratedString::from)
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
            write!(f, "{}", self.as_cow())
        }
    }
    impl AsStaticCow for TestEnum {
        fn as_cow(&self) -> Cow<'static, str> {
            match self {
                TestEnum::One => Cow::Borrowed("ONE"),
                TestEnum::Two => Cow::Borrowed("TWO"),
                TestEnum::Three => Cow::Borrowed("THREE"),
            }
        }
    }
    impl From<TestEnum> for Cow<'_, str> {
        fn from(value: TestEnum) -> Self {
            value.as_cow()
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
    fn enumerated_string_list_is_empty_when_no_values_present() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed(""),
            t: PhantomData::<TestEnum>,
        };
        assert!(list.is_empty(), "list should be empty")
    }

    #[test]
    fn enumerated_string_list_is_not_empty_when_values_present() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE"),
            t: PhantomData::<TestEnum>,
        };
        assert!(!list.is_empty(), "list should not be empty")
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
    fn enumerated_string_iter_has_count_0_when_list_is_empty() {
        let list = EnumeratedStringList {
            inner: Cow::Borrowed(""),
            t: PhantomData::<TestEnum>,
        };
        assert_eq!(0, list.iter().count());
    }

    #[test]
    fn enumerated_string_list_insert_adds_new_value_and_returns_true_if_not_already_present() {
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
    fn enumerated_string_list_insert_doew_not_add_new_value_and_returns_false_if_already_present() {
        let mut list = EnumeratedStringList {
            inner: Cow::Borrowed("ONE,TWO"),
            t: PhantomData::<TestEnum>,
        };

        assert!(!list.insert(TestEnum::Two), "should not have inserted TWO");
        assert_eq!(Cow::Borrowed("ONE,TWO"), list.inner);
    }

    #[test]
    fn enumerated_string_list_remove_removes_value_and_returns_true_if_present() {
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
    fn enumerated_string_list_remove_does_not_remove_value_and_returns_false_if_not_present() {
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

    #[test]
    fn enumerated_string_list_is_empty_should_be_equal_to_iter_count_0() {
        for inner in [" ", ",", " , ", "", ",,", ", ,"] {
            let list = EnumeratedStringList {
                inner: Cow::Borrowed(inner),
                t: PhantomData::<TestEnum>,
            };
            let list_iter = list.iter();
            assert_eq!(
                list.is_empty(),
                list_iter.count() == 0,
                "failed for `{inner}`"
            );
        }
    }
}
