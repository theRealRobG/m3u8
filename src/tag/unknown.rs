#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub fn parse(input: &str) -> Result<Tag, &'static str> {
    let mut chars = input.chars();
    let mut iterations = 0usize;
    let value_is_some = loop {
        iterations += 1;
        let Some(char) = chars.next() else {
            break false;
        };
        match char {
            ':' => break true,
            '\r' | '\n' => break false,
            _ => (),
        }
    };
    let name = &input[..(iterations - 1)];
    if value_is_some {
        let value = &input.trim_end()[iterations..];
        Ok(Tag { name, value })
    } else {
        let value = "";
        Ok(Tag { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_tag_with_no_value() {
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: ""
            }),
            parse("-TEST-TAG")
        );
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: ""
            }),
            parse("-TEST-TAG\r\n")
        );
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: ""
            }),
            parse("-TEST-TAG\n")
        );
    }

    #[test]
    fn parses_tag_with_value() {
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: "42"
            }),
            parse("-TEST-TAG:42")
        );
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: "42"
            }),
            parse("-TEST-TAG:42\r\n")
        );
        assert_eq!(
            Ok(Tag {
                name: "-TEST-TAG",
                value: "42"
            }),
            parse("-TEST-TAG:42\n")
        );
    }
}
