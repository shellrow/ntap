use std::collections::HashMap;

use super::DepsError;

// currently only implemented for windows
// basically no-dependency for unix-likes
pub fn check_deps() -> Result<(), DepsError> {
    Ok(())
}

// currently only implemented for windows
// basically no-dependency for unix-likes
pub fn get_deps_map() -> HashMap<String, bool> {
    HashMap::new()
}
