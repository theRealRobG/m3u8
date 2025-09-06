use crate::{
    date::{self, DateTime},
    error::{ParseTagValueError, UnrecognizedEnumerationError, ValidationError},
    tag::{
        AttributeValue, UnknownTag, UnquotedAttributeValue,
        hls::{EnumeratedString, EnumeratedStringList, LazyAttribute, TagName, into_inner_tag},
    },
    utils::AsStaticCow,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

/// Corresponds to the `#EXT-X-DATERANGE:CUE` attribute.
///
/// See [`Daterange`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue {
    /// Indicates that an action is to be triggered before playback of the primary asset begins,
    /// regardless of where playback begins in the primary asset.
    Pre,
    /// Indicates that an action is to be triggered after the primary asset has been played to its
    /// end without error.
    Post,
    /// Indicates that an action is to be triggered once. It SHOULD NOT be triggered again, even if
    /// the user replays the portion of the primary asset that includes the trigger point.
    Once,
}
impl<'a> TryFrom<&'a str> for Cue {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            PRE => Ok(Self::Pre),
            POST => Ok(Self::Post),
            ONCE => Ok(Self::Once),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Cue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Cue {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Cue::Pre => Cow::Borrowed(PRE),
            Cue::Post => Cow::Borrowed(POST),
            Cue::Once => Cow::Borrowed(ONCE),
        }
    }
}
impl From<Cue> for Cow<'_, str> {
    fn from(value: Cue) -> Self {
        value.as_cow()
    }
}
impl From<Cue> for EnumeratedString<'_, Cue> {
    fn from(value: Cue) -> Self {
        Self::Known(value)
    }
}
const PRE: &str = "PRE";
const POST: &str = "POST";
const ONCE: &str = "ONCE";

/// Corresponds to the `#EXT-X-DATERANGE:X-SNAP` attribute defined in the
/// `com.apple.hls.interstitial` extension attributes defined in [Appendix D].
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Snap {
    /// If the list contains OUT then the client SHOULD locate the segment boundary closest to the
    /// START-DATE of the interstitial in the Media Playlist of the primary content and transition
    /// to the interstitial at that boundary. If more than one Media Playlist is contributing to
    /// playback (audio plus video for example), the client SHOULD transition at the earliest
    /// segment boundary.
    Out,
    /// If the list contains IN then the client SHOULD locate the segment boundary closest to the
    /// scheduled resumption point from the interstitial in the Media Playlist of the primary
    /// content and resume playback of primary content at that boundary. If more than one Media
    /// Playlist is contributing to playback, the client SHOULD transition at the latest segment
    /// boundary.
    In,
}
impl<'a> TryFrom<&'a str> for Snap {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            OUT => Ok(Self::Out),
            IN => Ok(Self::In),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Snap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Snap {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Out => Cow::Borrowed(OUT),
            Self::In => Cow::Borrowed(IN),
        }
    }
}
impl From<Snap> for Cow<'_, str> {
    fn from(value: Snap) -> Self {
        value.as_cow()
    }
}
impl From<Snap> for EnumeratedString<'_, Snap> {
    fn from(value: Snap) -> Self {
        Self::Known(value)
    }
}
const OUT: &str = "OUT";
const IN: &str = "IN";

/// Corresponds to the `#EXT-X-DATERANGE:X-RESTRICT` attribute defined in the
/// `com.apple.hls.interstitial` extension attributes defined in [Appendix D].
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Restrict {
    /// If the list contains SKIP then while the interstitial is being played, the client MUST NOT
    /// allow the user to seek forward from the current playhead position or set the rate to greater
    /// than the regular playback rate until playback reaches the end of the interstitial.
    Skip,
    /// If the list contains JUMP then the client MUST NOT allow the user to seek from a position in
    /// the primary asset earlier than the START-DATE attribute to a position after it without first
    /// playing the interstitial asset, even if the interstitial at START-DATE was played through
    /// earlier. If the user attempts to seek across more than one interstitial, the client SHOULD
    /// choose at least one interstitial to play before allowing the seek to complete.
    Jump,
}
impl<'a> TryFrom<&'a str> for Restrict {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            SKIP => Ok(Self::Skip),
            JUMP => Ok(Self::Jump),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Restrict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Restrict {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Skip => Cow::Borrowed(SKIP),
            Self::Jump => Cow::Borrowed(JUMP),
        }
    }
}
impl From<Restrict> for Cow<'_, str> {
    fn from(value: Restrict) -> Self {
        value.as_cow()
    }
}
impl From<Restrict> for EnumeratedString<'_, Restrict> {
    fn from(value: Restrict) -> Self {
        Self::Known(value)
    }
}
const SKIP: &str = "SKIP";
const JUMP: &str = "JUMP";

/// Corresponds to the `#EXT-X-DATERANGE:X-TIMELINE-OCCUPIES` attribute defined in the
/// `com.apple.hls.interstitial` extension attributes defined in [Appendix D].
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineOccupies {
    /// Indicates that the interstitial should be represented in a timeline UI as a single point.
    Point,
    /// Indicates that the interstitial should be represented in a timeline UI as a range.
    Range,
}
impl<'a> TryFrom<&'a str> for TimelineOccupies {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            POINT => Ok(Self::Point),
            RANGE => Ok(Self::Range),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for TimelineOccupies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for TimelineOccupies {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Point => Cow::Borrowed(POINT),
            Self::Range => Cow::Borrowed(RANGE),
        }
    }
}
impl From<TimelineOccupies> for Cow<'_, str> {
    fn from(value: TimelineOccupies) -> Self {
        value.as_cow()
    }
}
impl From<TimelineOccupies> for EnumeratedString<'_, TimelineOccupies> {
    fn from(value: TimelineOccupies) -> Self {
        Self::Known(value)
    }
}
const POINT: &str = "POINT";
const RANGE: &str = "RANGE";

/// Corresponds to the `#EXT-X-DATERANGE:X-TIMELINE-STYLE` attribute defined in the
/// `com.apple.hls.interstitial` extension attributes defined in [Appendix D].
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineStyle {
    /// Indicates that the interstitial is intended to be presented in a timeline UI as being
    /// distinct from primary content.
    Highlight,
    /// Indicates that the interstitial is intended to be presented in a timeline UI as not
    /// differentiated from primary content.
    Primary,
}
impl<'a> TryFrom<&'a str> for TimelineStyle {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            HIGHLIGHT => Ok(Self::Highlight),
            PRIMARY => Ok(Self::Primary),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for TimelineStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for TimelineStyle {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Highlight => Cow::Borrowed(HIGHLIGHT),
            Self::Primary => Cow::Borrowed(PRIMARY),
        }
    }
}
impl From<TimelineStyle> for Cow<'_, str> {
    fn from(value: TimelineStyle) -> Self {
        value.as_cow()
    }
}
impl From<TimelineStyle> for EnumeratedString<'_, TimelineStyle> {
    fn from(value: TimelineStyle) -> Self {
        Self::Known(value)
    }
}
const HIGHLIGHT: &str = "HIGHLIGHT";
const PRIMARY: &str = "PRIMARY";

/// The value of the `EXT-X-DATERANGE:CLASS` attribute that indicates that the daterange should be
/// treated as per the definitions within [Interstitials].
///
/// [Interstitials]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
pub const INTERSTITIAL_CLASS: &str = "com.apple.hls.interstitial";
/// Corresponds to the attributes defined for HLS Interstitials in [Appnedix D].
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, Clone)]
pub struct InterstitialExtensionAttributes<'a, 'b> {
    asset_uri: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    asset_list: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    resume_offset: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    playout_limit: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    snap: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    restrict: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    content_may_vary: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    timeline_occupies: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    timeline_style: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    skip_control_offset: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    skip_control_duration: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
    skip_control_label_id: &'b LazyAttribute<'a, ExtensionAttributeValue<'a>>,
}
impl<'a, 'b> PartialEq for InterstitialExtensionAttributes<'a, 'b> {
    fn eq(&self, other: &Self) -> bool {
        self.asset_uri() == other.asset_uri()
            && self.asset_list() == other.asset_list()
            && self.resume_offset() == other.resume_offset()
            && self.playout_limit() == other.playout_limit()
            && self.snap() == other.snap()
            && self.restrict() == other.restrict()
            && self.content_may_vary() == other.content_may_vary()
            && self.timeline_occupies() == other.timeline_occupies()
            && self.timeline_style() == other.timeline_style()
            && self.skip_control_offset() == other.skip_control_offset()
            && self.skip_control_duration() == other.skip_control_duration()
            && self.skip_control_label_id() == other.skip_control_label_id()
    }
}
macro_rules! interstitial_getter {
    (@Doc = $doc:literal $name:ident @String) => {
        interstitial_getter!(@Private $name, &str, @String, $doc);
    };
    (@Doc = $doc:literal $name:ident @EnumeratedString<$type:ty>) => {
        interstitial_getter!(@Private $name, EnumeratedString<'_, $type>, @EnumeratedString, $doc);
    };
    (@Doc = $doc:literal $name:ident @EnumeratedStringList<$type:ty>) => {
        interstitial_getter!(
            @Private $name,
            EnumeratedStringList<'_, $type>,
            @EnumeratedStringList,
            $doc
        );
    };
    (@Doc = $doc:literal $name:ident @Number) => {
        interstitial_getter!(@Private $name, f64, @Number, $doc);
    };
    (
        @Private $name:ident,
        $type:ty,
        @$type_ident:ident,
        $doc:literal
    ) => {
        #[doc = $doc]
        pub fn $name(&self) -> Option<$type> {
            #[allow(unused_variables)]
            match self.$name {
                LazyAttribute::UserDefined(v) => match v {
                    ExtensionAttributeValue::QuotedString(cow) => ext_get_quoted!(@$type_ident cow),
                    ExtensionAttributeValue::HexadecimalSequence(_) => None,
                    ExtensionAttributeValue::SignedDecimalFloatingPoint(d) => ext_get_float!(@$type_ident d),
                },
                LazyAttribute::Unparsed(v) => get_lazy_unparsed!(@$type_ident v),
                LazyAttribute::None => None,
            }
        }
    };
}
macro_rules! ext_get_quoted {
    (@String $cow:expr) => {
        Some($cow.as_ref())
    };
    (@EnumeratedString $cow:expr) => {
        Some(EnumeratedString::from($cow.as_ref()))
    };
    (@EnumeratedStringList $cow:expr) => {
        Some(EnumeratedStringList::from($cow.as_ref()))
    };
    (@Number $cow:expr) => {
        None
    };
}
macro_rules! ext_get_float {
    (@String $d:expr) => {
        None
    };
    (@EnumeratedString $d:expr) => {
        None
    };
    (@EnumeratedStringList $d:expr) => {
        None
    };
    (@Number $d:expr) => {
        Some(*$d)
    };
}
macro_rules! get_lazy_unparsed {
    (@String $v:expr) => {
        $v.quoted()
    };
    (@EnumeratedString $v:expr) => {
        $v.quoted().map(EnumeratedString::from)
    };
    (@EnumeratedStringList $v:expr) => {
        $v.quoted().map(EnumeratedStringList::from)
    };
    (@Number $v:expr) => {
        $v.unquoted()
            .and_then(|s| s.try_as_decimal_floating_point().ok())
    };
}
impl<'a, 'b> InterstitialExtensionAttributes<'a, 'b> {
    interstitial_getter!(@Doc = "Corresponds to the `X-ASSET-URI` attribute."
        asset_uri @String);
    interstitial_getter!(@Doc = "Corresponds to the `X-ASSET-LIST` attribute."
        asset_list @String);
    interstitial_getter!(@Doc = "Corresponds to the `X-RESUME-OFFSET` attribute."
        resume_offset @Number);
    interstitial_getter!(@Doc = "Corresponds to the `X-PLAYOUT-LIMIT` attribute."
        playout_limit @Number);
    interstitial_getter!(@Doc = "Corresponds to the `X-SNAP` attribute."
        snap @EnumeratedStringList<Snap>);
    interstitial_getter!(@Doc = "Corresponds to the `X-RESTRICT` attribute."
        restrict @EnumeratedStringList<Restrict>);
    interstitial_getter!(@Doc = "Corresponds to the `X-TIMELINE-OCCUPIES` attribute."
        timeline_occupies @EnumeratedString<TimelineOccupies>);
    interstitial_getter!(@Doc = "Corresponds to the `X-TIMELINE-STYLE` attribute."
        timeline_style @EnumeratedString<TimelineStyle>);
    interstitial_getter!(@Doc = "Corresponds to the `X-SKIP-CONTROL-LABEL-ID` attribute."
        skip_control_label_id @String);
    // Note, that while the documentation indicates that X-SKIP-CONTROL-OFFSET and
    // X-SKIP-CONTROL-DURATION are both "decimal integer", and so should be represented as a `u64`,
    // we keep them as `f64` because the spec also states that the extension attribute value "MUST
    // have the form of a quoted-string, a hexadecimal-sequence, or signed-decimal-floating-point."
    // Since the user can set `f64` when constructing `ExtensionAttributeValue`, we avoid any
    // truncation of the value in the case that a float has been set, and leave that repsonsibility
    // to the user.
    interstitial_getter!(@Doc = "Corresponds to the `X-SKIP-CONTROL-OFFSET` attribute."
        skip_control_offset @Number);
    interstitial_getter!(@Doc = "Corresponds to the `X-SKIP-CONTROL-DURATION` attribute."
        skip_control_duration @Number);

    // The X-CONTENT-MAY-VARY is different enough that I didn't want to complicate the macro for it,
    // especialy considering it is just one case.

    /// Corresponds to the `X-CONTENT-MAY-VARY` attribute.
    pub fn content_may_vary(&self) -> bool {
        match self.content_may_vary {
            LazyAttribute::UserDefined(v) => match v {
                ExtensionAttributeValue::QuotedString(cow) => cow.as_bytes() == YES,
                ExtensionAttributeValue::HexadecimalSequence(_) => true,
                ExtensionAttributeValue::SignedDecimalFloatingPoint(_) => true,
            },
            LazyAttribute::Unparsed(v) => v.quoted().map(str::as_bytes).unwrap_or(YES) == YES,
            LazyAttribute::None => true,
        }
    }
}
impl<'a, 'b> From<&'b [(Cow<'a, str>, LazyAttribute<'a, ExtensionAttributeValue<'a>>)]>
    for InterstitialExtensionAttributes<'a, 'b>
{
    fn from(
        attributes: &'b [(Cow<'a, str>, LazyAttribute<'a, ExtensionAttributeValue<'a>>)],
    ) -> Self {
        let mut asset_uri = &LazyAttribute::None;
        let mut asset_list = &LazyAttribute::None;
        let mut resume_offset = &LazyAttribute::None;
        let mut playout_limit = &LazyAttribute::None;
        let mut snap = &LazyAttribute::None;
        let mut restrict = &LazyAttribute::None;
        let mut content_may_vary = &LazyAttribute::None;
        let mut timeline_occupies = &LazyAttribute::None;
        let mut timeline_style = &LazyAttribute::None;
        let mut skip_control_offset = &LazyAttribute::None;
        let mut skip_control_duration = &LazyAttribute::None;
        let mut skip_control_label_id = &LazyAttribute::None;
        for (key, value) in attributes {
            match key.as_ref() {
                X_ASSET_URI => asset_uri = value,
                X_ASSET_LIST => asset_list = value,
                X_RESUME_OFFSET => resume_offset = value,
                X_PLAYOUT_LIMIT => playout_limit = value,
                X_SNAP => snap = value,
                X_RESTRICT => restrict = value,
                X_CONTENT_MAY_VARY => content_may_vary = value,
                X_TIMELINE_OCCUPIES => timeline_occupies = value,
                X_TIMELINE_STYLE => timeline_style = value,
                X_SKIP_CONTROL_OFFSET => skip_control_offset = value,
                X_SKIP_CONTROL_DURATION => skip_control_duration = value,
                X_SKIP_CONTROL_LABEL_ID => skip_control_label_id = value,
                _ => (),
            }
        }
        Self {
            asset_uri,
            asset_list,
            resume_offset,
            playout_limit,
            snap,
            restrict,
            content_may_vary,
            timeline_occupies,
            timeline_style,
            skip_control_offset,
            skip_control_duration,
            skip_control_label_id,
        }
    }
}
/// Corresponds to the attributes defined for HLS Interstitials in [Appnedix D].
///
/// This provides mutable access to the properties. Setting or unsetting values here will set/unset
/// them on the [`Daterange`] from which this was derived from.
///
/// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
#[derive(Debug, PartialEq)]
pub struct InterstitialExtensionAttributesMut<'a, 'b> {
    daterange: &'b mut Daterange<'a>,
}
impl<'a, 'b> InterstitialExtensionAttributesMut<'a, 'b> {
    /// Provides access to the existing values of the interstitial attributes.
    pub fn attrs(&'b self) -> InterstitialExtensionAttributes<'a, 'b> {
        InterstitialExtensionAttributes::from(self.daterange.extension_attributes.as_slice())
    }
}
macro_rules! interstitial_setter {
    (@Doc = $doc:literal $method:ident @String, $name:expr) => {
        interstitial_setter!(
            @Private $method,
            impl Into<Cow<'a, str>>,
            @Into,
            $name,
            QuotedString,
            @Doc = $doc
        );
    };
    (@Doc = $doc:literal $method:ident @Number, $name:expr) => {
        interstitial_setter!(
            @Private $method,
            f64,
            @Ident,
            $name,
            SignedDecimalFloatingPoint,
            @Doc = $doc
        );
    };
    (@Doc = $doc:literal $method:ident @Bool, $name:expr) => {
        interstitial_setter!(
            @Private $method,
            bool,
            @ThenSome,
            $name,
            QuotedString,
            @Doc = $doc
        );
    };
    (
        @Private $method:ident,
        $type:ty,
        @$value_conversion:ident,
        $name:expr,
        $ext_attr:ident,
        @Doc = $doc:literal
    ) => {
        #[doc = $doc]
        pub fn $method(&mut self, value: $type) {
            #[allow(clippy::redundant_locals)]
            let value = convert_value!(@$value_conversion value);
            self.daterange.set_extension_attribute(
                $name,
                ExtensionAttributeValue::$ext_attr(value)
            );
        }
    };
}
macro_rules! convert_value {
    (@Into $value:expr) => {
        $value.into()
    };
    (@ThenSome $value:expr) => {
        $value
            .then_some(Cow::Borrowed("YES"))
            .unwrap_or(Cow::Borrowed("NO"))
    };
    (@Ident $value:expr) => {
        $value
    };
}
macro_rules! interstitial_unsetter {
    (@Doc = $doc:literal $method:ident, $name:expr) => {
        #[doc = $doc]
        pub fn $method(&mut self) {
            self.daterange.unset_extension_attribute($name);
        }
    };
}
impl<'a, 'b> InterstitialExtensionAttributesMut<'a, 'b> {
    interstitial_setter!(@Doc = "Sets the `X-ASSET-URI` attribute."
        set_asset_uri @String, X_ASSET_URI);
    interstitial_unsetter!(@Doc = "Unsets the `X-ASSET-URI` attribute."
        unset_asset_uri, X_ASSET_URI);
    interstitial_setter!(@Doc = "Sets the `X-ASSET-LIST` attribute."
        set_asset_list @String, X_ASSET_LIST);
    interstitial_unsetter!(@Doc = "Unsets the `X-ASSET-LIST` attribute."
        unset_asset_list, X_ASSET_LIST);
    interstitial_setter!(@Doc = "Sets the `X-RESUME-OFFSET` attribute."
        set_resume_offset @Number, X_RESUME_OFFSET);
    interstitial_unsetter!(@Doc = "Unsets the `X-RESUME-OFFSET` attribute."
        unset_resume_offset, X_RESUME_OFFSET);
    interstitial_setter!(@Doc = "Sets the `X-PLAYOUT-LIMIT` attribute."
        set_playout_limit @Number, X_PLAYOUT_LIMIT);
    interstitial_unsetter!(@Doc = "Unsets the `X-PLAYOUT-LIMIT` attribute."
        unset_playout_limit, X_PLAYOUT_LIMIT);
    interstitial_setter!(@Doc = "Sets the `X-SNAP` attribute."
        set_snap @String, X_SNAP);
    interstitial_unsetter!(@Doc = "Unsets the `X-SNAP` attribute."
        unset_snap, X_SNAP);
    interstitial_setter!(@Doc = "Sets the `X-RESTRICT` attribute."
        set_restrict @String, X_RESTRICT);
    interstitial_unsetter!(@Doc = "Unsets the `X-RESTRICT` attribute."
        unset_restrict, X_RESTRICT);
    interstitial_setter!(@Doc = "Sets the `X-CONTENT-MAY-VARY` attribute."
        set_content_may_vary @Bool, X_CONTENT_MAY_VARY);
    interstitial_unsetter!(@Doc = "Unsets the `X-CONTENT-MAY-VARY` attribute."
        unset_content_may_vary, X_CONTENT_MAY_VARY);
    interstitial_setter!(@Doc = "Sets the `X-TIMELINE-OCCUPIES` attribute."
        set_timeline_occupies @String, X_TIMELINE_OCCUPIES);
    interstitial_unsetter!(@Doc = "Unsets the `X-TIMELINE-OCCUPIES` attribute."
        unset_timeline_occupies, X_TIMELINE_OCCUPIES);
    interstitial_setter!(@Doc = "Sets the `X-TIMELINE-STYLE` attribute."
        set_timeline_style @String, X_TIMELINE_STYLE);
    interstitial_unsetter!(@Doc = "Unsets the `X-TIMELINE-STYLE` attribute."
        unset_timeline_style, X_TIMELINE_STYLE);
    interstitial_setter!(@Doc = "Sets the `X-SKIP-CONTROL-OFFSET` attribute."
        set_skip_control_offset @Number, X_SKIP_CONTROL_OFFSET);
    interstitial_unsetter!(@Doc = "Unsets the `X-SKIP-CONTROL-OFFSET` attribute."
        unset_skip_control_offset, X_SKIP_CONTROL_OFFSET);
    interstitial_setter!(@Doc = "Sets the `X-SKIP-CONTROL-DURATION` attribute."
        set_skip_control_duration @Number, X_SKIP_CONTROL_DURATION);
    interstitial_unsetter!(@Doc = "Unsets the `X-SKIP-CONTROL-DURATION` attribute."
        unset_skip_control_duration, X_SKIP_CONTROL_DURATION);
    interstitial_setter!(@Doc = "Sets the `X-SKIP-CONTROL-LABEL-ID` attribute."
        set_skip_control_label_id @String, X_SKIP_CONTROL_LABEL_ID);
    interstitial_unsetter!(@Doc = "Unsets the `X-SKIP-CONTROL-LABEL-ID` attribute."
        unset_skip_control_label_id, X_SKIP_CONTROL_LABEL_ID);
}
const X_ASSET_URI: &str = "X-ASSET-URI";
const X_ASSET_LIST: &str = "X-ASSET-LIST";
const X_RESUME_OFFSET: &str = "X-RESUME-OFFSET";
const X_PLAYOUT_LIMIT: &str = "X-PLAYOUT-LIMIT";
const X_SNAP: &str = "X-SNAP";
const X_RESTRICT: &str = "X-RESTRICT";
const X_CONTENT_MAY_VARY: &str = "X-CONTENT-MAY-VARY";
const X_TIMELINE_OCCUPIES: &str = "X-TIMELINE-OCCUPIES";
const X_TIMELINE_STYLE: &str = "X-TIMELINE-STYLE";
const X_SKIP_CONTROL_OFFSET: &str = "X-SKIP-CONTROL-OFFSET";
const X_SKIP_CONTROL_DURATION: &str = "X-SKIP-CONTROL-DURATION";
const X_SKIP_CONTROL_LABEL_ID: &str = "X-SKIP-CONTROL-LABEL-ID";

/// The attribute list for the tag (`#EXT-X-DATERANGE:<attribute-list>`).
///
/// See [`Daterange`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct DaterangeAttributeList<'a> {
    /// Corresponds to the `ID` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    id: Cow<'a, str>,
    /// Corresponds to the `START-DATE` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    start_date: Option<DateTime>,
    /// Corresponds to the `CLASS` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    class: Option<Cow<'a, str>>,
    /// Corresponds to the `CUE` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    cue: Option<Cow<'a, str>>,
    /// Corresponds to the `END-DATE` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    end_date: Option<DateTime>,
    /// Corresponds to the `DURATION` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    duration: Option<f64>,
    /// Corresponds to the `PLANNED-DURATION` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    planned_duration: Option<f64>,
    /// Corresponds to `X-<extension-attribute>` attributes.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    /// Corresponds to the `END-ON-NEXT` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    end_on_next: bool,
    /// Corresponds to the `SCTE35-CMD` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    scte35_cmd: Option<Cow<'a, str>>,
    /// Corresponds to the `SCTE35-OUT` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    scte35_out: Option<Cow<'a, str>>,
    /// Corresponds to the `SCTE35-IN` attribute.
    ///
    /// See [`Daterange`] for a link to the HLS documentation for this attribute.
    scte35_in: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`DaterangeBuilder`] indicating that `id` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct DaterangeIdNeedsToBeSet;
/// Placeholder struct for [`DaterangeBuilder`] indicating that `id` has been set.
#[derive(Debug, Clone, Copy)]
pub struct DaterangeIdHasBeenSet;

/// A builder for convenience in constructing a [`Daterange`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct DaterangeBuilder<'a, IdStatus> {
    attribute_list: DaterangeAttributeList<'a>,
    id_status: PhantomData<IdStatus>,
}
impl<'a> DaterangeBuilder<'a, DaterangeIdNeedsToBeSet> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: DaterangeAttributeList {
                id: Cow::Borrowed(""),
                start_date: Default::default(),
                class: Default::default(),
                cue: Default::default(),
                end_date: Default::default(),
                duration: Default::default(),
                planned_duration: Default::default(),
                extension_attributes: Default::default(),
                end_on_next: Default::default(),
                scte35_cmd: Default::default(),
                scte35_out: Default::default(),
                scte35_in: Default::default(),
            },
            id_status: PhantomData,
        }
    }
}
impl<'a> DaterangeBuilder<'a, DaterangeIdHasBeenSet> {
    /// Finish building and construct the `Daterange`.
    pub fn finish(self) -> Daterange<'a> {
        Daterange::new(self.attribute_list)
    }
}
impl<'a, IdStatus> DaterangeBuilder<'a, IdStatus> {
    /// Add the provided `id` to the attributes built into `Daterange`.
    pub fn with_id(
        mut self,
        id: impl Into<Cow<'a, str>>,
    ) -> DaterangeBuilder<'a, DaterangeIdHasBeenSet> {
        self.attribute_list.id = id.into();
        DaterangeBuilder {
            attribute_list: self.attribute_list,
            id_status: PhantomData,
        }
    }

    /// Add the provided `start_date` to the attributes built into `Daterange`.
    pub fn with_start_date(mut self, start_date: DateTime) -> Self {
        self.attribute_list.start_date = Some(start_date);
        self
    }

    /// Add the provided `class` to the attributes built into `Daterange`.
    pub fn with_class(mut self, class: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.class = Some(class.into());
        self
    }

    /// Add the provided `cue` to the attributes built into `Daterange`.
    pub fn with_cue(mut self, cue: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.cue = Some(cue.into());
        self
    }

    /// Add the provided `end_date` to the attributes built into `Daterange`.
    pub fn with_end_date(mut self, end_date: DateTime) -> Self {
        self.attribute_list.end_date = Some(end_date);
        self
    }

    /// Add the provided `duration` to the attributes built into `Daterange`.
    pub fn with_duration(mut self, duration: f64) -> Self {
        self.attribute_list.duration = Some(duration);
        self
    }

    /// Add the provided `planned_duration` to the attributes built into `Daterange`.
    pub fn with_planned_duration(mut self, planned_duration: f64) -> Self {
        self.attribute_list.planned_duration = Some(planned_duration);
        self
    }

    /// Add the proivded extension attribute to the attributes built into `Daterange`.
    ///
    /// The attribute name SHOULD be prefixed with `X-`. The library does not validate that this is
    /// the case and unexpected results may occur if this is not followed.
    ///
    /// This sets one attribute at a time. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{Daterange, ExtensionAttributeValue};
    /// # use quick_m3u8::tag::IntoInnerTag;
    /// # use quick_m3u8::date_time;
    /// let daterange = Daterange::builder()
    ///     .with_id("id")
    ///     .with_start_date(date_time!(2025-08-02 T 21:03:00.000 -05:00))
    ///     .with_extension_attribute(
    ///         "X-MESSAGE",
    ///         ExtensionAttributeValue::QuotedString("Hello, World!".into()),
    ///     )
    ///     .with_extension_attribute(
    ///         "X-ANSWER",
    ///         ExtensionAttributeValue::SignedDecimalFloatingPoint(42.0),
    ///     )
    ///     .finish();
    ///
    /// // The order of output of attributes may be mixed so we have to assert that it could be
    /// // either order:
    /// let expected_output_option_1 = concat!(
    ///     "#EXT-X-DATERANGE:ID=\"id\",START-DATE=\"2025-08-02T21:03:00.000-05:00\",",
    ///     "X-MESSAGE=\"Hello, World!\",X-ANSWER=42"
    /// ).as_bytes();
    /// let expected_output_option_2 = concat!(
    ///     "#EXT-X-DATERANGE:ID=\"id\",START-DATE=\"2025-08-02T21:03:00.000-05:00\",",
    ///     "X-ANSWER=42,X-MESSAGE=\"Hello, World!\""
    /// ).as_bytes();
    /// let inner = daterange.into_inner();
    /// let bytes = inner.value();
    /// assert!(bytes == expected_output_option_1 || bytes == expected_output_option_2);
    /// ```
    pub fn with_extension_attribute(
        mut self,
        extension_attribute_name: impl Into<Cow<'a, str>>,
        extension_attribute_value: ExtensionAttributeValue<'a>,
    ) -> Self {
        self.attribute_list
            .extension_attributes
            .insert(extension_attribute_name.into(), extension_attribute_value);
        self
    }

    /// Add the provided extension attributes to the attributes built into `Daterange`.
    ///
    /// The attribute names SHOULD be prefixed with `X-`. The library does not validate that this is
    /// the case and unexpected results may occur if this is not followed.
    pub fn with_extension_attributes(
        mut self,
        extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    ) -> Self {
        self.attribute_list.extension_attributes = extension_attributes;
        self
    }

    /// Add `END-ON-NEXT=YES` to the attributes that are built into `Daterange`.
    pub fn with_end_on_next(mut self) -> Self {
        self.attribute_list.end_on_next = true;
        self
    }

    /// Add the provided `scte35_cmd` to the attributes that are built into the `Daterange`.
    pub fn with_scte35_cmd(mut self, scte35_cmd: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.scte35_cmd = Some(scte35_cmd.into());
        self
    }

    /// Add the provided `scte35_out` to the attributes that are built into the `Daterange`.
    pub fn with_scte35_out(mut self, scte35_out: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.scte35_out = Some(scte35_out.into());
        self
    }

    /// Add the provided `scte35_in` to the attributes that are built into the `Daterange`.
    pub fn with_scte35_in(mut self, scte35_in: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.scte35_in = Some(scte35_in.into());
        self
    }
}
impl<'a> Default for DaterangeBuilder<'a, DaterangeIdNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-DATERANGE` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.5.1>
#[derive(Debug, Clone)]
pub struct Daterange<'a> {
    id: Cow<'a, str>,
    start_date: LazyAttribute<'a, DateTime>,
    class: LazyAttribute<'a, Cow<'a, str>>,
    cue: LazyAttribute<'a, Cow<'a, str>>,
    end_date: LazyAttribute<'a, DateTime>,
    duration: LazyAttribute<'a, f64>,
    planned_duration: LazyAttribute<'a, f64>,
    extension_attributes: Vec<(Cow<'a, str>, LazyAttribute<'a, ExtensionAttributeValue<'a>>)>,
    end_on_next: LazyAttribute<'a, bool>,
    scte35_cmd: LazyAttribute<'a, Cow<'a, str>>,
    scte35_out: LazyAttribute<'a, Cow<'a, str>>,
    scte35_in: LazyAttribute<'a, Cow<'a, str>>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Daterange<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
            && self.start_date() == other.start_date()
            && self.class() == other.class()
            && self.cue() == other.cue()
            && self.end_date() == other.end_date()
            && self.duration() == other.duration()
            && self.planned_duration() == other.planned_duration()
            && self.extension_attributes() == other.extension_attributes()
            && self.end_on_next() == other.end_on_next()
            && self.scte35_cmd() == other.scte35_cmd()
            && self.scte35_out() == other.scte35_out()
            && self.scte35_in() == other.scte35_in()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Daterange<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let mut id = None;
        let mut start_date = LazyAttribute::None;
        let mut class = LazyAttribute::None;
        let mut cue = LazyAttribute::None;
        let mut end_date = LazyAttribute::None;
        let mut duration = LazyAttribute::None;
        let mut planned_duration = LazyAttribute::None;
        let mut extension_attributes = Vec::new();
        let mut end_on_next = LazyAttribute::None;
        let mut scte35_cmd = LazyAttribute::None;
        let mut scte35_out = LazyAttribute::None;
        let mut scte35_in = LazyAttribute::None;
        for (name, value) in attribute_list {
            match name {
                ID => id = value.quoted(),
                START_DATE => start_date.found(value),
                CLASS => class.found(value),
                CUE => cue.found(value),
                END_DATE => end_date.found(value),
                DURATION => duration.found(value),
                PLANNED_DURATION => planned_duration.found(value),
                END_ON_NEXT => end_on_next.found(value),
                SCTE35_CMD => scte35_cmd.found(value),
                SCTE35_OUT => scte35_out.found(value),
                SCTE35_IN => scte35_in.found(value),
                n if n.starts_with("X-") => {
                    extension_attributes.push((Cow::Borrowed(n), LazyAttribute::Unparsed(value)))
                }
                _ => (),
            }
        }
        let Some(id) = id else {
            return Err(ValidationError::MissingRequiredAttribute(ID));
        };
        Ok(Self {
            id: Cow::Borrowed(id),
            start_date,
            class,
            cue,
            end_date,
            duration,
            planned_duration,
            extension_attributes,
            end_on_next,
            scte35_cmd,
            scte35_out,
            scte35_in,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Daterange<'a> {
    /// Constructs a new `Daterange` tag.
    fn new(attribute_list: DaterangeAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let DaterangeAttributeList {
            id,
            start_date,
            class,
            cue,
            end_date,
            duration,
            planned_duration,
            extension_attributes,
            end_on_next,
            scte35_cmd,
            scte35_out,
            scte35_in,
        } = attribute_list;
        Self {
            id,
            start_date: start_date.map(LazyAttribute::new).unwrap_or_default(),
            class: class.map(LazyAttribute::new).unwrap_or_default(),
            cue: cue.map(LazyAttribute::new).unwrap_or_default(),
            end_date: end_date.map(LazyAttribute::new).unwrap_or_default(),
            duration: duration.map(LazyAttribute::new).unwrap_or_default(),
            planned_duration: planned_duration.map(LazyAttribute::new).unwrap_or_default(),
            extension_attributes: extension_attributes
                .into_iter()
                .map(|(key, value)| (key, LazyAttribute::new(value)))
                .collect(),
            end_on_next: LazyAttribute::new(end_on_next),
            scte35_cmd: scte35_cmd.map(LazyAttribute::new).unwrap_or_default(),
            scte35_out: scte35_out.map(LazyAttribute::new).unwrap_or_default(),
            scte35_in: scte35_in.map(LazyAttribute::new).unwrap_or_default(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Daterange` as such:
    /// ```
    /// # use quick_m3u8::tag::hls::{Daterange, ExtensionAttributeValue, Cue};
    /// # use quick_m3u8::date_time;
    /// let daterange = Daterange::builder()
    ///     .with_id("id")
    ///     .with_start_date(date_time!(2025-08-02 T 21:22:33.123))
    ///     .with_duration(120.0)
    ///     .with_cue(Cue::Once)
    ///     .with_class("com.example.ad.id")
    ///     .with_extension_attribute(
    ///         "X-AD-ID",
    ///         ExtensionAttributeValue::QuotedString("1234".into())
    ///     )
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `id`. The following
    /// will fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Daterange;
    /// let daterange = Daterange::builder().finish();
    /// ```
    pub fn builder() -> DaterangeBuilder<'a, DaterangeIdNeedsToBeSet> {
        DaterangeBuilder::new()
    }

    // === GETTERS ===

    /// Corresponds to the `ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Corresponds to the `CLASS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn class(&self) -> Option<&str> {
        match &self.class {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `START-DATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn start_date(&self) -> Option<DateTime> {
        match &self.start_date {
            LazyAttribute::UserDefined(s) => Some(*s),
            LazyAttribute::Unparsed(v) => v.quoted().and_then(|s| date::parse(s).ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `CUE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// This attribute provides an [`EnumeratedStringList`] wrapper around [`Cue`]. The
    /// documentation on `EnumeratedStringList` provides more information around the concept. Below
    /// shows an example usage:
    /// ```
    /// # use quick_m3u8::{Reader, HlsLine, config::ParsingOptions, tag::KnownTag,
    /// # tag::hls::{self, Cue}};
    /// let daterange =
    ///     r#"#EXT-X-DATERANGE:ID="id",START-DATE="2025-08-02T21:31:00Z",CUE="PRE,ONCE""#;
    /// let mut reader = Reader::from_str(daterange, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::Daterange(tag))))) => {
    ///         let cue = tag.cue().expect("should have cue defined");
    ///         assert!(cue.contains(Cue::Pre));
    ///         assert!(cue.contains(Cue::Once));
    ///         assert!(!cue.contains(Cue::Post));
    ///     }
    ///     r => panic!("unexpected result {r:?}")
    /// }
    /// ```
    pub fn cue(&self) -> Option<EnumeratedStringList<'_, Cue>> {
        match &self.cue {
            LazyAttribute::UserDefined(s) => Some(EnumeratedStringList::from(s.as_ref())),
            LazyAttribute::Unparsed(v) => v.quoted().map(EnumeratedStringList::from),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `END-DATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn end_date(&self) -> Option<DateTime> {
        match &self.end_date {
            LazyAttribute::UserDefined(s) => Some(*s),
            LazyAttribute::Unparsed(v) => v.quoted().and_then(|s| date::parse(s).ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn duration(&self) -> Option<f64> {
        match &self.duration {
            LazyAttribute::UserDefined(d) => Some(*d),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|d| d.try_as_decimal_floating_point().ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `PLANNED-DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn planned_duration(&self) -> Option<f64> {
        match &self.planned_duration {
            LazyAttribute::UserDefined(d) => Some(*d),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|d| d.try_as_decimal_floating_point().ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `X-<extension-attribute>` attributes.
    ///
    /// NOTE: prior to draft 18 these were known as `X-<client-attribute>`.
    ///
    /// This method collects all the attributes prefixed with `X-` and provides them in a `HashMap`.
    /// For example:
    /// ```
    /// # use quick_m3u8::{
    /// # Reader, HlsLine, config::ParsingOptions, tag::KnownTag,
    /// # tag::hls::{self, ExtensionAttributeValue}
    /// # };
    /// # use std::collections::HashMap;
    /// let daterange =
    ///     r#"#EXT-X-DATERANGE:ID="id",START-DATE="2025-08-02T21:31:00Z",X-EX-A="A",X-EX-B=42"#;
    /// let mut reader = Reader::from_str(daterange, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::Daterange(tag))))) => {
    ///         assert_eq!(
    ///             HashMap::from([
    ///                 ("X-EX-A", ExtensionAttributeValue::QuotedString("A".into())),
    ///                 ("X-EX-B", ExtensionAttributeValue::SignedDecimalFloatingPoint(42.0)),
    ///             ]),
    ///             tag.extension_attributes()
    ///         );
    ///     }
    ///     r => panic!("unexpected result {r:?}")
    /// }
    /// ```
    pub fn extension_attributes(&self) -> HashMap<&str, ExtensionAttributeValue<'_>> {
        HashMap::from_iter(self.extension_attributes.iter().filter_map(|(key, value)| {
            match value {
                LazyAttribute::UserDefined(a) => {
                    Some((key.as_ref(), ExtensionAttributeValue::from(a)))
                }
                LazyAttribute::Unparsed(v) => ExtensionAttributeValue::try_from(*v)
                    .ok()
                    .map(|v| (key.as_ref(), v)),
                LazyAttribute::None => None,
            }
        }))
    }

    /// Corresponds to one of the `X-<extension-attribute>` attributes (keyed by `name`).
    ///
    /// NOTE: prior to draft 18 these were known as `X-<client-attribute>`.
    ///
    /// This method attempts to get the attribute value for the provided `name`. The `X-` prefix
    /// must be included. For example:
    /// ```
    /// # use quick_m3u8::{
    /// # Reader, HlsLine, config::ParsingOptions, tag::KnownTag,
    /// # tag::hls::{self, ExtensionAttributeValue}
    /// # };
    /// # use std::collections::HashMap;
    /// let daterange =
    ///     r#"#EXT-X-DATERANGE:ID="id",START-DATE="2025-08-02T21:31:00Z",X-EX-A="A",X-EX-B=42"#;
    /// let mut reader = Reader::from_str(daterange, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::Daterange(tag))))) => {
    ///         assert_eq!(
    ///             Some(ExtensionAttributeValue::QuotedString("A".into())),
    ///             tag.extension_attribute("X-EX-A"),
    ///         );
    ///         assert_eq!(
    ///             Some(ExtensionAttributeValue::SignedDecimalFloatingPoint(42.0)),
    ///             tag.extension_attribute("X-EX-B"),
    ///         );
    ///     }
    ///     r => panic!("unexpected result {r:?}")
    /// }
    /// ```
    pub fn extension_attribute<'b>(&'a self, name: &'b str) -> Option<ExtensionAttributeValue<'a>> {
        self.extension_attributes.iter().find_map(|(key, value)| {
            if name == key.as_ref() {
                match value {
                    LazyAttribute::UserDefined(v) => Some(ExtensionAttributeValue::from(v)),
                    LazyAttribute::Unparsed(v) => ExtensionAttributeValue::try_from(*v).ok(),
                    LazyAttribute::None => None,
                }
            } else {
                None
            }
        })
    }

    /// Corresponds to the keys of the `X-<extension-attribute>` attributes.
    ///
    /// NOTE: prior to draft 18 these were known as `X-<client-attribute>`.
    ///
    /// This method provides the extension attribute keys that exist in the tag. For example:
    /// ```
    /// # use quick_m3u8::{
    /// # Reader, HlsLine, config::ParsingOptions, tag::KnownTag,
    /// # tag::hls,
    /// # };
    /// # use std::collections::HashSet;
    /// let daterange =
    ///     r#"#EXT-X-DATERANGE:ID="id",START-DATE="2025-08-02T21:31:00Z",X-EX-A="A",X-EX-B=42"#;
    /// let mut reader = Reader::from_str(daterange, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::Daterange(tag))))) => {
    ///         assert_eq!(
    ///             HashSet::from(["X-EX-A", "X-EX-B"]),
    ///             tag.extension_attribute_keys(),
    ///         );
    ///     }
    ///     r => panic!("unexpected result {r:?}")
    /// }
    /// ```
    pub fn extension_attribute_keys(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for (key, _) in &self.extension_attributes {
            set.insert(key.as_ref());
        }
        set
    }

    /// Provides typed access to the extension attributes defined for HLS Interstitials in
    /// [Appendix D].
    ///
    /// This will return `None` if the `CLASS` is not set to `com.apple.hls.interstitial`.
    ///
    /// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
    pub fn interstitial_attributes(&self) -> Option<InterstitialExtensionAttributes<'a, '_>> {
        if self.class() == Some(INTERSTITIAL_CLASS) {
            Some(InterstitialExtensionAttributes::from(
                self.extension_attributes.as_slice(),
            ))
        } else {
            None
        }
    }

    /// Provides typed access to the extension attributes defined for HLS Interstitials in
    /// [Appendix D].
    ///
    /// This will return `None` if the `CLASS` is not set to `com.apple.hls.interstitial`.
    ///
    /// This provides mutable access to the properties. Setting or unsetting values here will set/unset
    /// them on the [`Daterange`] from which this was derived from.
    ///
    /// [Appendix D]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#appendix-D
    pub fn interstitial_attributes_mut(
        &mut self,
    ) -> Option<InterstitialExtensionAttributesMut<'a, '_>> {
        if self.class() == Some(INTERSTITIAL_CLASS) {
            Some(InterstitialExtensionAttributesMut { daterange: self })
        } else {
            None
        }
    }

    /// Corresponds to the `END-ON-NEXT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn end_on_next(&self) -> bool {
        match self.end_on_next {
            LazyAttribute::UserDefined(b) => b,
            LazyAttribute::Unparsed(v) => {
                matches!(v, AttributeValue::Unquoted(UnquotedAttributeValue(YES)))
            }
            LazyAttribute::None => false,
        }
    }

    // The specification indicates that the SCTE35-(CMD|OUT|IN) attributes are
    // represented as hexadecimal sequences. This implies that they should be parsed as
    // UnquotedString (given that section "4.2. Attribute Lists" indicates that a
    // "hexadecimal-sequence [is] an unquoted string of characters"); however, in
    // practice, I've found that some packagers have put this information in quoted
    // strings (containing the hexadecimal sequence), so I'll allow this parser to be
    // lenient on that requirement and accept both.

    /// Corresponds to the `SCTE35-CMD` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note, the specification indicates that the SCTE35-(CMD|OUT|IN) attributes are represented as
    /// hexadecimal sequences. This implies that they should be parsed as UnquotedString (given that
    /// section "4.2. Attribute Lists" indicates that a "hexadecimal-sequence \[is\] an unquoted
    /// string of characters"); however, in practice, I've found that some packagers have put this
    /// information in quoted strings (containing the hexadecimal sequence), so we've allowed this
    /// parser to be lenient on that requirement and accept both. The implication is that
    /// `#EXT-X-DATERANGE:SCTE35-CMD=0x123` is equivalent to `#EXT-X-DATERANGE:SCTE35-CMD="0x123"`.
    pub fn scte35_cmd(&self) -> Option<&str> {
        match &self.scte35_cmd {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => match v {
                AttributeValue::Unquoted(v) => v.try_as_utf_8().ok(),
                AttributeValue::Quoted(s) => Some(s),
            },
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `SCTE35-OUT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note, the specification indicates that the SCTE35-(CMD|OUT|IN) attributes are represented as
    /// hexadecimal sequences. This implies that they should be parsed as UnquotedString (given that
    /// section "4.2. Attribute Lists" indicates that a "hexadecimal-sequence \[is\] an unquoted
    /// string of characters"); however, in practice, I've found that some packagers have put this
    /// information in quoted strings (containing the hexadecimal sequence), so we've allowed this
    /// parser to be lenient on that requirement and accept both. The implication is that
    /// `#EXT-X-DATERANGE:SCTE35-OUT=0x123` is equivalent to `#EXT-X-DATERANGE:SCTE35-OUT="0x123"`.
    pub fn scte35_out(&self) -> Option<&str> {
        match &self.scte35_out {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => match v {
                AttributeValue::Unquoted(v) => v.try_as_utf_8().ok(),
                AttributeValue::Quoted(s) => Some(s),
            },
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `SCTE35-IN` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note, the specification indicates that the SCTE35-(CMD|OUT|IN) attributes are represented as
    /// hexadecimal sequences. This implies that they should be parsed as UnquotedString (given that
    /// section "4.2. Attribute Lists" indicates that a "hexadecimal-sequence \[is\] an unquoted
    /// string of characters"); however, in practice, I've found that some packagers have put this
    /// information in quoted strings (containing the hexadecimal sequence), so we've allowed this
    /// parser to be lenient on that requirement and accept both. The implication is that
    /// `#EXT-X-DATERANGE:SCTE35-IN=0x123` is equivalent to `#EXT-X-DATERANGE:SCTE35-IN="0x123"`.
    pub fn scte35_in(&self) -> Option<&str> {
        match &self.scte35_in {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => match v {
                AttributeValue::Unquoted(v) => v.try_as_utf_8().ok(),
                AttributeValue::Quoted(s) => Some(s),
            },
            LazyAttribute::None => None,
        }
    }

    // === SETTERS ===

    /// Sets the `ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_id(&mut self, id: impl Into<Cow<'a, str>>) {
        self.id = id.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `CLASS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_class(&mut self, class: impl Into<Cow<'a, str>>) {
        self.class.set(class.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CLASS` attribute (set it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_class(&mut self) {
        self.class.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `START-DATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_start_date(&mut self, start_date: DateTime) {
        self.start_date.set(start_date);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `START-DATE` attribute (set it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_start_date(&mut self) {
        self.start_date.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `CUE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note, that [`Cue`] implements `Into<Cow<str>>` and therefore can be used directly to set the
    /// value. Similarly, an array of `Cue` can be used. For example:
    /// ```
    /// # use quick_m3u8::{
    /// # Reader, HlsLine, config::ParsingOptions, tag::KnownTag,
    /// # tag::hls::{self, Cue, EnumeratedStringList, EnumeratedString}
    /// # };
    /// let daterange =
    ///     r#"#EXT-X-DATERANGE:ID="id",START-DATE="2025-08-02T21:31:00Z",CUE="PRE,ONCE""#;
    /// let mut reader = Reader::from_str(daterange, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::Daterange(mut tag))))) => {
    ///         let mut cue = tag.cue().expect("should have cue defined");
    ///         cue.remove(Cue::Pre);
    ///         tag.set_cue(cue.to_owned());
    ///         assert_eq!("ONCE", tag.cue().expect("must be defined").as_ref());
    ///         
    ///         tag.set_cue(Cue::Pre);
    ///         assert_eq!("PRE", tag.cue().expect("must be defined").as_ref());
    ///         
    ///         tag.set_cue(EnumeratedStringList::from([Cue::Once, Cue::Post]));
    ///         assert_eq!("ONCE,POST", tag.cue().expect("must be defined").as_ref());
    ///     }
    ///     r => panic!("unexpected result {r:?}")
    /// }
    /// ```
    pub fn set_cue(&mut self, cue: impl Into<Cow<'a, str>>) {
        self.cue.set(cue.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CUE` attribute (set it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_cue(&mut self) {
        self.cue.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `END-DATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_end_date(&mut self, end_date: DateTime) {
        self.end_date.set(end_date);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `END-DATE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_end_date(&mut self) {
        self.end_date.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_duration(&mut self, duration: f64) {
        self.duration.set(duration);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `DURATION` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_duration(&mut self) {
        self.duration.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `PLANNED-DURATION` attribute.
    pub fn set_planned_duration(&mut self, planned_duration: f64) {
        self.planned_duration.set(planned_duration);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `PLANNED-DURATION` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_planned_duration(&mut self) {
        self.planned_duration.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets an extension attribute (`X-<extension-attribute>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that this silently fails if the name provided does not begin with `"X-"`. This is
    /// likely to change in the future as per issue [#1].
    ///
    /// [#1]: https://github.com/theRealRobG/m3u8/issues/1
    pub fn set_extension_attribute(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: ExtensionAttributeValue<'a>,
    ) {
        let name = name.into();
        if !name.starts_with("X-") {
            return;
        }
        self.extension_attributes.retain(|(k, _)| *k != name);
        self.extension_attributes
            .push((name, LazyAttribute::new(value)));
        self.output_line_is_dirty = true;
    }

    /// Unsets an extension attribute (`X-<extension-attribute>`) (removes it from the `HashMap`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that this silently fails if the name provided does not begin with `"X-"`. This is
    /// likely to change in the future as per issue [#1].
    ///
    /// [#1]: https://github.com/theRealRobG/m3u8/issues/1
    pub fn unset_extension_attribute(&mut self, name: impl Into<Cow<'a, str>>) {
        let name = name.into();
        if !name.starts_with("X-") {
            return;
        }
        self.extension_attributes.retain(|(k, _)| *k != name);
        self.output_line_is_dirty = true;
    }

    /// Sets the `END-ON-NEXT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_end_on_next(&mut self, end_on_next: bool) {
        self.end_on_next.set(end_on_next);
        self.output_line_is_dirty = true;
    }

    /// Sets the `SCTE35-CMD` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_scte35_cmd(&mut self, scte35_cmd: impl Into<Cow<'a, str>>) {
        self.scte35_cmd.set(scte35_cmd.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SCTE35-CMD` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_scte35_cmd(&mut self) {
        self.scte35_cmd.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `SCTE35-OUT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_scte35_out(&mut self, scte35_out: impl Into<Cow<'a, str>>) {
        self.scte35_out.set(scte35_out.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SCTE35-OUT` attribute (sets it to `None`).
    pub fn unset_scte35_out(&mut self) {
        self.scte35_out.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `SCTE35-IN` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_scte35_in(&mut self, scte35_in: impl Into<Cow<'a, str>>) {
        self.scte35_in.set(scte35_in.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SCTE35-IN` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_scte35_in(&mut self) {
        self.scte35_in.unset();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&DaterangeAttributeList {
            id: self.id().into(),
            class: self.class().map(|x| x.into()),
            start_date: self.start_date(),
            cue: self.cue().map(|x| x.into()),
            end_date: self.end_date(),
            duration: self.duration(),
            planned_duration: self.planned_duration(),
            extension_attributes: {
                let mut map = HashMap::new();
                for (key, value) in self.extension_attributes() {
                    map.insert(Cow::Borrowed(key), value);
                }
                map
            },
            scte35_cmd: self.scte35_cmd().map(|x| x.into()),
            scte35_out: self.scte35_out().map(|x| x.into()),
            scte35_in: self.scte35_in().map(|x| x.into()),
            end_on_next: self.end_on_next(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Daterange);

/// Provides the value for an extension attribute (`X-<extension-attribute>` as defined in the
/// EXT-X-DATERANGE tag specification).
#[derive(Debug, PartialEq, Clone)]
pub enum ExtensionAttributeValue<'a> {
    /// A quoted string value.
    QuotedString(Cow<'a, str>),
    /// A hexadecimal sequence value (an unquoted string of characters, prefixed by `0x`, all of
    /// which are valid hex characters). Note that the library does not validate that the parsed
    /// input is valid hex.
    HexadecimalSequence(Cow<'a, str>),
    /// A signed decimal floating point value.
    SignedDecimalFloatingPoint(f64),
}
impl<'a> ExtensionAttributeValue<'a> {
    /// Create a new [`Self::QuotedString`] case.
    pub fn quoted_string(quoted_string: impl Into<Cow<'a, str>>) -> Self {
        Self::QuotedString(quoted_string.into())
    }

    /// Create a new [`Self::HexadecimalSequence`] case.
    pub fn hexadecimal_sequence(hexadecimal_sequence: impl Into<Cow<'a, str>>) -> Self {
        Self::HexadecimalSequence(hexadecimal_sequence.into())
    }

    /// Create a new [`Self::SignedDecimalFloatingPoint`] case.
    pub fn signed_decimal_floating_point(signed_decimal_floating_point: f64) -> Self {
        Self::SignedDecimalFloatingPoint(signed_decimal_floating_point)
    }
}

impl<'a> TryFrom<AttributeValue<'a>> for ExtensionAttributeValue<'a> {
    type Error = &'static str;

    fn try_from(value: AttributeValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AttributeValue::Unquoted(v) => {
                if let Ok(d) = v.try_as_decimal_floating_point() {
                    Ok(Self::SignedDecimalFloatingPoint(d))
                } else if let Ok(s) = v.try_as_utf_8()
                    && is_hexadecimal_sequence(s)
                {
                    Ok(Self::HexadecimalSequence(Cow::Borrowed(s)))
                } else {
                    Err("Invalid extension attribute value")
                }
            }
            AttributeValue::Quoted(s) => Ok(Self::QuotedString(Cow::Borrowed(s))),
        }
    }
}

impl<'a> From<&'a ExtensionAttributeValue<'a>> for ExtensionAttributeValue<'a> {
    fn from(value: &'a ExtensionAttributeValue<'a>) -> Self {
        match value {
            Self::QuotedString(cow) => Self::QuotedString(Cow::Borrowed(cow)),
            Self::HexadecimalSequence(cow) => Self::HexadecimalSequence(Cow::Borrowed(cow)),
            Self::SignedDecimalFloatingPoint(d) => Self::SignedDecimalFloatingPoint(*d),
        }
    }
}

fn is_hexadecimal_sequence(s: &str) -> bool {
    let mut bytes = s.bytes();
    if bytes.next() != Some(b'0') {
        return false;
    }
    let x = bytes.next();
    if x != Some(b'x') && x != Some(b'X') {
        return false;
    }
    if !bytes.all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    true
}

const ID: &str = "ID";
const CLASS: &str = "CLASS";
const START_DATE: &str = "START-DATE";
const CUE: &str = "CUE";
const END_DATE: &str = "END-DATE";
const DURATION: &str = "DURATION";
const PLANNED_DURATION: &str = "PLANNED-DURATION";
const SCTE35_CMD: &str = "SCTE35-CMD";
const SCTE35_OUT: &str = "SCTE35-OUT";
const SCTE35_IN: &str = "SCTE35-IN";
const END_ON_NEXT: &str = "END-ON-NEXT";
const YES: &[u8] = b"YES";

fn calculate_line(attribute_list: &DaterangeAttributeList) -> Vec<u8> {
    let DaterangeAttributeList {
        id,
        start_date,
        class,
        cue,
        end_date,
        duration,
        planned_duration,
        extension_attributes,
        end_on_next,
        scte35_cmd,
        scte35_out,
        scte35_in,
    } = attribute_list;
    let mut line = format!("#EXT{}:{}=\"{}\"", TagName::Daterange.as_str(), ID, id,);
    if let Some(start_date) = start_date {
        line.push_str(format!(",{START_DATE}=\"{start_date}\"").as_str());
    }
    if let Some(class) = class {
        line.push_str(format!(",{CLASS}=\"{class}\"").as_str());
    }
    if let Some(cue) = cue {
        line.push_str(format!(",{CUE}=\"{cue}\"").as_str());
    }
    if let Some(end_date) = end_date {
        line.push_str(format!(",{END_DATE}=\"{end_date}\"").as_str());
    }
    if let Some(duration) = duration {
        line.push_str(format!(",{DURATION}={duration}").as_str());
    }
    if let Some(planned_duration) = planned_duration {
        line.push_str(format!(",{PLANNED_DURATION}={planned_duration}").as_str());
    }
    for (key, value) in extension_attributes {
        line.push(',');
        line.push_str(key);
        line.push('=');
        match value {
            ExtensionAttributeValue::HexadecimalSequence(s) => {
                line.push_str(s);
            }
            ExtensionAttributeValue::QuotedString(s) => {
                line.push('"');
                line.push_str(s);
                line.push('"');
            }
            ExtensionAttributeValue::SignedDecimalFloatingPoint(d) => {
                line.push_str(format!("{d}").as_str());
            }
        };
    }
    if let Some(scte35_cmd) = scte35_cmd {
        line.push_str(format!(",{SCTE35_CMD}={scte35_cmd}").as_str());
    }
    if let Some(scte35_out) = scte35_out {
        line.push_str(format!(",{SCTE35_OUT}={scte35_out}").as_str());
    }
    if let Some(scte35_in) = scte35_in {
        line.push_str(format!(",{SCTE35_IN}={scte35_in}").as_str());
    }
    if *end_on_next {
        line.push_str(",END-ON-NEXT=YES");
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        date_time,
        tag::{IntoInnerTag, hls::test_macro::mutation_tests},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_optionals_should_be_valid() {
        let tag = Daterange::builder()
            .with_id("some-id")
            .with_start_date(date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .finish();
        assert_eq!(
            b"#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\"",
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_should_be_valid() {
        let tag = Daterange::builder()
            .with_id("some-id")
            .with_start_date(date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_class("com.example.class")
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_end_date(date_time!(2025-06-14 T 23:43:42.000 -05:00))
            .with_duration(120.0)
            .with_planned_duration(180.0)
            .with_scte35_cmd("0xABCD")
            .with_scte35_out("0xABCD")
            .with_scte35_in("0xABCD")
            .with_end_on_next()
            .finish();
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\",",
                "CLASS=\"com.example.class\",CUE=\"ONCE\",",
                "END-DATE=\"2025-06-14T23:43:42.000-05:00\",DURATION=120,PLANNED-DURATION=180,",
                "SCTE35-CMD=0xABCD,SCTE35-OUT=0xABCD,SCTE35-IN=0xABCD,END-ON-NEXT=YES"
            )
            .as_bytes(),
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_and_some_client_attributes_should_be_valid() {
        let tag = Daterange::builder()
            .with_id("some-id")
            .with_start_date(date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_extension_attribute(
                "X-COM-EXAMPLE-A",
                ExtensionAttributeValue::QuotedString("Example A".into()),
            )
            .with_extension_attribute(
                "X-COM-EXAMPLE-B",
                ExtensionAttributeValue::SignedDecimalFloatingPoint(42.0),
            )
            .with_extension_attribute(
                "X-COM-EXAMPLE-C",
                ExtensionAttributeValue::HexadecimalSequence("0xABCD".into()),
            )
            .finish();
        // Client attributes can come in any order (due to it being a HashMap) so we need to be a
        // little more creative in validating the tag format.
        let tag_inner = tag.into_inner();
        let tag_as_bytes = tag_inner.value();
        let mut found_a = false;
        let mut found_b = false;
        let mut found_c = false;
        for (index, split) in tag_as_bytes.split(|b| b == &b',').enumerate() {
            match index {
                0 => assert_eq!(b"#EXT-X-DATERANGE:ID=\"some-id\"", split),
                1 => assert_eq!(b"START-DATE=\"2025-06-14T23:41:42.000-05:00\"", split),
                2 | 3 | 4 => {
                    if split.starts_with(b"X-COM-EXAMPLE-A") {
                        if found_a {
                            panic!("Already found A")
                        }
                        found_a = true;
                        assert_eq!(b"X-COM-EXAMPLE-A=\"Example A\"", split);
                    } else if split.starts_with(b"X-COM-EXAMPLE-B") {
                        if found_b {
                            panic!("Already found B")
                        }
                        found_b = true;
                        assert_eq!(b"X-COM-EXAMPLE-B=42", split);
                    } else if split.starts_with(b"X-COM-EXAMPLE-C") {
                        if found_c {
                            panic!("Already found C")
                        }
                        found_c = true;
                        assert_eq!(b"X-COM-EXAMPLE-C=0xABCD", split);
                    } else {
                        panic!("Unexpected attribute at index {}", index);
                    }
                }
                _ => panic!("Too many attributes"),
            }
        }
        assert!(found_a);
        assert!(found_b);
        assert!(found_c);
    }

    #[test]
    fn mutation_should_work() {
        let mut daterange = Daterange::builder()
            .with_id("some-id")
            .with_start_date(DateTime::default())
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_extension_attribute(
                "X-TO-REMOVE",
                ExtensionAttributeValue::QuotedString("remove me".into()),
            )
            .finish();
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",",
                "CUE=\"ONCE\",X-TO-REMOVE=\"remove me\"",
            )
            .as_bytes(),
            daterange.clone().into_inner().value()
        );
        daterange.set_id("another-id");
        daterange.set_class("com.example.test");
        daterange.unset_cue();
        daterange.set_extension_attribute(
            "X-EXAMPLE",
            ExtensionAttributeValue::QuotedString("TEST".into()),
        );
        daterange.unset_extension_attribute("X-TO-REMOVE");
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"another-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",",
                "CLASS=\"com.example.test\",X-EXAMPLE=\"TEST\"",
            )
            .as_bytes(),
            daterange.into_inner().value()
        );
    }

    #[test]
    fn mutating_cue_works_as_expected() {
        let mut daterange = Daterange::builder()
            .with_id("some-id")
            .with_start_date(DateTime::default())
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .finish();
        let mut cue = daterange.cue().unwrap();
        cue.insert(Cue::Pre);
        daterange.set_cue(cue.to_owned());
        assert_eq!(
            EnumeratedStringList::from([Cue::Once, Cue::Pre]),
            daterange.cue().unwrap()
        );
    }

    #[test]
    fn interstitial_attributes_are_parsed_correctly_and_mutable() {
        let daterange_line = concat!(
            "#EXT-X-DATERANGE:ID=\"ad-1\",X-ASSET-LIST=\"ad-1.json\",X-RESUME-OFFSET=10.0,",
            "X-PLAYOUT-LIMIT=60.0,X-SNAP=\"OUT,IN\",X-RESTRICT=\"JUMP,SKIP\",",
            "X-CONTENT-MAY-VARY=\"NO\",X-TIMELINE-OCCUPIES=\"RANGE\",X-TIMELINE-STYLE=\"PRIMARY\",",
            "X-SKIP-CONTROL-OFFSET=10,X-SKIP-CONTROL-DURATION=20,X-SKIP-CONTROL-LABEL-ID=\"skip\""
        );
        let tag = crate::custom_parsing::tag::parse(daterange_line)
            .expect("parsing should succeed")
            .parsed;
        let mut daterange = Daterange::try_from(tag).expect("tag should be valid daterange");

        // attrs are None until CLASS is set.
        assert_eq!(None, daterange.interstitial_attributes());
        assert_eq!(None, daterange.interstitial_attributes_mut());
        daterange.set_class(INTERSTITIAL_CLASS);
        let interstitial_attrs = daterange
            .interstitial_attributes()
            .expect("interstitial attrs should be defined");

        assert_eq!(None, interstitial_attrs.asset_uri());
        assert_eq!(Some("ad-1.json"), interstitial_attrs.asset_list());
        assert_eq!(Some(10.0), interstitial_attrs.resume_offset());
        assert_eq!(Some(60.0), interstitial_attrs.playout_limit());
        assert_eq!(
            Some(EnumeratedStringList::from([Snap::Out, Snap::In])),
            interstitial_attrs.snap()
        );
        assert_eq!(
            Some(EnumeratedStringList::from([Restrict::Jump, Restrict::Skip])),
            interstitial_attrs.restrict()
        );
        assert_eq!(false, interstitial_attrs.content_may_vary());
        assert_eq!(
            Some(EnumeratedString::from(TimelineOccupies::Range)),
            interstitial_attrs.timeline_occupies()
        );
        assert_eq!(
            Some(EnumeratedString::from(TimelineStyle::Primary)),
            interstitial_attrs.timeline_style()
        );
        assert_eq!(Some(10.0), interstitial_attrs.skip_control_offset());
        assert_eq!(Some(20.0), interstitial_attrs.skip_control_duration());
        assert_eq!(Some("skip"), interstitial_attrs.skip_control_label_id());

        // Test mutation
        let mut attrs = daterange
            .interstitial_attributes_mut()
            .expect("attrs should be defined");
        attrs.set_asset_uri("ad-1.m3u8");
        attrs.unset_asset_list();
        attrs.set_resume_offset(70.0);
        attrs.set_playout_limit(50.0);
        attrs.set_snap(Snap::Out);
        attrs.set_restrict(Restrict::Jump);
        attrs.unset_content_may_vary();
        attrs.set_timeline_occupies(TimelineOccupies::Point);
        attrs.set_timeline_style(TimelineStyle::Highlight);
        attrs.set_skip_control_offset(20.0);
        attrs.set_skip_control_duration(10.0);
        attrs.set_skip_control_label_id("skippy");

        // Test on mutable ref
        assert_eq!(Some("ad-1.m3u8"), attrs.attrs().asset_uri());
        assert_eq!(None, attrs.attrs().asset_list());
        assert_eq!(Some(70.0), attrs.attrs().resume_offset());
        assert_eq!(Some(50.0), attrs.attrs().playout_limit());
        assert_eq!(
            Some(EnumeratedStringList::from([Snap::Out])),
            attrs.attrs().snap()
        );
        assert_eq!(
            Some(EnumeratedStringList::from([Restrict::Jump])),
            attrs.attrs().restrict()
        );
        assert_eq!(true, attrs.attrs().content_may_vary());
        assert_eq!(
            Some(EnumeratedString::from(TimelineOccupies::Point)),
            attrs.attrs().timeline_occupies()
        );
        assert_eq!(
            Some(EnumeratedString::from(TimelineStyle::Highlight)),
            attrs.attrs().timeline_style()
        );
        assert_eq!(Some(20.0), attrs.attrs().skip_control_offset());
        assert_eq!(Some(10.0), attrs.attrs().skip_control_duration());
        assert_eq!(Some("skippy"), attrs.attrs().skip_control_label_id());

        // Test has been set on daterange too
        let attrs = daterange
            .interstitial_attributes()
            .expect("should have interstitials defined");
        assert_eq!(Some("ad-1.m3u8"), attrs.asset_uri());
        assert_eq!(None, attrs.asset_list());
        assert_eq!(Some(70.0), attrs.resume_offset());
        assert_eq!(Some(50.0), attrs.playout_limit());
        assert_eq!(Some(EnumeratedStringList::from([Snap::Out])), attrs.snap());
        assert_eq!(
            Some(EnumeratedStringList::from([Restrict::Jump])),
            attrs.restrict()
        );
        assert_eq!(true, attrs.content_may_vary());
        assert_eq!(
            Some(EnumeratedString::from(TimelineOccupies::Point)),
            attrs.timeline_occupies()
        );
        assert_eq!(
            Some(EnumeratedString::from(TimelineStyle::Highlight)),
            attrs.timeline_style()
        );
        assert_eq!(Some(20.0), attrs.skip_control_offset());
        assert_eq!(Some(10.0), attrs.skip_control_duration());
        assert_eq!(Some("skippy"), attrs.skip_control_label_id());
    }

    mutation_tests!(
        Daterange::builder()
            .with_id("some-id")
            .with_start_date(date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_class("com.example.class")
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_end_date(date_time!(2025-06-14 T 23:43:42.000 -05:00))
            .with_duration(120.0)
            .with_planned_duration(180.0)
            .with_scte35_cmd("0xABCD")
            .with_scte35_out("0xABCD")
            .with_scte35_in("0xABCD")
            .finish(),
        (id, "another-id", @Attr="ID=\"another-id\""),
        (start_date, @Option DateTime::default(), @Attr="START-DATE=\"1970-01-01T00:00:00.000Z\""),
        (class, @Option "com.test.class", @Attr="CLASS=\"com.test.class\""),
        (cue, @Option EnumeratedStringList::from([Cue::Once, Cue::Pre]), @Attr="CUE=\"ONCE,PRE\""),
        (end_date, @Option DateTime::default(), @Attr="END-DATE=\"1970-01-01T00:00:00.000Z\""),
        (duration, @Option 60.0, @Attr="DURATION=60"),
        (planned_duration, @Option 80.0, @Attr="PLANNED-DURATION=80"),
        (scte35_cmd, @Option "0x1234", @Attr="SCTE35-CMD=0x1234"),
        (scte35_out, @Option "0x1234", @Attr="SCTE35-OUT=0x1234"),
        (scte35_in, @Option "0x1234", @Attr="SCTE35-IN=0x1234"),
        (end_on_next, true, @Attr="END-ON-NEXT=YES")
    );
}
