use time::UtcOffset;

pub fn get_local_offset() -> Option<UtcOffset> {
    let local_now: chrono::DateTime<chrono::Local> = chrono::Local::now();
    let offset_seconds = local_now.offset().local_minus_utc();
    match time::UtcOffset::from_whole_seconds(offset_seconds) {
        Ok(offset) => {
            return Some(offset);
        }
        Err(_) => {
            return None;
        }
    }
}