use failure::{Error, Fail, ResultExt};
use handlebars;
use json;
use liquid;
use yaml;

use std::io::Read;

pub use super::spec::*;
use std::io::Cursor;
use std::str::FromStr;

pub mod liquid_filters {
    use liquid::compiler::Filter;
    use liquid::derive::*;
    use liquid::error::Result;
    use liquid::interpreter::Context;
    use liquid::value::Value;

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(name = "base64", description = "convert a string to base64", parsed(Base64Filter))]
    pub struct Base64;

    #[derive(Debug, Default, Display_filter)]
    #[name = "base64"]
    struct Base64Filter;

    impl Filter for Base64Filter {
        fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
            Ok(Value::scalar(base64::encode(input.to_str().as_bytes())))
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Engine {
    Handlebars,
    Liquid,
}

impl FromStr for Engine {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(match s {
            "liquid" => Engine::Liquid,
            "handlebars" => Engine::Handlebars,
            _ => bail!("Engine named '{}' is unknown", s),
        })
    }
}

pub enum EngineChoice<'a> {
    Handlebars(handlebars::Handlebars<'a>, json::Value),
    Liquid(liquid::Parser, liquid::value::Object),
}

pub fn validate(data: StreamOrPath, specs: &[Spec]) -> Result<(), Error> {
    if specs.is_empty() {
        bail!("No spec provided, neither from standard input, nor from file")
    }
    let count_of_specs_needing_stdin = specs.iter().filter(|s| s.src.is_stream()).count();
    if count_of_specs_needing_stdin > 1 {
        bail!("Cannot read more than one template spec from standard input")
    }
    if data.is_stream() && count_of_specs_needing_stdin == 1 {
        bail!("Data is read from standard input, as well as one template. Please choose one")
    }
    if let Some(spec_which_overwrites_input) = specs
        .iter()
        .filter_map(|s| {
            use self::StreamOrPath::*;
            match (&s.src, &s.dst) {
                (&Path(ref src), &Path(ref dst)) => src
                    .canonicalize()
                    .and_then(|src| dst.canonicalize().map(|dst| (src, dst)))
                    .ok()
                    .and_then(|(src, dst)| if src == dst { Some(s) } else { None }),
                _ => None,
            }
        })
        .next()
    {
        bail!(
            "Refusing to overwrite input file at '{}' with output",
            spec_which_overwrites_input.src
        )
    }
    Ok(())
}

pub fn de_json_or_yaml<R: Read>(mut reader: R) -> Result<json::Value, Error> {
    let mut buf = Vec::<u8>::new();
    reader
        .read_to_end(&mut buf)
        .context("Could not read input stream data deserialization")?;

    Ok(match json::from_reader::<_, json::Value>(Cursor::new(&buf)) {
        Ok(v) => v,
        Err(json_err) => yaml::from_reader(Cursor::new(&buf)).map_err(|yaml_err| {
            yaml_err
                .context("YAML deserialization failed")
                .context(json_err)
                .context("JSON deserialization failed")
                .context("Could not deserialize data, tried JSON and YAML")
        })?,
    })
}
