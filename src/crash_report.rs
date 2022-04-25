use bytesize::ByteSize;
use std::env;
use std::fmt::{Display, Formatter};
use std::panic;
use sysinfo::{System, SystemExt};
use url::Url;

struct CrashReport {
    panic_log: String,
    total_memory: ByteSize,
    used_memory: ByteSize,
    total_swap: ByteSize,
    used_swap: ByteSize,
    cores: usize,
    os: String,
    os_family: String,
    os_version: String,
    kernel_version: String,
    arch: String,
    benchie_version: String,
}

const GITHUB_NEW_ISSUE_URL: &str = "https://github.com/benchie-io/benchie/issues/new";
const MAX_URL_LENGTH: usize = 4000;

impl CrashReport {
    fn new(panic_info: impl Display) -> Self {
        // Please note that we use "new_all" to ensure that all list of
        // components, network interfaces, disks and users are already
        // filled!
        let mut system = System::new_all();
        // First we update all information of our `System` struct.
        system.refresh_all();

        Self {
            panic_log: format!("{}", panic_info),
            total_memory: ByteSize::kb(system.total_memory()),
            used_memory: ByteSize::kb(system.used_memory()),
            total_swap: ByteSize::kb(system.total_swap()),
            used_swap: ByteSize::kb(system.used_swap()),
            cores: system.processors().len(),
            os: env::consts::OS.to_owned(),
            os_family: env::consts::FAMILY.to_owned(),
            os_version: system.os_version().unwrap_or_else(|| "unknown".to_owned()),
            kernel_version: system
                .kernel_version()
                .unwrap_or_else(|| "unknown".to_owned()),
            arch: env::consts::ARCH.to_owned(),
            benchie_version: option_env!("CARGO_PKG_VERSION")
                .unwrap_or("not found")
                .to_owned(),
        }
    }

    fn without_panic_info() -> Self {
        Self::new("<!-- Please copy and paste any relevant log output. -->".to_string())
    }
}

impl Display for CrashReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hi benchie Team! My benchie program just crashed. This is the report:

## Bug description
<!-- A clear and concise description of what the bug is. -->

## Relevant log output
```
{}
```

## How to reproduce
<!-- Steps to reproduce the behavior -->

## Expected behavior
<!-- A clear and concise description of what you expected to happen. -->

## benchie information
<!-- Your benchie command, .benchie/data.json,... -->

## Environment & Setup
| Name | Value |
|---|--|
| OS | {} |
| OS family | {} |
| OS version | {} |
| kernel version | {} |
| Arch | {} |
| # cores | {} |
| total memory | {} |
| used memory | {} |
| total swap | {} |
| used swap | {} |

## benchie Version
`{}`
",
            self.panic_log,
            self.os,
            self.os_family,
            self.os_version,
            self.kernel_version,
            self.arch,
            self.cores,
            self.total_memory,
            self.used_memory,
            self.total_swap,
            self.used_swap,
            self.benchie_version
        )
    }
}

fn build_github_issue_url(panic_info: &impl Display) -> String {
    let mut base_url = Url::parse(GITHUB_NEW_ISSUE_URL).expect("Github issues URL is valid");

    let url = base_url
        .query_pairs_mut()
        .append_pair("labels", "bug")
        .append_pair("body", &format!("{}", CrashReport::new(panic_info)))
        .finish()
        .to_string();

    if url.len() < MAX_URL_LENGTH {
        url
    } else {
        base_url
            .query_pairs_mut()
            .clear()
            .append_pair("labels", "bug")
            .append_pair("body", &format!("{}", CrashReport::without_panic_info()))
            .finish()
            .to_string()
    }
}

fn build_panic_message(panic_info: &impl Display) -> String {
    let url = build_github_issue_url(panic_info);

    format!(
        "This is a non-recoverable error which probably happens when benchie has a panic.
{}

{}

If you want the benchie team to look into it, please open the link above 🙏
To increase the chance of success, please post your schema and a snippet of
how you used benchie in the issue.
",
        panic_info, url
    )
}

pub fn initialize_crash_reporter() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("{}", build_panic_message(panic_info))
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn panic_message_should_always_contain_panic_info() {
        let msg = build_panic_message(&big_panic_info());

        assert!(
            msg.contains(&big_panic_info()),
            "message should contain a panic info, even if it's large"
        );
        assert!(
            msg.contains(GITHUB_NEW_ISSUE_URL),
            "should contain Github URL"
        );

        let msg = build_panic_message(&small_panic_info());

        assert!(
            msg.contains(&small_panic_info()),
            "message should contain a panic info, even if it's large"
        );
        assert!(
            msg.contains(GITHUB_NEW_ISSUE_URL),
            "should contain Github URL"
        )
    }

    #[test]
    fn github_urls_should_not_exceed_maximum_url_length() {
        let small_url = build_github_issue_url(&small_panic_info());
        let big_url = build_github_issue_url(&big_panic_info());

        assert!(
            small_url.len() < MAX_URL_LENGTH,
            "url should not exceed maximum URL length allowed in modern browsers"
        );
        assert!(
            big_url.len() < MAX_URL_LENGTH,
            "url should not exceed maximum URL length allowed in modern browsers"
        );
    }

    #[test]
    fn panic_info_can_be_omitted_in_github_url_if_too_long() {
        let small_url = build_github_issue_url(&small_panic_info());
        let big_url = build_github_issue_url(&big_panic_info());

        let small_url = Url::parse(&small_url).unwrap();
        let (_, body) = small_url
            .query_pairs()
            .find(|(key, _)| key == "body")
            .unwrap();

        assert!(
            body.contains(&small_panic_info()),
            "small url does contain panic info message"
        );

        let big_url = Url::parse(&big_url).unwrap();
        let (_, body) = big_url
            .query_pairs()
            .find(|(key, _)| key == "body")
            .unwrap();

        assert!(
            !body.contains(&big_panic_info()),
            "big url does not contain panic info message"
        );
    }

    fn big_panic_info() -> String {
        (1..1000)
            .into_iter()
            .fold(String::new(), |res, _| res + " Hello World!")
    }

    fn small_panic_info() -> String {
        "Hello World!".to_string()
    }
}
