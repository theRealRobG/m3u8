use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
#[derive(Debug, PartialEq)]
pub struct ServerControl {
    pub can_skip_until: Option<f64>,
    pub can_skip_dateranges: bool,
    pub hold_back: Option<f64>,
    pub part_hold_back: Option<f64>,
    pub can_block_reload: bool,
}

impl TryFrom<ParsedTagValue<'_>> for ServerControl {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let mut can_skip_until = None;
        let mut can_skip_dateranges = false;
        let mut hold_back = None;
        let mut part_hold_back = None;
        let mut can_block_reload = false;
        for (key, value) in attribute_list {
            match key {
                "CAN-SKIP-UNTIL" => can_skip_until = value.as_option_f64(),
                "CAN-SKIP-DATERANGES" => {
                    can_skip_dateranges = value.as_option_unquoted_str() == Some("YES")
                }
                "HOLD-BACK" => hold_back = value.as_option_f64(),
                "PART-HOLD-BACK" => part_hold_back = value.as_option_f64(),
                "CAN-BLOCK-RELOAD" => {
                    can_block_reload = value.as_option_unquoted_str() == Some("YES")
                }
                _ => (),
            }
        }
        Ok(Self {
            can_skip_until,
            can_skip_dateranges,
            hold_back,
            part_hold_back,
            can_block_reload,
        })
    }
}
