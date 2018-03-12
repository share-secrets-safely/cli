use failure::{Error, Fail, ResultExt};
use std::io;
use json;
use yaml;
use yaml_rust;
use yaml_rust::YamlEmitter;
use treediff::{diff, tools};

pub fn de_json_or_yaml_document_support<R: io::Read>(mut reader: R) -> Result<json::Value, Error> {
    let mut buf = String::new();
    reader
        .read_to_string(&mut buf)
        .context("Could not read input stream as utf8 string for deserialization")?;

    Ok(match json::from_str(&buf) {
        Ok(v) => v,
        Err(json_err) => yaml_rust::YamlLoader::load_from_str(&buf)
            .map_err(|yaml_err| {
                yaml_err
                    .context("YAML deserialization failed")
                    .context(json_err)
                    .context("JSON deserialization failed")
                    .context("Could not deserialize data, tried JSON and YAML")
            })
            .and_then(|v| match v.len() {
                0 => panic!("Deserialized a YAML without a single value"),
                1 => Ok(yaml_rust_to_json(&v[0], &mut String::new())),
                _ => {
                    unimplemented!()
                    //                    let mut m = tools::Merger::with_filter(yamls[0].clone(), NeverDrop::default());
                    //                    for docs in yamls.as_slice().windows(2) {
                    //                        diff(&docs[0], &docs[1], &mut m);
                    //                    }
                    //                    if m.filter().clashed_keys.len() > 0 {
                    //                        return Err(format!("{}", m.filter()).into());
                    //                    }
                }
            })?,
    })
}

fn yaml_rust_to_json(only_document: &yaml_rust::Yaml, buf: &mut String) -> json::Value {
    {
        let mut emitter = YamlEmitter::new(buf);
        emitter
            .dump(only_document)
            .expect("dumping a valid yaml into a string to work");
    }
    yaml::from_str(&buf).expect("loading a single document to work")
}
