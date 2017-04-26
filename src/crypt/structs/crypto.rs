
#[derive(Clone)]
pub struct PlainPw {
    content: Vec<u8>
}
#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}
#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
}