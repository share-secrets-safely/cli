use failure::{err_msg, Error, ResultExt};
use std::fs::File;
use std::io::Read;
use util::ResetCWD;
use base::Vault;
use std::path::{PathBuf, Path};
use glob::glob;
use util::{fingerprint_of, UserIdFingerprint};
use gpgme::{self, Key};
use itertools::Itertools;

impl Vault {
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
