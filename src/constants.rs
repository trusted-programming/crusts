#[derive(Debug, Copy, Clone)]
pub struct PlatformConfig {
    pub bear: &'static str,
    pub bear_args: &'static [&'static str],
    pub url: &'static str,
}

#[cfg(target_os = "macos")]
pub static CONFIG: PlatformConfig = PlatformConfig {
    bear: "bear",
    bear_args: &["--", "make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-macosx.tar.gz",
};

#[cfg(target_os = "windows")]
pub static CONFIG: PlatformConfig = PlatformConfig {
    bear: "intercept-build",
    bear_args: &["make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-windows.tar.gz",
};

#[cfg(target_os = "linux")]
pub static CONFIG: PlatformConfig = PlatformConfig {
    bear: "bear",
    bear_args: &["--", "make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-linux.tar.gz",
};

pub const RULES: [&str; 11] = [
    "formalizeCode.x",
    "varTypeNoBounds.x",
    "null.x",
    "array.x",
    "fn.x",
    "errnoLocation.x",
    "atoi.x",
    "time.x",
    "const2mut.x",
    "stdio.x",
    "unsafe.x",
];
