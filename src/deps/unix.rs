use super::DepsError;

// currently only implemented for windows
// basically no-dependency for unix-likes
pub fn check_deps() -> Result<(), DepsError> {
    Ok(())
}
