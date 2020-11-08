pub struct InputBlockId(Vec<u8>);

impl InputBlockId {
    pub fn from_bytes(bytes: &[u8]) -> InputBlockId {
        InputBlockId(Vec::from(bytes))
    }
}
