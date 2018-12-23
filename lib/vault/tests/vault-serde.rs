extern crate serde_yaml;
extern crate sheesy_vault;

use sheesy_vault::TrustModel;
use sheesy_vault::Vault;

#[test]
fn vault_trust_model_serde() {
    let mut v = Vault::default();
    v.trust_model = Some(TrustModel::GpgWebOfTrust);
    let res = serde_yaml::to_string(&v).unwrap();

    assert_eq!(
        res,
        r#"---
name: ~
auto_import: true
trust_model: gpg-web-of-trust
secrets: "."
gpg_keys: ~
recipients: ".gpg-id""#
    );
    assert_eq!(
        serde_yaml::to_string(&serde_yaml::from_str::<Vault>(&res).unwrap()).unwrap(),
        res
    );
}

#[test]
fn default_vault_ser() {
    let v = Vault::default();
    assert_eq!(
        serde_yaml::to_string(&v).unwrap(),
        r#"---
name: ~
auto_import: true
trust_model: ~
secrets: "."
gpg_keys: ~
recipients: ".gpg-id""#
    );
}
