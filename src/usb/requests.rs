#[derive(Debug)]
pub enum Request {
    GetStatus,
    ClearFeature,
    SetFeature,
    SetAddress,
    GetDescriptor,
    SetDescriptor,
    GetConfiguration,
    SetConfiguration,
    GetInterface,
    SetInterface,
    SynchFrame,
    Unknown,
}

impl Request {
    pub fn from_u8(value: u8) -> Request {
        match value {
            0 => Request::GetStatus,
            1 => Request::ClearFeature,
            3 => Request::SetFeature,
            5 => Request::SetAddress,
            6 => Request::GetDescriptor,
            7 => Request::SetDescriptor,
            8 => Request::GetConfiguration,
            9 => Request::SetConfiguration,
            10 => Request::GetInterface,
            11 => Request::SetInterface,
            12 => Request::SynchFrame,
            _ => Request::Unknown,
        }
    }
}
