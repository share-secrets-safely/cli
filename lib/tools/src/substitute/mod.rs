mod spec;

mod util;

use atty;
use json;
use yaml_rust;
use failure::{err_msg, Error, ResultExt};
use liquid;
use handlebars::Handlebars;

use std::ffi::OsStr;
use std::io::{self, stdin};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

pub use self::spec::*;
pub use self::util::Engine;
use self::util::{de_json_or_yaml, validate, EngineChoice};
use std::collections::BTreeSet;

pub fn substitute(
    engine: Engine,
    input_data: &StreamOrPath,
    specs: &[Spec],
    separator: &OsStr,
    try_deserialize: bool,
    replacements: &[(String, String)],
) -> Result<(), Error> {
    use self::StreamOrPath::*;
    let mut own_specs = Vec::new();

    let (dataset, specs) = match *input_data {
        Stream => if atty::is(atty::Stream::Stdin) {
            bail!("Stdin is a TTY. Cannot substitute a template without any data.")
        } else {
            let stdin = stdin();
            let locked_stdin = stdin.lock();
            (de_json_or_yaml(locked_stdin)?, specs)
        },
        Path(ref p) => (
            de_json_or_yaml(File::open(&p).context(format!("Could not open input data file at '{}'", p.display()))?)?,
            if specs.is_empty() {
                own_specs.push(Spec {
                    src: Stream,
                    dst: Stream,
                });
                &own_specs
            } else {
                specs
            },
        ),
    };

    validate(input_data, specs)?;
    let dataset = substitute_in_data(dataset, replacements);
    let dataset = into_liquid_object(dataset)?;

    let mut seen_file_outputs = BTreeSet::new();
    let mut seen_writes_to_stdout = 0;
    let liquid = liquid::ParserBuilder::with_liquid().build();
    let mut buf = Vec::<u8>::new();
    let mut ibuf = String::new();

    for spec in specs {
        let append = match spec.dst {
            Path(ref p) => {
                let seen = seen_file_outputs.contains(p);
                seen_file_outputs.insert(p);
                seen
            }
            Stream => {
                seen_writes_to_stdout += 1;
                false
            }
        };

        let mut istream = spec.src.open_as_input()?;
        ibuf.clear();
        istream.read_to_string(&mut ibuf)?;

        let tpl = liquid.parse(&ibuf).map_err(|err| {
            format_err!("{}", err).context(format!("Failed to parse liquid template at '{}'", spec.src.name()))
        })?;

        let mut ostream = spec.dst.open_as_output(append)?;
        if seen_writes_to_stdout > 1 || append {
            ostream.write_all(separator.as_bytes())?;
        }

        {
            let ostream_for_template: &mut io::Write = if try_deserialize { &mut buf } else { &mut ostream };

            let rendered = tpl.render(&dataset).map_err(|err| {
                format_err!("{}", err).context(format!(
                    "Failed to render template from template at '{}'",
                    spec.src.short_name()
                ))
            })?;
            ostream_for_template.write_all(rendered.as_bytes())?;
        }

        if try_deserialize {
            {
                let str_buf = ::std::str::from_utf8(&buf).context(format!(
                    "Validation of template output at '{}' failed as it was not valid UTF8",
                    spec.dst.name()
                ))?;
                yaml_rust::YamlLoader::load_from_str(str_buf).context(format!(
                    "Validation of template output at '{}' failed. It's neither valid YAML, nor JSON",
                    spec.dst.name()
                ))?;
            }
            let mut read = io::Cursor::new(buf);
            io::copy(&mut read, &mut ostream)
                .map_err(|_| err_msg("Failed to output validated template to destination."))?;
            buf = read.into_inner();
            buf.clear();
        }
    }
    Ok(())
}

fn into_liquid_object(src: json::Value) -> Result<liquid::Object, Error> {
    let dst = json::from_value(src)?;
    match dst {
        liquid::Value::Object(obj) => Ok(obj),
        _ => Err(err_msg("Data model root must be an object")),
    }
}

fn substitute_in_data(mut d: json::Value, r: &[(String, String)]) -> json::Value {
    if r.is_empty() {
        return d;
    }

    {
        use json::Value::*;
        let mut stack = vec![&mut d];
        while let Some(v) = stack.pop() {
            match *v {
                String(ref mut s) => {
                    *s = r.iter()
                        .fold(s.to_owned(), |s, &(ref f, ref t)| s.replace(f.as_str(), t))
                }
                Array(ref mut v) => stack.extend(v.iter_mut()),
                Object(ref mut m) => stack.extend(m.iter_mut().map(|(_, v)| v)),
                _ => continue,
            }
        }
    }

    d
}
