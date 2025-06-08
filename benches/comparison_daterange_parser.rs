use std::{error::Error, fmt::Display};

#[derive(Default, Debug)]
pub struct DaterangeInformation {
    pub end_index: usize,
    pub class_attribute_exists: bool,
    pub scte35_out_start_index: Option<usize>,
    pub scte35_out_end_index: Option<usize>,
    pub id_start_index: Option<usize>,
    pub id_end_index: Option<usize>,
}

pub fn parse(line: &str) -> Option<DaterangeInformation> {
    let mut state = TagURIParsingState::default();
    let mut info = DaterangeInformation::default();
    let mut chars_count = 0;
    let chars = line.chars().enumerate();

    for (index, char) in chars {
        chars_count += 1;
        match index {
            0..=16 => continue,
            17 => {
                if &line[..17] == DATERANGE_START {
                    state = TagURIParsingState::StartOfNewAttributeName;
                } else {
                    return None;
                }
                if handle_char(&mut state, index, char, &mut info).is_err() {
                    break;
                }
            }
            _ => {
                if handle_char(&mut state, index, char, &mut info).is_err() {
                    break;
                }
            }
        }
    }

    Some(info)
}

#[derive(Debug)]
enum LineParsingError {
    /// The line parsing broke early (due to carriage return or new line).
    EarlyBreak,
}
impl Display for LineParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineParsingError::EarlyBreak => write!(
                f,
                "line parsing broke early (due to carriage return or new line)."
            ),
        }
    }
}
impl Error for LineParsingError {}

#[derive(Default)]
enum TagURIParsingState {
    StartOfNewAttributeName,
    #[default]
    ReadingDaterange,
    LookingForIdNameD,
    LookingForIdEquals,
    LookingForIdOpenQuote,
    LookingForIdCloseQuote,
    LookingForClassNameL,
    LookingForClassNameA,
    LookingForClassNameFirstS,
    LookingForClassNameSecondS,
    LookingForClassEquals,
    LookingForClassOpenQuote,
    LookingForClassCloseQuote,
    LookingForScteNameC,
    LookingForScteNameFirstT,
    LookingForScteNameE,
    LookingForScteName3,
    LookingForScteName5,
    LookingForScteNameHyphen,
    LookingForScteNameO,
    LookingForScteNameU,
    LookingForScteNameSecondT,
    LookingForScteEquals,
    // There is a bug with Harmonic streams where SCTE35-OUT is a quoted string instead of a
    // hexadecimal sequence, so we need to consider both possibilities, and thus we include the
    // `LookingForScteOpenQuote` case and the associated `LookingForScteCloseQuote` case.
    LookingForScteOpenQuote,
    LookingForScteCloseQuote,
    LookingForScteEnd,
    IgnoringUntilCloseQuote,
}

impl TagURIParsingState {
    fn reset(&mut self) {
        *self = TagURIParsingState::default();
    }
}

fn handle_char(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match state {
        TagURIParsingState::ReadingDaterange => reading_daterange(state, index, char, info),
        TagURIParsingState::StartOfNewAttributeName => {
            start_of_new_attribute_name(state, index, char, info)
        }
        TagURIParsingState::LookingForIdNameD => looking_for_id_name_d(state, index, char, info),
        TagURIParsingState::LookingForIdEquals => looking_for_id_equals(state, index, char, info),
        TagURIParsingState::LookingForIdOpenQuote => {
            looking_for_id_open_quote(state, index, char, info)
        }
        TagURIParsingState::LookingForIdCloseQuote => {
            looking_for_id_close_quote(state, index, char, info)
        }
        TagURIParsingState::LookingForClassNameL => {
            looking_for_class_name_l(state, index, char, info)
        }
        TagURIParsingState::LookingForClassNameA => {
            looking_for_class_name_a(state, index, char, info)
        }
        TagURIParsingState::LookingForClassNameFirstS => {
            looking_for_class_name_first_s(state, index, char, info)
        }
        TagURIParsingState::LookingForClassNameSecondS => {
            looking_for_class_name_second_s(state, index, char, info)
        }
        TagURIParsingState::LookingForClassEquals => {
            looking_for_class_equals(state, index, char, info)
        }
        TagURIParsingState::LookingForClassOpenQuote => {
            looking_for_class_open_quote(state, index, char, info)
        }
        TagURIParsingState::LookingForClassCloseQuote => {
            looking_for_class_close_quote(state, char, info)
        }
        TagURIParsingState::LookingForScteNameC => {
            looking_for_scte_name_c(state, index, char, info)
        }
        TagURIParsingState::LookingForScteNameFirstT => {
            looking_for_scte_name_first_t(state, index, char, info)
        }
        TagURIParsingState::LookingForScteNameE => {
            looking_for_scte_name_e(state, index, char, info)
        }
        TagURIParsingState::LookingForScteName3 => looking_for_scte_name3(state, index, char, info),
        TagURIParsingState::LookingForScteName5 => looking_for_scte_name5(state, index, char, info),
        TagURIParsingState::LookingForScteNameHyphen => {
            looking_for_scte_name_hyphen(state, index, char, info)
        }
        TagURIParsingState::LookingForScteNameO => {
            looking_for_scte_name_o(state, index, char, info)
        }
        TagURIParsingState::LookingForScteNameU => {
            looking_for_scte_name_u(state, index, char, info)
        }
        TagURIParsingState::LookingForScteNameSecondT => {
            looking_for_scte_name_second_t(state, index, char, info)
        }
        TagURIParsingState::LookingForScteEquals => {
            looking_for_scte_equals(state, index, char, info)
        }
        TagURIParsingState::LookingForScteOpenQuote => {
            looking_for_scte_open_quote(state, index, char, info)
        }
        TagURIParsingState::LookingForScteCloseQuote => {
            looking_for_scte_close_quote(state, index, char, info)
        }
        TagURIParsingState::LookingForScteEnd => looking_for_scte_end(state, index, char, info),
        TagURIParsingState::IgnoringUntilCloseQuote => ignoring_until_close_quote(state, char),
    }
}

const DATERANGE_START: &str = "#EXT-X-DATERANGE:";
const COMMON_CHARS: [char; 4] = [',', '\r', '\n', '\"'];

fn handle_common_char(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        ',' => *state = TagURIParsingState::StartOfNewAttributeName,
        '\r' | '\n' => {
            info.end_index = index;
            return Err(LineParsingError::EarlyBreak);
        }
        '\"' => *state = TagURIParsingState::IgnoringUntilCloseQuote,
        _ => state.reset(),
    }
    Ok(())
}

fn reading_daterange(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn start_of_new_attribute_name(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'I' => *state = TagURIParsingState::LookingForIdNameD,
        'C' => *state = TagURIParsingState::LookingForClassNameL,
        'S' => *state = TagURIParsingState::LookingForScteNameC,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_id_name_d(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'D' => *state = TagURIParsingState::LookingForIdEquals,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_id_equals(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '=' => *state = TagURIParsingState::LookingForIdOpenQuote,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_id_open_quote(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '\"' => {
            info.id_start_index = Some(index + 1);
            *state = TagURIParsingState::LookingForIdCloseQuote;
        }
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_id_close_quote(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    if char == '\"' {
        info.id_end_index = Some(index);
        state.reset();
    }
    Ok(())
}

fn looking_for_class_name_l(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'L' => *state = TagURIParsingState::LookingForClassNameA,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_name_a(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'A' => *state = TagURIParsingState::LookingForClassNameFirstS,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_name_first_s(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'S' => *state = TagURIParsingState::LookingForClassNameSecondS,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_name_second_s(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'S' => *state = TagURIParsingState::LookingForClassEquals,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_equals(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '=' => *state = TagURIParsingState::LookingForClassOpenQuote,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_open_quote(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '\"' => *state = TagURIParsingState::LookingForClassCloseQuote,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_class_close_quote(
    state: &mut TagURIParsingState,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    if char == '\"' {
        info.class_attribute_exists = true;
        state.reset();
    }
    Ok(())
}

fn looking_for_scte_name_c(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'C' => *state = TagURIParsingState::LookingForScteNameFirstT,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_first_t(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'T' => *state = TagURIParsingState::LookingForScteNameE,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_e(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'E' => *state = TagURIParsingState::LookingForScteName3,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name3(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '3' => *state = TagURIParsingState::LookingForScteName5,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name5(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '5' => *state = TagURIParsingState::LookingForScteNameHyphen,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_hyphen(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '-' => *state = TagURIParsingState::LookingForScteNameO,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_o(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'O' => *state = TagURIParsingState::LookingForScteNameU,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_u(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'U' => *state = TagURIParsingState::LookingForScteNameSecondT,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_name_second_t(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        'T' => *state = TagURIParsingState::LookingForScteEquals,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_equals(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '=' => {
            info.scte35_out_start_index = Some(index + 1);
            *state = TagURIParsingState::LookingForScteOpenQuote;
        }
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_open_quote(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        '\"' => {
            info.scte35_out_start_index = Some(index + 1);
            *state = TagURIParsingState::LookingForScteCloseQuote;
        }
        '0' => *state = TagURIParsingState::LookingForScteEnd,
        c if COMMON_CHARS.contains(&c) => handle_common_char(state, index, char, info)?,
        _ => state.reset(),
    }
    Ok(())
}

fn looking_for_scte_close_quote(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    if char == '\"' {
        info.scte35_out_end_index = Some(index);
        state.reset();
    }
    Ok(())
}

fn looking_for_scte_end(
    state: &mut TagURIParsingState,
    index: usize,
    char: char,
    info: &mut DaterangeInformation,
) -> Result<(), LineParsingError> {
    match char {
        ',' => {
            info.scte35_out_end_index = Some(index);
            *state = TagURIParsingState::StartOfNewAttributeName;
        }
        '\r' | '\n' => {
            info.scte35_out_end_index = Some(index);
            info.end_index = index;
            return Err(LineParsingError::EarlyBreak);
        }
        _ => (),
    }
    Ok(())
}

fn ignoring_until_close_quote(
    state: &mut TagURIParsingState,
    char: char,
) -> Result<(), LineParsingError> {
    if char == '\"' {
        state.reset();
    }
    Ok(())
}
