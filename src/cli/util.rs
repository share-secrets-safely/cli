#[cfg(feature = "process")]
pub fn output_formats() -> &'static [&'static str] {
    &["json", "yaml"]
}
