macro_rules! assert_setter_works {
    (
        $init:expr,
        $field:ident,
        $val:expr,
        @Setter=$setter:ident$(($set_val:expr))?,
        @Expected=$expected:expr,
        @Message=$msg:literal,
        @Attr=$(($not:tt))?$attr:literal,
        @AttrMessage=$attr_msg:literal
    ) => {
        let mut test = $init;
        test.$setter($($set_val)?);
        assert_eq!($expected, test.$field(), $msg);
        let inner = test.into_inner();
        let output_line = $crate::utils::str_from(inner.value());
        assert!(
            $($not)?output_line.contains($attr),
            $attr_msg,
            $attr,
            output_line,
        );
    };
}

macro_rules! mutation_test {
    ($init:expr, $field:ident, @Option $val:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<set_ $field>]($val),
                    @Expected=Some($val),
                    @Message="setter failed",
                    @Attr=$attr,
                    @AttrMessage="into_inner for setter failed ({} not found in {})"
                );
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<unset_ $field>],
                    @Expected=None,
                    @Message="unsetter failed",
                    @Attr=(!)$attr,
                    @AttrMessage="into_inner for unsetter failed ({} found in {})"
                );
            }
        }
    };
    ($init:expr, $field:ident, $val:expr; @Default=$default:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<set_ $field>]($val),
                    @Expected=$val,
                    @Message="setter failed",
                    @Attr=$attr,
                    @AttrMessage="into_inner for setter failed ({} not found in {})"
                );
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<unset_ $field>],
                    @Expected=$default,
                    @Message="unsetter failed",
                    @Attr=(!)$attr,
                    @AttrMessage="into_inner for unsetter failed ({} found in {})"
                );
            }
        }
    };
    ($init:expr, $field:ident, $val:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<set_ $field>]($val),
                    @Expected=$val,
                    @Message="setter failed",
                    @Attr=$attr,
                    @AttrMessage="into_inner for setter failed ({} not found in {})"
                );
            }
        }
    };
    ($init:expr, $field:ident, $val:expr => @Expected=$exp:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                $crate::tag::hls::test_macro::assert_setter_works!(
                    $init,
                    $field,
                    $val,
                    @Setter=[<set_ $field>]($val),
                    @Expected=$exp,
                    @Message="setter failed",
                    @Attr=$attr,
                    @AttrMessage="into_inner for setter failed ({} not found in {})"
                );
            }
        }
    };
}

/// Macro expands to validate that all passed in setters (and unsetters) update the value on the
/// initial expression appropriately and validate the output line (into_inner) also includes the
/// expected attribute.
///
/// The single matcher is:
/// ```ignore
/// (
///     $init:expr$
///     (,
///         (
///             $field:ident,
///             $(@$opt:ident )?$val:expr$(; @Default=$default:literal)?,
///             @Attr=$attr:literal
///         )
///     )+
/// )
/// ```
///
/// Parameters explained:
/// * `$init` is an expression to initialize an object to be tested.
/// * `$field` is the name of the getter method on the initialized object.
/// * `$opt` is always equal to `Option` (included as a variable to allow for optional matching).
/// * `$val` is the value to be set and therefore also what we expect the getter method to return
///   after it has been set. When `@Option` is included, it checks for equality on `Some($val)`),
///   and also validates that calling `unset_$field` works as expected too. When `@Default=$default`
///   is included it expects that there is a default value provided by `$field()` when unset, and
///   checks that this is the case when unsetting. `@Option` and `@Default` cannot be used together.
/// * `$default` is the value provided for `$field` when it has been unset.
/// * `$attr` is the literal value expected to be found in the output line (`into_inner().value()`).
///
/// An example usage is:
/// ```ignore
/// mutation_tests!(
///     ContentSteering::builder("server-uri.json")
///         .with_pathway_id("1234")
///         .finish(),
///     (server_uri, "other-steering.json", @Attr="SERVER-URI=\"other-steering.json\""),
///     (pathway_id, @Option "abcd", @Attr="PATHWAY-ID=\"abcd\"")
/// );
/// ```
/// This will unwrap to the following (even with this simple case of just two attributes we've saved
/// a lot of writing and maintenance):
/// ```ignore
/// #[test]
/// fn set_server_uri() {
///     let mut test = ContentSteering::builder("server-uri.json")
///         .with_pathway_id("1234")
///         .finish();
///     test.set_server_uri("other-steering.json");
///     assert_eq!("other-steering.json", test.server_uri(), "setter failed");
///     let inner = test.into_inner();
///     let output_line = crate::utils::str_from(inner.value());
///     assert!(
///         output_line.contains("SERVER-URI=\"other-steering.json\""),
///         "into_inner for setter failed ({} not found in {})",
///         "SERVER-URI=\"other-steering.json\"",
///         output_line,
///     );
/// }
///
/// #[test]
/// fn set_pathway_id() {
///     let mut test = ContentSteering::builder("server-uri.json")
///         .with_pathway_id("1234")
///         .finish();
///     test.set_pathway_id("abcd");
///     assert_eq!(Some("abcd"), test.pathway_id(), "setter failed");
///     let inner = test.into_inner();
///     let output_line = crate::utils::str_from(inner.value());
///     assert!(
///         output_line.contains("PATHWAY-ID=\"abcd\""),
///         "into_inner for setter failed ({} not found in {})",
///         "PATHWAY-ID=\"abcd\"",
///         output_line,
///     );
///
///     let mut test = ContentSteering::builder("server-uri.json")
///         .with_pathway_id("1234")
///         .finish();
///     test.unset_pathway_id();
///     assert_eq!(None, test.pathway_id(), "unsetter failed");
///     let inner = test.into_inner();
///     let output_line = crate::utils::str_from(inner.value());
///     assert!(
///         !output_line.contains("PATHWAY-ID=\"abcd\""),
///         "into_inner for unsetter failed ({} found in {})",
///         "PATHWAY-ID=\"abcd\"",
///         output_line,
///     );
/// }
/// ```
/// An example usage of `@Default` can be found with the `PreloadHint` test where `BYTERANGE-START`
/// has a default value of `0` (per HLS spec):
/// ```ignore
/// mutation_tests!(
///     PreloadHint::builder("PART", "part.2.mp4")
///         .with_byterange_start(512)
///         .with_byterange_length(1024)
///         .finish(),
///     (hint_type, "PART", @Attr="TYPE=PART"),
///     (uri, "part.2.mp4", @Attr="URI=\"part.2.mp4\""),
///     (byterange_start, 100; @Default=0, @Attr="BYTERANGE-START=100"),
///     (byterange_length, @Option 200, @Attr="BYTERANGE-LENGTH=200")
/// );
/// ```
macro_rules! mutation_tests {
    (
        $init:expr$
        (,
            (
                $field:ident,
                $(@$opt:ident )?$val:expr$( => @Expected=$exp:expr)?$(; @Default=$default:expr)?,
                @Attr=$attr:literal
            )
        )+
    ) => {
        $(
            $crate::tag::hls::test_macro::mutation_test!(
                $init, $field, $(@$opt )?$val$( => @Expected=$exp)?$(; @Default=$default)?, @Attr=$attr
            );
        )+
    };
}

pub(crate) use assert_setter_works;
pub(crate) use mutation_test;
pub(crate) use mutation_tests;
