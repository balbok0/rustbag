pub(crate) trait MaybeSized {
    fn known_size(&self) -> Option<usize>;
}
