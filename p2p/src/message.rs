pub enum Receive {
    Pex(PexReceive),
}

pub enum Send {
    Pex(PexSend),
}

pub enum PexReceive {
    Noop,
}

pub enum PexSend {}
