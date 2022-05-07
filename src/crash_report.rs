use crate::system::System;
use std::fmt::{Display, Formatter};
use std::panic;
use url::Url;

struct CrashReport {
    panic_log: String,
    system: System,
}

const GITHUB_NEW_ISSUE_URL: &str = "https://github.com/benchie-io/benchie/issues/new";
const MAX_URL_LENGTH: usize = 4000;

impl CrashReport {
    fn new(panic_info: impl Display) -> Self {
        Self {
            panic_log: format!("{}", panic_info),
            system: System::default(),
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
            self.system.os,
            self.system.os_family,
            self.system.os_version,
            self.system.kernel_version,
            self.system.arch,
            self.system.cores,
            self.system.total_memory,
            self.system.used_memory,
            self.system.total_swap,
            self.system.used_swap,
            self.system.benchie_version
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
