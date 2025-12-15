pub type Result<T, E = BoarError> = core::result::Result<T, E>;

#[derive(Debug)]
pub enum BoarError {
    #[allow(dead_code)]
    Script(String),
}
