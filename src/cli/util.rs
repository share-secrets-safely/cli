#[cfg(any(feature = "process", feature = "rest"))]
pub fn output_formats() -> &'static [&'static str] {
    &["json", "yaml"]
}
