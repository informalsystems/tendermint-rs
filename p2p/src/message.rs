pub enum Receive {
    Pex(PexReceive),
}

pub enum Send {
    Pex(PexSend),
}

pub enum PexReceive {
    Noop,
}

pub trait Outgoing: std::marker::Send + Sync {}

pub enum PexSend {}
impl Outgoing for PexSend {}
