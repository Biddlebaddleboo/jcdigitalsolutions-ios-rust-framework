#[cfg(any(target_os = "ios", test))]
use framework_maps::MapAnnotation;

/// An annotation snapshot failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnnotationSnapshotError {
    /// Two markers in one snapshot use the same caller-defined identifier.
    DuplicateIdentifier {
        /// The repeated caller-defined marker id.
        id: u64,
    },
}

#[cfg(any(target_os = "ios", test))]
pub(crate) fn sorted_snapshot<'snapshot, 'title>(
    annotations: &'snapshot [MapAnnotation<'title>],
) -> Result<Vec<&'snapshot MapAnnotation<'title>>, AnnotationSnapshotError> {
    let mut sorted: Vec<_> = annotations.iter().collect();
    sorted.sort_unstable_by_key(|annotation| annotation.id());
    for pair in sorted.windows(2) {
        if pair[0].id() == pair[1].id() {
            return Err(AnnotationSnapshotError::DuplicateIdentifier { id: pair[0].id() });
        }
    }
    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::{AnnotationSnapshotError, sorted_snapshot};
    use framework_maps::{MapAnnotation, MapCoordinate};

    #[test]
    fn snapshot_sorts_distinct_ids_without_changing_values() {
        let coordinate = MapCoordinate::new(43.0, -79.0).unwrap();
        let input = [
            MapAnnotation::new(8, coordinate, Some("east")),
            MapAnnotation::new(2, coordinate, Some("west")),
        ];
        let sorted = sorted_snapshot(&input).unwrap();
        assert_eq!(sorted[0].id(), 2);
        assert_eq!(sorted[1].id(), 8);
        assert_eq!(sorted[0].title(), Some("west"));
        assert_eq!(input[0].id(), 8);
    }

    #[test]
    fn duplicate_ids_reject_snapshot_before_native_changes() {
        let coordinate = MapCoordinate::new(43.0, -79.0).unwrap();
        let input = [
            MapAnnotation::new(4, coordinate, Some("first")),
            MapAnnotation::new(4, coordinate, Some("duplicate")),
        ];
        assert_eq!(
            sorted_snapshot(&input),
            Err(AnnotationSnapshotError::DuplicateIdentifier { id: 4 })
        );
    }
}
