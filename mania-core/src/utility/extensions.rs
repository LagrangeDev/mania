use hex::FromHexError;

pub trait HexString {
    fn hex(&self) -> String;
    fn unhex(&self) -> Result<Vec<u8>, FromHexError>;
}

impl<T: AsRef<[u8]>> HexString for T {
    fn hex(&self) -> String {
        hex::encode(self.as_ref())
    }

    fn unhex(&self) -> Result<Vec<u8>, FromHexError> {
        hex::decode(self.as_ref())
    }
}
