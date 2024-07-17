#[derive(Debug)]
pub struct NetworkInterface {
    name: String,
    address: String,
}

#[derive(Debug)]
pub enum IoError {
    NetworkListError,
}

pub fn List() -> Result<Vec<NetworkInterface>, IoError> {
    Ok(vec![NetworkInterface {
        name: "PlaceHolder".to_string(),
        address: "localhost".to_string(),
    }])
}
