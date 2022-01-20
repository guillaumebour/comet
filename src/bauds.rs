use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use std::time::Duration;

#[derive(Table)]
struct BaudRate {
    #[table(title = "Bauds", justify = "Justify::Left")]
    bauds: i32,
    #[table(title = "Bit duration", justify = "Justify::Left")]
    bit_duration: String,
}

static COMMON_BAUDRATES: &'static [i32] = &[
    50, 75, 110, 134, 150, 200, 300, 600, 1200, 1800, 2400, 4800, 9600, 19200, 28800, 38400, 57600,
    76800, 115200, 230400, 460800, 576000, 921600,
];

// Based on https://lucidar.me/en/serialib/most-used-baud-rates-table/
pub fn display_common_baudrates() {
    let mut baudrates = Vec::new();
    for bd in COMMON_BAUDRATES {
        let nb_sec = 1.0 / f64::from(*bd);
        let bit_duration = Duration::from_secs_f64(nb_sec);
        let bit_duration_str = if bit_duration.as_millis() > 0 {
            format!("{:.3} ms", 1_000.0 * nb_sec)
        } else {
            format!("{:.3} µs", 1_000_000.0 * nb_sec)
        };

        baudrates.push(BaudRate {
            bauds: *bd,
            bit_duration: bit_duration_str,
        })
    }
    print_stdout(baudrates.with_title()).unwrap();
}
