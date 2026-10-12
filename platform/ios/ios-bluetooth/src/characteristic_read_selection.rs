#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UniqueMatch {
    Missing,
    One(usize),
    Multiple,
}

pub(crate) fn unique_match(
    count: usize,
    mut matches: impl FnMut(usize) -> Result<bool, ()>,
) -> Result<UniqueMatch, ()> {
    let mut found = None;
    for index in 0..count {
        if matches(index)? {
            if found.is_some() {
                return Ok(UniqueMatch::Multiple);
            }
            found = Some(index);
        }
    }
    Ok(found.map_or(UniqueMatch::Missing, UniqueMatch::One))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_match_rejects_absent_and_duplicate_values() {
        let values = [3, 4, 3];
        assert_eq!(
            unique_match(values.len(), |i| Ok(values[i] == 8)),
            Ok(UniqueMatch::Missing)
        );
        assert_eq!(
            unique_match(values.len(), |i| Ok(values[i] == 4)),
            Ok(UniqueMatch::One(1))
        );
        assert_eq!(
            unique_match(values.len(), |i| Ok(values[i] == 3)),
            Ok(UniqueMatch::Multiple)
        );
        assert_eq!(unique_match(values.len(), |_| Err(())), Err(()));
    }
}
