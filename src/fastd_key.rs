use hex::FromHex;
use recap::Recap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Recap)]
#[recap(regex = r#"(?m)^key\s*"(?P<key>[0-9a-fA-F]{64})";"#)]
struct FastdKeyConfigfileForm {
    #[serde(with = "hex")]
    key: [u8; 32],
}

pub(crate) fn parse_from_raw(line: &str) -> Option<[u8; 32]> {
    match <[u8; 32]>::from_hex(line.trim()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

pub(crate) fn parse_from_config(line: &str) -> Option<[u8; 32]> {
    match line.parse::<FastdKeyConfigfileForm>() {
        Ok(parsed) => Some(parsed.key),
        Err(_) => None,
    }
}
