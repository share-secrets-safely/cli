#[cfg(any(feature = "process", feature = "extract"))]
pub fn output_formats() -> &'static [&'static str] {
    &["json", "yaml"]
}
