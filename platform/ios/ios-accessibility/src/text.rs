pub(crate) fn map_optional_text<T>(value: Option<&str>, map: impl FnOnce(&str) -> T) -> Option<T> {
    value.map(map)
}
