pub mod base64 {
    use base64::engine::general_purpose;
    use base64::Engine;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = general_purpose::STANDARD_NO_PAD.encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        general_purpose::STANDARD_NO_PAD.decode(base64.as_bytes()).map_err(|e| serde::de::Error::custom(e))
    }

    pub fn deserialize_str<'de>(base64: String) -> Result<Vec<u8>, &'static str> {
        general_purpose::STANDARD_NO_PAD.decode(base64).map_err(|_| "base64 err")
    }
}
