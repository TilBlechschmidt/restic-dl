use std::path::Path;

pub fn path_to_url(path: &Path) -> String {
    path.components()
        .skip_while(|x| x.as_os_str() == "/")
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub mod filters {
    pub fn humanbytes(bytes: &u64) -> ::askama::Result<String> {
        Ok(super::format_size(*bytes))
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1000;
    const MB: u64 = KB * KB;
    const GB: u64 = MB * KB;
    const TB: u64 = GB * KB;

    fn div_round(dividend: u64, divisor: u64) -> u64 {
        (dividend + (divisor >> 1)) / divisor
    }

    let kilobytes = div_round(bytes, KB);
    let megabytes = div_round(bytes, MB);
    let gigabytes = div_round(bytes, GB);
    let terrabytes = div_round(bytes, TB);

    if bytes == 0 {
        format!("Zero bytes")
    } else if bytes < KB {
        format!("{bytes} bytes")
    } else if kilobytes < 1000 {
        format!("{kilobytes} KB")
    } else if megabytes < 1000 {
        format!("{megabytes} MB")
    } else if gigabytes < 100 {
        format!("{} GB", div_round(bytes, MB * 10) as f64 / 100.0)
    } else if gigabytes < 1000 {
        format!("{gigabytes} GB")
    } else if terrabytes < 100 {
        format!("{} TB", div_round(bytes, GB * 10) as f64 / 100.0)
    } else {
        format!("{terrabytes} TB")
    }
}

#[cfg(test)]
mod does {
    use super::format_size as f;

    #[test]
    fn format_byte_counts_correctly() {
        assert_eq!(f(999), "999 bytes");
        assert_eq!(f(1000), "1 KB");

        assert_eq!(f(1499), "1 KB");
        assert_eq!(f(1501), "2 KB");

        assert_eq!(f(999_499), "999 KB");
        assert_eq!(f(999_500), "1 MB");

        assert_eq!(f(999_499_999), "999 MB");
        assert_eq!(f(999_500_500), "1 GB");

        assert_eq!(f(1_420_000_000), "1.42 GB");

        assert_eq!(f(99_420_000_000), "99.42 GB");
        assert_eq!(f(100_420_000_000), "100 GB");

        assert_eq!(f(999_499_000_000), "999 GB");
        assert_eq!(f(999_500_000_000), "1 TB");

        assert_eq!(f(99_420_000_000_000), "99.42 TB");
        assert_eq!(f(99_500_000_000_000), "100 TB");
    }
}
