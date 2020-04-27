use std::fmt::Debug;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub struct BiChanEnd<Req, Res> {
    sender: UnboundedSender<Res>,
    receiver: UnboundedReceiver<Req>,
}

impl<Req, Res> BiChanEnd<Req, Res> {
    pub async fn next_request(&mut self) -> Req {
        self.receiver.recv().await.unwrap() // FIXME: Unwrap
    }

    pub fn reply(&mut self, response: Res)
    where
        Res: Debug,
    {
        self.sender.send(response).expect("could not send response");
    }
}

pub struct BiChan<Req, Res> {
    sender: UnboundedSender<Req>,
    receiver: UnboundedReceiver<Res>,
}

impl<Req, Res> BiChan<Req, Res> {
    pub async fn query(&mut self, request: Req) -> Res
    where
        Req: Debug,
    {
        self.sender.send(request).expect("could not send request");
        self.receiver.recv().await.unwrap() // FIXME: Unwrap
    }
}

pub fn bichan<Req, Res>() -> (BiChan<Req, Res>, BiChanEnd<Req, Res>) {
    let (send_request, recv_request) = unbounded_channel();
    let (send_response, recv_response) = unbounded_channel();

    let chan = BiChan {
        receiver: recv_response,
        sender: send_request,
    };

    let end = BiChanEnd {
        sender: send_response,
        receiver: recv_request,
    };

    (chan, end)
}
