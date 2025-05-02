pub(crate) fn to_i64(value: &u64, _: &dyn askama::Values) -> askama::Result<i64> {
    (*value).try_into()
        .map_err(|e| askama::Error::custom(e))
}
