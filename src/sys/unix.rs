use crate::deps::DepsError;

pub fn check_deps() -> Result<(), DepsError> {
    crate::deps::check_deps()
}
