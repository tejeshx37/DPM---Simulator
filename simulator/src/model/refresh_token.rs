pub trait RefreshToken: Default + Send + Sync + 'static {
    fn refresh(&self);
}
