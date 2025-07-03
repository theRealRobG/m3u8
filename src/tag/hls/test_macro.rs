macro_rules! mutation_test {
    ($init:expr, $field:ident, @Option $val:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                let mut test = $init;
                test.[<set_ $field>]($val);
                assert_eq!(Some($val), test.$field(), "setter failed");
                let inner = test.into_inner();
                let output_line = $crate::utils::str_from(inner.value());
                assert!(
                    output_line.contains($attr),
                    "into_inner for setter failed ({} not found in {})",
                    $attr,
                    output_line,
                );

                let mut test = $init;
                test.[<unset_ $field>]();
                assert_eq!(None, test.$field(), "unsetter failed");
                let inner = test.into_inner();
                let output_line = $crate::utils::str_from(inner.value());
                assert!(
                    !output_line.contains($attr),
                    "into_inner for unsetter failed ({} found in {})",
                    $attr,
                    output_line,
                );
            }
        }
    };
    ($init:expr, $field:ident, $val:expr, @Attr=$attr:literal) => {
        paste::paste! {
            #[test]
            fn [<set_ $field>]() {
                let mut test = $init;
                test.[<set_ $field>]($val);
                assert_eq!($val, test.$field(), "setter failed");
                let inner = test.into_inner();
                let output_line = $crate::utils::str_from(inner.value());
                assert!(
                    output_line.contains($attr),
                    "into_inner for setter failed ({} not found in {})",
                    $attr,
                    output_line,
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
/// ($init:expr$(, ($field:ident, $(@$opt:ident )?$val:expr, @Attr=$attr:literal))+)
/// ```
///
/// Parameters explained:
/// * `$init` is an expression to initialize an object to be tested.
/// * `$field` is the name of the getter method on the initialized object.
/// * `$opt` is always equal to `Option` (included as a variable to allow for optional matching).
/// * `$val` is the value to be set and therefore also what we expect the getter method to return
///   after it has been set. When `@Option` is included, it checks for equality on `Some($val)`),
///   and also validates that calling `unset_$field` works as expected too.
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
macro_rules! mutation_tests {
    ($init:expr$(, ($field:ident, $(@$opt:ident )?$val:expr, @Attr=$attr:literal))+) => {
        $(
            $crate::tag::hls::test_macro::mutation_test!($init, $field, $(@$opt )?$val, @Attr=$attr);
        )+
    };
}

pub(crate) use mutation_test;
pub(crate) use mutation_tests;
