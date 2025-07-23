use anyhow::Result;
use ndb_oui::OuiDb;

#[cfg(not(feature = "bundled"))]
use std::path::PathBuf;

pub const OUI_CSV_NAME: &str = "oui.csv";
pub const OUI_R2_URL: &str = "https://r2.ntap.io/oui.csv";

#[cfg(not(feature = "bundled"))]
pub fn get_oui_db_filepath() -> Option<PathBuf> {
    match crate::sys::get_database_dir_path() {
        Some(mut db_dir) => {
            db_dir.push(OUI_CSV_NAME);
            Some(db_dir)
        }
        None => {
            eprintln!("Error: Could not get database directory path");
            None
        }
    }
}

#[cfg(feature = "bundled")]
pub fn get_oui_db() -> Result<OuiDb> {
    Ok(OuiDb::bundled())
}

#[cfg(not(feature = "bundled"))]
pub fn get_oui_db() -> Result<OuiDb> {
    let path = get_oui_db_filepath().ok_or_else(|| {
        anyhow::anyhow!("Failed to get OUI database file path")
    })?;
    let reader = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open OUI database file: {}", e))?;
    OuiDb::from_csv(reader)
}
