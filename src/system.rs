use crate::value;
use crate::Value;
use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::env;
use sysinfo::SystemExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    #[serde(with = "value")]
    pub total_memory: ByteSize,
    #[serde(with = "value")]
    pub used_memory: ByteSize,
    #[serde(with = "value")]
    pub total_swap: ByteSize,
    #[serde(with = "value")]
    pub used_swap: ByteSize,
    #[serde(with = "value")]
    pub cores: i64,
    #[serde(with = "value")]
    pub os: String,
    #[serde(with = "value")]
    pub os_family: String,
    #[serde(with = "value")]
    pub os_version: String,
    #[serde(with = "value")]
    pub kernel_version: String,
    #[serde(with = "value")]
    pub arch: String,
    #[serde(with = "value")]
    pub benchie_version: String,
}

impl Default for System {
    fn default() -> Self {
        // Please note that we use "new_all" to ensure that all list of
        // components, network interfaces, disks and users are already
        // filled!
        let mut system = sysinfo::System::new_all();
        // First we update all information of our `System` struct.

        system.refresh_all();

        Self {
            total_memory: ByteSize::kb(system.total_memory()),
            used_memory: ByteSize::kb(system.used_memory()),
            total_swap: ByteSize::kb(system.total_swap()),
            used_swap: ByteSize::kb(system.used_swap()),
            cores: system
                .processors()
                .len()
                .try_into()
                .expect("nobody has that many cores, that this would fail"),
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
}

impl<'a> IntoIterator for &'a System {
    type Item = (String, Value);
    type IntoIter = SystemIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SystemIterator {
            system: self,
            index: 0,
        }
    }
}

pub struct SystemIterator<'a> {
    system: &'a System,
    index: usize,
}

impl<'a> Iterator for SystemIterator<'a> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => (
                "total_memory".to_string(),
                Value::ByteSize(self.system.total_memory),
            ),
            1 => (
                "used_memory".to_string(),
                Value::ByteSize(self.system.used_memory),
            ),
            2 => (
                "total_swap".to_string(),
                Value::ByteSize(self.system.total_swap),
            ),
            3 => (
                "used_swap".to_string(),
                Value::ByteSize(self.system.used_swap),
            ),
            4 => ("cores".to_string(), Value::Integer(self.system.cores)),
            5 => ("os".to_string(), Value::String(self.system.os.clone())),
            6 => (
                "os_family".to_string(),
                Value::String(self.system.os_family.clone()),
            ),
            7 => (
                "os_version".to_string(),
                Value::String(self.system.os_version.clone()),
            ),
            8 => (
                "kernel_version".to_string(),
                Value::String(self.system.kernel_version.clone()),
            ),
            9 => ("arch".to_string(), Value::String(self.system.arch.clone())),
            10 => (
                "benchie_version".to_string(),
                Value::String(self.system.benchie_version.clone()),
            ),
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ensure_all_system_values_are_read() {
        let s = System::default();

        assert!(s.total_memory.0 > 0);
        assert!(s.used_memory.0 > 0);
        assert!(s.total_swap.0 > 0);
        assert!(s.used_swap.0 > 0);
        assert!(s.cores > 0);
        assert!(matches!(s.os.as_str(), "macos" | "windows" | "linux"));
        assert!(matches!(s.os_family.as_str(), "unix"));
        assert!(s.os_version.len() > 3 && s.os_version.contains('.'));
        assert!(s.kernel_version.len() > 3 && s.kernel_version.contains('.'));
        assert!(matches!(s.arch.as_str(), "aarch64" | "x86"));
        assert!(s.benchie_version.len() > 3 && s.benchie_version.contains('.'));
    }
}
