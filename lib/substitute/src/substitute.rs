use atty;
use failure::{Error, Fail, ResultExt};
use json;
use yaml;
use handlebars::{no_escape, Handlebars};

use std::ffi::OsStr;
use std::io::{stdin, Read};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

pub use spec::*;
use std::io::Cursor;
use std::collections::BTreeSet;

fn validate(data: &StreamOrPath, specs: &[Spec]) -> Result<(), Error> {
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
                (&Path(ref src), &Path(ref dst)) => src.canonicalize()
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

fn de_json_or_yaml<R: Read>(mut reader: R) -> Result<json::Value, Error> {
    let mut buf = Vec::<u8>::new();
    reader
        .read_to_end(&mut buf)
        .context("Could not read input stream data deserialization")?;

    Ok(
        match yaml::from_reader::<_, yaml::Value>(Cursor::new(&buf)) {
            Ok(v) => yaml::from_value(v).context("Could not transform YAML value into JSON")?,
            Err(yaml_err) => json::from_reader(reader).map_err(|json_err| {
                json_err
                    .context("JSON deserialization failed")
                    .context(yaml_err.context("YAML deserialization failed"))
                    .context("Could not deserialize data, tried YAML and JSON")
            })?,
        },
    )
}

pub fn substitute(input_data: StreamOrPath, specs: &[Spec], separator: &OsStr) -> Result<(), Error> {
    use StreamOrPath::*;
    let mut own_specs = Vec::new();

    let (dataset, specs) = match input_data {
        Stream => if atty::is(atty::Stream::Stdin) {
            bail!("Stdin is a TTY. Cannot substitute a template without any data.")
        } else {
            let stdin = stdin();
            let locked_stdin = stdin.lock();
            (de_json_or_yaml(locked_stdin)?, specs)
        },
        Path(ref p) => (
            de_json_or_yaml(File::open(&p).context(format!(
                "Could not open input data file at '{}'",
                p.display()
            ))?)?,
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

    validate(&input_data, specs)?;

    let mut seen_file_outputs = BTreeSet::new();
    let mut seen_writes_to_stdout = 0;
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);
    hbs.register_escape_fn(no_escape);

    for spec in specs {
        let append = match &spec.dst {
            &Path(ref p) => {
                let seen = seen_file_outputs.contains(p);
                seen_file_outputs.insert(p);
                seen
            }
            &Stream => {
                seen_writes_to_stdout += 1;
                false
            }
        };

        let mut istream = spec.src.open_as_input()?;
        hbs.register_template_source(spec.src.short_name(), &mut istream)
            .context(format!(
                "Failed to register handlebars template at '{}'",
                spec.src.name()
            ))?;

        let mut ostream = spec.dst.open_as_output(append)?;
        if seen_writes_to_stdout > 1 || append {
            ostream.write(separator.as_bytes())?;
        }

        hbs.render_to_write(spec.src.short_name(), &dataset, &mut ostream)
            .context(format!(
                "Could instantiate template or writing to '{}' failed",
                spec.dst.name()
            ))?;
    }
    Ok(())
}
