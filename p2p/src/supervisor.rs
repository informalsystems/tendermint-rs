//! Supervision of the p2p machinery managing peers and the flow of data from and to them.

use std::collections::{hash_map::Entry, HashMap};
use std::convert::TryFrom as _;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use eyre::{Context, Report, Result};
use flume::{unbounded, Receiver, Sender};

use tendermint::node;

use crate::message;
use crate::peer;
use crate::transport::{self, Connection, Endpoint as _};

mod protocol;
use protocol::{Input, Internal, Output, Protocol};

/// Indicates how a [`transport::Connection`] was established.
pub enum Direction {
    /// Established by accepting a new connection from the [`transport::Transport`].
    Incoming,
    /// Established by issuing a connect on the [`transport::Transport`].
    Outgoing,
}

/// Set of control instructions supported by the [`Supervisor`]. Intended to empower the caller to
/// instruct when to establish new connections and multiplex messages to peers.
pub enum Command {
    /// Accept next incoming connection. As it will unblock the subroutine which is responsible for
    /// accepting even when no incoming connection is pending, the accept can take place at a
    /// later point then when the command was issued. Protocols which rely on hard upper bounds
    /// like the number of concurrently connected peers should issue a disconnect to remedy the
    /// situation.
    Accept,
    /// Establishes a new connection to the remote end in [`transport::ConnectInfo`].
    Connect(transport::ConnectInfo),
    /// Disconnects the [`peer::Peer`] known by [`node::Id`]. This will tear down the entire tree of
    /// subroutines managing the peer in question.
    Disconnect(node::Id),
    /// Dispatch the given message to the peer known for [`node::Id`].
    Msg(node::Id, message::Send),
}

/// Set of significant events in the p2p subsystem.
pub enum Event {
    /// A new connection has been established.
    Connected(node::Id, Direction),
    /// A [`peer::Peer`] has been disconnected.
    Disconnected(node::Id, Report),
    /// A new [`message::Receive`] from the [`peer::Peer`] has arrived.
    Message(node::Id, message::Receive),
    /// The [`Supervisor`] has terminated and should be dropped immediatelly. Optionally carrying
    /// an error which caused the termination.
    Terminated(Option<Report>),
    /// A connection upgraded successfully to a [`peer::Peer`].
    Upgraded(node::Id),
    /// An upgrade from failed.
    UpgradeFailed(node::Id, Report),
}

/// Fatal errors from the supervisor machinery.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error during subroutine execution.
    #[error("subroutine execution")]
    Join(String, Report),
    /// Internal state lock is poisoned and can't be recovered.
    #[error("state lock poisoned")]
    StateLockPoisoned,
}

/// Wrapping a [`transport::Transport`] the `Supervisor` runs the p2p machinery to manage peers over
/// physical connections. Offering multiplexing of ingress and egress messages and a surface to
/// empower higher-level protocols to control the behaviour of the p2p substack.
///
/// TODO(xla): Document subroutine/thread hierarchy and data flow.
pub struct Supervisor {
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

impl Supervisor {
    /// Takes the [`transport::Transport`] and sets up managed subroutines. When the `Supervisor`
    /// is returned the p2p subsystem has been successfully set up on the given network interface
    /// (as far as applicable for the transport) so the caller can use the command input and
    /// consume events.
    ///
    /// # Errors
    ///
    /// * If the bind of the transport fails
    pub fn run<T>(transport: T, info: transport::BindInfo) -> Result<Self>
    where
        T: transport::Transport + Send + 'static,
    {
        let (endpoint, incoming) = transport.bind(info)?;
        let (command_tx, command_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();

        thread::spawn(move || {
            match Self::main::<T>(command_rx, event_tx.clone(), endpoint, incoming) {
                Ok(handles) => {
                    for (_name, handle) in handles {
                        if let Err(_err) = handle.join() {
                            // TODO(xla): Log error of subroutine.
                        }
                    }
                    // TODO(xla): Log completion.
                    // If the receiver is gone, there is nothing to be done about the failed send.
                    event_tx.send(Event::Terminated(None)).ok();
                }
                Err(err) => {
                    // TODO(xla): Log error.
                    // If the receiver is gone, there is nothing to be done about the failed send.
                    event_tx.send(Event::Terminated(Some(err))).ok();
                }
            }
        });

        Ok(Self {
            command_tx,
            event_rx,
        })
    }

    /// Returns the next available message from the underlying channel.
    ///
    /// A `None` signals that the supervisor is stopped and no further events will arrive.
    #[must_use]
    pub fn recv(&self) -> Option<Event> {
        self.event_rx.recv().ok()
    }

    /// Instruct to execute the given [`Command`].
    ///
    /// # Errors
    ///
    /// * If the underlying channels dropped and the receiver is gone and indicating that the
    /// handle the caller holds isn't any good anymore and should be dropped entirely.
    pub fn command(&self, cmd: Command) -> Result<()> {
        self.command_tx.send(cmd).wrap_err("command send failed")
    }
}

type Connected<T> =
    Arc<Mutex<HashMap<node::Id, transport::Direction<<T as transport::Transport>::Connection>>>>;
type Peers<T> = Arc<
    Mutex<HashMap<node::Id, peer::Peer<peer::Running<<T as transport::Transport>::Connection>>>>,
>;

#[allow(clippy::needless_pass_by_value)]
impl Supervisor {
    #[allow(clippy::too_many_lines)]
    fn main<T>(
        command_rx: Receiver<Command>,
        event_tx: Sender<Event>,
        endpoint: <T as transport::Transport>::Endpoint,
        incoming: <T as transport::Transport>::Incoming,
    ) -> Result<Vec<(String, thread::JoinHandle<Result<()>>)>>
    where
        T: transport::Transport + Send + 'static,
    {
        let connected: Connected<T> = Arc::new(Mutex::new(HashMap::new()));
        let peers: Peers<T> = Arc::new(Mutex::new(HashMap::new()));
        let mut handles: Vec<(String, thread::JoinHandle<Result<()>>)> = Vec::new();

        // FIXME(xla): No pipe should be of infinite length. Here the appropriate cap could be the
        // maximal amount of inputs produced without another loop draining partially or fully.
        let (input_tx, input_rx) = unbounded();

        // FIXME(xla): The cap for this channel likely coincides with the maximum of connected
        // peers.
        let (accept_tx, accept_rx) = unbounded::<()>();
        {
            let input_tx = input_tx.clone();
            let connected = connected.clone();
            let name = "supervisor-accept".to_string();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(|| Self::accept::<T>(accept_rx, connected, incoming, input_tx))?;

            handles.push((name, handle));
        }

        // FIXME(xla): The cap for this channel likely coincides with the maximum of connected
        // peers.
        let (connect_tx, connect_rx) = unbounded::<transport::ConnectInfo>();
        {
            let input_tx = input_tx.clone();
            let connected = connected.clone();
            let name = "supervisor-connect".to_string();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(|| Self::connect::<T>(connected, connect_rx, endpoint, input_tx))?;

            handles.push((name, handle));
        }

        // FIXME(xla): Cap here amount of maximum peers by some arbitrary multiplier.
        let (msg_tx, msg_rx) = unbounded::<(node::Id, message::Send)>();
        {
            let input_tx = input_tx.clone();
            let peers = peers.clone();
            let name = "supervisor-message".to_string();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(move || Self::message::<T>(input_tx, msg_rx, peers))?;

            handles.push((name, handle));
        };

        // FIXME(xla): Should be capped by maximum of connected peers.
        let (stop_tx, stop_rx) = unbounded::<node::Id>();
        {
            let input_tx = input_tx.clone();
            let peers = peers.clone();
            let name = "supervisor-stop".to_string();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(move || Self::stop::<T>(input_tx, peers, stop_rx))?;

            handles.push((name, handle));
        }

        // FIXME(xla): Should be capped by maximum of connected peers.
        let (upgrade_tx, upgrade_rx) = unbounded();
        {
            let peers = peers.clone();
            let name = "supervisor-upgrade".to_string();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(move || Self::upgrade::<T>(connected, input_tx, peers, upgrade_rx))?;

            handles.push((name, handle));
        }

        let mut protocol = Protocol::default();

        loop {
            // FIXME(xla): This should yield a Result where in case of error we bail.
            let input = {
                let peers = match peers.lock() {
                    Ok(peers) => peers,
                    Err(_err) => {
                        // FIXME(xla): The lock got poisoned and will stay in that state, we
                        // need to terminate, but should log an error here.
                        break;
                    }
                };

                let mut selector = flume::Selector::new()
                    .recv(&command_rx, |res| match res {
                        Ok(cmd) => Input::Command(cmd),
                        // TODO(xla): The comamnd stream is gone, warrants a bail.
                        Err(flume::RecvError::Disconnected) => todo!(),
                    })
                    .recv(&input_rx, |res| match res {
                        Ok(input) => input,
                        // TODO(xla): The input stream is gone, warrants a bail.
                        Err(flume::RecvError::Disconnected) => todo!(),
                    });

                for (id, peer) in &*peers {
                    selector = selector.recv(&peer.state.receiver, move |res| match res {
                        Ok(msg) => Input::Receive(*id, msg),
                        // TODO(xla): The other end was held by a peer and is gone, it's only safe
                        // to assume that the peer has vanished and needs to be cleaned up.
                        Err(flume::RecvError::Disconnected) => todo!(),
                    });
                }

                selector.wait()
            };

            let res: Result<Vec<()>, flume::TrySendError<()>> = protocol
                .transition(input)
                .into_iter()
                .map(|output| match output {
                    Output::Event(event) => event_tx.try_send(event).map_err(map_try_err),
                    Output::Internal(internal) => match internal {
                        Internal::Accept => accept_tx.try_send(()).map_err(map_try_err),
                        Internal::Connect(info) => connect_tx.try_send(info).map_err(map_try_err),
                        Internal::SendMessage(peer_id, msg) => {
                            msg_tx.try_send((peer_id, msg)).map_err(map_try_err)
                        }
                        Internal::Stop(peer_id) => stop_tx.try_send(peer_id).map_err(map_try_err),
                        Internal::Upgrade(peer_id) => {
                            upgrade_tx.try_send(peer_id).map_err(map_try_err)
                        }
                    },
                })
                .collect();

            match res {
                Ok(_) => {}
                // FIXME(xla): The case becomes relevant as soon as the transition from
                // unbounded to bounded channels is done.
                Err(flume::TrySendError::Full(_)) => todo!(),
                Err(err) => return Err(Report::from(err)),
            }
        }

        Ok(handles)
    }

    fn accept<T>(
        accept_rx: Receiver<()>,
        connected: Connected<T>,
        mut incoming: <T as transport::Transport>::Incoming,
        input_tx: Sender<Input>,
    ) -> Result<()>
    where
        T: transport::Transport + Send + 'static,
    {
        loop {
            accept_rx.recv()?;

            match incoming.next() {
                // Incoming stream is finished, there is nothing left to do for this
                // subroutine.
                None => return Ok(()),
                // TODO(xla): Needs clarification if this error indicates the transport is fallen
                // sideways and should be dropped and therefore the supervisor wrapping it.
                Some(Err(_err)) => todo!(),
                Some(Ok(conn)) => match node::Id::try_from(conn.public_key()) {
                    // TODO(xla): Miigt be a non-issue as the only error variant is an
                    // unsupported key scheme, needs clarification if that path should be
                    // reworked out of the system to avoid creeping in contexts like this one.
                    Err(_err) => todo!(),
                    Ok(id) => {
                        let mut connected =
                            connected.lock().map_err(|_| Error::StateLockPoisoned)?;

                        let msg = match connected.entry(id) {
                            Entry::Vacant(entry) => {
                                entry.insert(transport::Direction::Incoming(conn));
                                Input::Accepted(id)
                            }
                            // If the id in question is already connected we terminate
                            // the duplicate one and inform the protocol of it.
                            Entry::Occupied(_entry) => Input::DuplicateConnRejected(
                                id,
                                Direction::Incoming,
                                conn.close().err(),
                            ),
                        };

                        input_tx.try_send(msg)?;
                    }
                },
            }
        }
    }

    fn connect<T>(
        connected: Connected<T>,
        connect_rx: Receiver<transport::ConnectInfo>,
        endpoint: <T as transport::Transport>::Endpoint,
        input_tx: Sender<Input>,
    ) -> Result<()>
    where
        T: transport::Transport + Send + 'static,
    {
        loop {
            let info = connect_rx.recv()?;

            match endpoint.connect(info) {
                Err(_err) => todo!(),
                Ok(conn) => {
                    match node::Id::try_from(conn.public_key()) {
                        // TODO(xla): Miigt be a non-issue as the only error variant is an
                        // unsupported key scheme, needs clarification if that path should be
                        // reworked out of the system to avoid creeping in contexts like this one.
                        Err(_err) => todo!(),
                        Ok(id) => {
                            let mut connected =
                                connected.lock().map_err(|_| Error::StateLockPoisoned)?;

                            let msg = match connected.entry(id) {
                                Entry::Vacant(entry) => {
                                    entry.insert(transport::Direction::Outgoing(conn));
                                    Input::Connected(id)
                                }
                                // If the id in question is already connected we terminate
                                // the duplicate one and inform the protocol of it.
                                Entry::Occupied(_entry) => Input::DuplicateConnRejected(
                                    id,
                                    Direction::Outgoing,
                                    conn.close().err(),
                                ),
                            };

                            input_tx.try_send(msg)?;
                        }
                    }
                }
            };
        }
    }

    fn message<T>(
        _input_tx: Sender<Input>,
        msg_rx: Receiver<(node::Id, message::Send)>,
        peers: Peers<T>,
    ) -> Result<()>
    where
        T: transport::Transport + Send + 'static,
    {
        loop {
            let (id, msg) = msg_rx.recv()?;

            let peers = peers.lock().map_err(|_| Error::StateLockPoisoned)?;

            match peers.get(&id) {
                // TODO(xla): Ideally acked that the message passed to the peer.
                // FIXME(xla): As the state lock is held up top, it's dangerous if send is
                // ever blocking for any amount of time, which makes this call sensitive to the
                // implementation details of send.
                Some(peer) => match peer.send(msg) {
                    Ok(()) => {}
                    // TODO(xla): Unclear at this point when a send to a peer can fail. Likely
                    // cause here is that the peer is stopped, it should result in a peer stopped
                    // event.
                    Err(_err) => todo!(),
                },
                // TODO(xla): A missing peer needs to be bubbled up as that indicates there is
                // a mismatch between the tracked peers in the protocol and the ones the supervisor holds
                // onto. Something is afoot and it needs to be reconciled asap.
                None => todo!(),
            }
        }
    }

    fn stop<T>(input_tx: Sender<Input>, peers: Peers<T>, stop_rx: Receiver<node::Id>) -> Result<()>
    where
        T: transport::Transport + Send + 'static,
    {
        loop {
            let id = stop_rx.recv()?;

            // To avoid that the lock is held for too long this block is significant.
            let peer = {
                let mut peers = peers.lock().map_err(|_| Error::StateLockPoisoned)?;
                peers.remove(&id)
            };

            let msg = match peer {
                Some(peer) => Input::Stopped(id, peer.stop().state.error),
                None => {
                    // TODO(xla): A missing peer needs to be bubbled up as that indicates there is
                    // a mismatch between the protocol tracked peers and the ones the supervisor holds
                    // onto. Something is afoot and it needs to be reconciled asap.
                    todo!()
                }
            };

            input_tx.try_send(msg)?
        }
    }

    fn upgrade<T>(
        connected: Connected<T>,
        input_tx: Sender<Input>,
        peers: Peers<T>,
        upgrade_rx: Receiver<node::Id>,
    ) -> Result<()>
    where
        T: transport::Transport + Send + 'static,
    {
        loop {
            let peer_id = upgrade_rx.recv()?;
            let mut connected = connected.lock().map_err(|_| Error::StateLockPoisoned)?;

            let msg = match connected.remove(&peer_id) {
                None => Input::UpgradeFailed(peer_id, Report::msg("connection not found")),
                Some(conn) => {
                    match peer::Peer::try_from(conn) {
                        Err(_err) => todo!(),
                        // TODO(xla): Provide actual (possibly configured) list of streams.
                        // FIXME(xla): With the connected lock being held up top, this is sensitive
                        // to the run implementation, it shall not block. Better to not depend on
                        // these details.
                        Ok(peer) => match peer.run(vec![]) {
                            Ok(peer) => {
                                let mut peers =
                                    peers.lock().map_err(|_| Error::StateLockPoisoned)?;
                                match peers.entry(peer.id) {
                                    Entry::Vacant(entry) => {
                                        entry.insert(peer);
                                        Input::Upgraded(peer_id)
                                    }
                                    // TODO(xla): If there is a peer already present in the slot it
                                    // should likely be discarded, alternatively this check should
                                    // happen before the upgrade is executed.
                                    Entry::Occupied(_entry) => todo!(),
                                }
                            }
                            Err(err) => Input::UpgradeFailed(peer_id, err),
                        },
                    }
                }
            };

            input_tx.try_send(msg)?;
        }
    }
}

#[allow(clippy::missing_const_for_fn, clippy::needless_pass_by_value)]
fn map_try_err<T>(err: flume::TrySendError<T>) -> flume::TrySendError<()> {
    match err {
        flume::TrySendError::Disconnected(_) => flume::TrySendError::Disconnected(()),
        flume::TrySendError::Full(_) => flume::TrySendError::Full(()),
    }
}
