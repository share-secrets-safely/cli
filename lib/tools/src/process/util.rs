use super::types::NeverDrop;
use failure::{err_msg, Error, Fail, ResultExt};
use json;
use process::types::State;
use std::io;
use treediff::{diff, tools};
use yaml;
use yaml_rust;
use yaml_rust::YamlEmitter;

pub fn de_json_or_yaml_document_support<R: io::Read>(mut reader: R, state: &State) -> Result<json::Value, Error> {
    let mut buf = String::new();
    reader
        .read_to_string(&mut buf)
        .context("Could not read input stream as utf8 string for deserialization")?;

    Ok(match json::from_str(&buf) {
        Ok(v) => v,
        Err(json_err) => {
            yaml::from_str(&buf)
                .map_err(|yaml_err| (yaml_err, json_err))
                .or_else(|(yaml_err, json_err)| {
                    yaml_rust::YamlLoader::load_from_str(&buf)
                        .map_err(|rust_yaml_err| {
                            yaml_err
                                .context("YAML deserialization failed")
                                .context(json_err)
                                .context("JSON deserialization failed")
                                .context(rust_yaml_err)
                                .context("rust-yaml deserialization failed")
                                .context("Could not deserialize data, tried JSON and YAML. The data might be malformed")
                        })
                        .and_then(|v| match v.len() {
                            0 => Err(err_msg("Deserialized a YAML without a single value").context("fatal")),
                            1 => {
                                Err(err_msg("We expect single-document yaml files to be read by serde")
                                    .context("fatal"))
                            }
                            _ => {
                                let mut m =
                                    tools::Merger::with_filter(v[0].clone(), NeverDrop::with_mode(&state.merge_mode));
                                for docs in v.as_slice().windows(2) {
                                    diff(&docs[0], &docs[1], &mut m);
                                }
                                if !m.filter().clashed_keys.is_empty() {
                                    return Err(
                                        format_err!("{}", m.filter()).context("The merge failed due to conflicts")
                                    );
                                }
                                Ok(yaml_rust_to_json(&m.into_inner()))
                            }
                        })
                })?
        }
    })
}

fn yaml_rust_to_json(only_document: &yaml_rust::Yaml) -> json::Value {
    let mut buf = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut buf);
        emitter
            .dump(only_document)
            .expect("dumping a valid yaml into a string to work");
    }
    yaml::from_str(&buf).expect("loading a single document to work")
}
