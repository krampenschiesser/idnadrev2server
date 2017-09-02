#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EncryptionType {
    None,
    RingChachaPoly1305,
    RingAESGCM,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PasswordHashType {
    None,
    Argon2i { iterations: u16, memory_costs: u16, parallelism: u16 },
    SCrypt { iterations: u8, memory_costs: u32, parallelism: u32 },
}

#[derive(Clone, Eq, PartialEq)]
pub struct PlainPw {
    content: Vec<u8>
}

impl PlainPw {
    pub fn new(pw_plain: &[u8]) -> Self {
        PlainPw { content: pw_plain.to_vec() }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }
}

impl<'a> From<&'a str> for PlainPw {
    fn from(i: &str) -> Self {
        PlainPw::new(i.as_bytes())
    }
}

impl From<String> for PlainPw {
    fn from(i: String) -> Self {
        PlainPw::new(i.as_bytes())
    }
}


impl ::std::fmt::Display for PlainPw {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "PlainPw-no display")
    }
}

impl ::std::fmt::Debug for PlainPw {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "PlainPw-no display")
    }
}