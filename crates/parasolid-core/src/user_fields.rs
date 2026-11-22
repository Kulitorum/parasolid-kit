//! Framing of application-owned integer words, separate from schema fields.

use crate::{ErrorDetails, ErrorKind, ParseError};

// Source: Siemens, Parasolid XT Format Reference, July 2022, section 6.1,
// PDF pp117-119 ("Visible at PK", NOT "Has Node-ID").
// parasolid://documents/parasolid-xt-2022/pages/117
// These are public numeric type/visibility facts, not schema field layouts.
// Absence from the table does not establish invisibility. In particular the
// research targets 110/111/112 remain unknown here, even with an exact catalog.
fn documented_pk_visibility(node_type: u16) -> Option<bool> {
    match node_type {
        10..=19
        | 29..=32
        | 38
        | 50..=54
        | 56
        | 60
        | 67
        | 68
        | 70
        | 80
        | 81
        | 90
        | 100
        | 120
        | 124
        | 130
        | 133
        | 134
        | 137
        | 200
        | 201
        | 222 => Some(true),
        40
        | 41
        | 45
        | 59
        | 74
        | 79
        | 82..=89
        | 91
        | 98
        | 99
        | 101
        | 102
        | 121
        | 122
        | 125..=128
        | 135
        | 136
        | 141
        | 163
        | 176
        | 184
        | 185
        | 189..=193
        | 204..=213
        | 220
        | 221
        | 223
        | 224
        | 229 => Some(false),
        _ => None,
    }
}

pub(crate) fn word_count(
    node_type: u16,
    declared_words: u8,
    offset: usize,
    element_limit: usize,
) -> Result<usize, ParseError> {
    if declared_words == 0 {
        return Ok(0);
    }
    match documented_pk_visibility(node_type) {
        Some(true) => {
            let count = usize::from(declared_words);
            if count > element_limit {
                return Err(ParseError::limit(
                    offset,
                    "user_field_words",
                    count,
                    element_limit,
                ));
            }
            Ok(count)
        }
        Some(false) => Ok(0),
        None => Err(ParseError::new(
            ErrorKind::UnsupportedUserFields,
            offset,
            "user-field framing requires documented PK visibility for this node type",
            ErrorDetails::NodeType { node_type },
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_visibility_is_not_inferred_from_type_ranges() {
        for node_type in [20, 33, 55, 57, 78, 110, 111, 112, 123, 129, 132, 32767] {
            assert_eq!(word_count(node_type, 0, 7, 16), Ok(0));
            let error = word_count(node_type, 1, 7, 16).err();
            assert_eq!(
                error.as_ref().map(ParseError::kind),
                Some(ErrorKind::UnsupportedUserFields)
            );
        }
    }

    #[test]
    fn visibility_is_not_the_node_id_flag() {
        for node_type in [10, 12, 17, 80, 100] {
            assert_eq!(word_count(node_type, 16, 0, 16), Ok(16));
        }
        for node_type in [74, 79, 82, 89, 91, 98, 101, 184] {
            assert_eq!(word_count(node_type, 16, 0, 1), Ok(0));
        }
        assert!(word_count(12, 2, 0, 1).is_err());
    }

    #[test]
    fn frames_additional_nodes_documented_in_july_2022() {
        for node_type in [200, 201, 222] {
            assert_eq!(word_count(node_type, 3, 0, 16), Ok(3));
        }
        for node_type in [176, 185, 189, 190, 191, 192, 193, 204, 205, 206,
            207, 208, 209, 210, 211, 212, 213, 220, 221, 223, 224, 229] {
            assert_eq!(word_count(node_type, 3, 0, 1), Ok(0));
        }
        assert!(word_count(194, 3, 0, 16).is_err());
        assert!(word_count(202, 3, 0, 16).is_err());
    }
}
