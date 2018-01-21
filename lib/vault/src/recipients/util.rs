use failure::{Fail, err_msg, Error, ResultExt};
use std::fs::File;
use std::io::{Write, Read};
use util::ResetCWD;
use base::Vault;
use std::path::{PathBuf, Path};
use glob::glob;
use util::{fingerprint_of, UserIdFingerprint};
use gpgme::{self, Key};
use itertools::Itertools;

fn valid_fingerprint(id: &str) -> Result<&str, Error> {
    if id.len() < 8 || id.len() > 40 {
        return Err(format_err!(
            "Fingerprint '{}' must be between 8 and 40 characters long.",
            id
        ));
    }
    let has_only_fingerprint_chars = id.chars().all(|c| match c {
        'a'...'f' | 'A'...'F' | '0'...'9' => true,
        _ => false,
    });

    if has_only_fingerprint_chars {
        Ok(id)
    } else {
        Err(format_err!(
            "Fingerprint '{}' must only contain characters a-f, A-F and 0-9.",
            id
        ))
    }
}

impl Vault {
    pub fn import_keys(
        &self,
        gpg_ctx: &mut gpgme::Context,
        gpg_keys_dir: &Path,
        gpg_key_ids: &[String],
        output: &mut Write,
    ) -> Result<Vec<String>, Error> {
        let imported_gpg_keys_fprs = gpg_key_ids
            .iter()
            .map(|s| {
                valid_fingerprint(&s)
                    .and_then(|fpr| self.read_fingerprint_file(&fpr, &gpg_keys_dir))
                    .and_then(|(fpr_path, kb)| {
                        let res = gpg_ctx.import(kb).map_err(|e| {
                            e.context(format!(
                                "Could not import key to gpg key database from content of file at '{}'",
                                fpr_path.display()
                            )).into()
                        });
                        writeln!(
                            output,
                            "Imported recipient key at path '{}'",
                            fpr_path.display()
                        ).ok();
                        res
                    })
            })
            .fold(Ok(Vec::new()), |r, k| match k {
                Ok(imports) => {
                    r.map(|mut v| {
                        v.extend(imports.imports().filter_map(|i| {
                            i.fingerprint().map(ToOwned::to_owned).ok()
                        }));
                        v
                    })
                }
                Err(e) => {
                    match r {
                        Ok(_) => Err(e),
                        r @ Err(_) => r.map_err(|f| format_err!("{}\n{}", e, f)),
                    }
                }
            })?;
        if imported_gpg_keys_fprs.len() < gpg_key_ids.len() {
            panic!(
                "You should come and take a look at this! It should not be possible \
                        to successfully import less keys than specified."
            )
        }

        {
            let mut extra_keys = imported_gpg_keys_fprs.clone();
            extra_keys.retain(|k| !gpg_key_ids.iter().any(|ok| k.ends_with(ok)));
            if !extra_keys.is_empty() {
                return Err(format_err!(
                    "One of the imported key-files contained more than one recipient.\n\
                This might mean somebody trying to sneak in their key. The offending fingerprints are listed below\n\
                {}",
                    extra_keys.join("\n")
                ));
            }
        }
        Ok(imported_gpg_keys_fprs)
    }

    pub fn find_signing_key(&self, ctx: &mut gpgme::Context, signing_key_id: Option<&str>) -> Result<Key, Error> {
        let recipients_fprs = self.recipients_list().context(
            "A recipients list is needed assure the signing key is in the recipients list.",
        )?;
        if recipients_fprs.is_empty() {
            return Err(err_msg(
                "The recipients list is empty, but you are expected to be on that list.",
            ));
        }
        let key_is_in_recipients_list = |(k, fpr)| if recipients_fprs.iter().any(|rfpr| rfpr == &fpr) {
            Some(k)
        } else {
            None
        };
        let signing_key_fpr = match signing_key_id {
            Some(id) => Some(ctx.find_key(id)
                .map_err(Into::into)
                .and_then(|k| fingerprint_of(&k))
                .context(format!(
                    "The given signing key named '{}' could not be found in the keychain.",
                    id
                ))?),
            None => None,
        };
        let only_matching_signing_key = |(k, fpr)| match signing_key_fpr.as_ref() {
            Some(sk_fpr) => if &fpr == sk_fpr { Some((k, fpr)) } else { None },
            None => Some((k, fpr)),
        };
        let mut signing_keys: Vec<_> = ctx.find_secret_keys(None::<String>)?
            .filter_map(Result::ok)
            .filter_map(|k| fingerprint_of(&k).map(|fpr| (k, fpr)).ok())
            .filter_map(only_matching_signing_key)
            .filter_map(key_is_in_recipients_list)
            .collect();
        match signing_keys.len() {
            0 => Err(err_msg(
                "Didn't find a single secret key suitable to sign keys.",
            )),
            1 => Ok(signing_keys.pop().expect("one entry")),
            _ => Err(
                format_err!("Multiple keys are suitable for signing, which is ambiguous.\n{}",
                signing_keys
                    .iter()
                    .map(|sk| format!("{}", UserIdFingerprint(sk)))
                    .join("\n"),
            ),
            ),
        }
    }

    pub fn read_fingerprint_file(&self, fpr: &str, gpg_keys_dir: &Path) -> Result<(PathBuf, Vec<u8>), Error> {
        let fpr_path = if fpr.len() == 40 {
            gpg_keys_dir.join(fpr)
        } else {
            let _cwd = ResetCWD::new(gpg_keys_dir)?;
            let glob_pattern = format!("*{}", fpr);
            let matching_paths: Vec<_> = glob(&glob_pattern)
                .expect("valid pattern")
                .filter_map(Result::ok)
                .collect();
            match matching_paths.len() {
                1 => gpg_keys_dir.join(&matching_paths[0]),
                0 => {
                    bail!(
                        "Did not find key file matching glob pattern '{}' in directory '{}'.",
                        glob_pattern,
                        gpg_keys_dir.display()
                    )
                }
                l @ _ => {
                    bail!(
                        "Found {} matching key files for glob pattern '{}' in directory '{}', but expected just one.",
                        l,
                        glob_pattern,
                        gpg_keys_dir.display()
                    )
                }
            }
        };
        let mut buf = Vec::new();
        File::open(&fpr_path)
            .context(format!(
                "Could not open key file '{}' for reading",
                fpr_path.display()
            ))
            .and_then(|mut f| {
                f.read_to_end(&mut buf).context(format!(
                    "Could not read key file at '{}'.",
                    fpr_path.display()
                ))
            })?;
        Ok((fpr_path, buf))
    }
}
