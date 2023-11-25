use chrono::{Duration, TimeZone, Utc};

fn ntp_to_datetime(ntp_time: u64) -> chrono::DateTime<Utc> {
    let int_part: i64 = (ntp_time >> 32).try_into().unwrap();
    let seconds = Duration::seconds(int_part);

    let frac_part: i64 = (ntp_time & u32::MAX as u64).try_into().unwrap();
    let milliseconds =
        Duration::milliseconds(((frac_part as f64 / u32::MAX as f64) * 1000.0) as i64);

    let ntp_base = Utc.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap();

    ntp_base + seconds + milliseconds
}

pub fn ntp_to_string(ntp_time: u64) -> String {
    let time = ntp_to_datetime(ntp_time);

    time.format("%Y-%m-%d %H:%M:%S%.f").to_string()
}

pub fn ntp_to_f64(ntp_time: u64) -> f64 {
    let seconds: i64 = (ntp_time >> 32).try_into().unwrap();
    let frac_part: i64 = (ntp_time & u32::MAX as u64).try_into().unwrap();
    let fraction = frac_part as f64 / u32::MAX as f64;
    seconds as f64 + fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntp_to_datetime_test() {
        let ntp_timestamp: u64 = 0xc59286ee1ba5e354;
        let datetime = ntp_to_datetime(ntp_timestamp);

        let valid_datetime =
            Utc.with_ymd_and_hms(2005, 1, 14, 17, 59, 10).unwrap() + Duration::milliseconds(108);

        assert_eq!(datetime, valid_datetime);
    }

    #[test]
    fn ntp_to_string_test() {
        let ntp_timestamp: u64 = 0xc59286ee1ba5e354;
        let datetime = ntp_to_string(ntp_timestamp);

        let valid_datetime = "2005-01-14 17:59:10.108".to_string();

        assert_eq!(datetime, valid_datetime);
    }
}
