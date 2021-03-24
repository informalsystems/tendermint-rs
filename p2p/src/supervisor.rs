//! Supervision of the p2p machinery managing peers and the flow of data from and to them.

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom as _;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use eyre::{eyre, Context, Report, Result};
use flume::{unbounded, Receiver, Sender};

use tendermint::node;
use tendermint::public_key::PublicKey;

use crate::message;
use crate::peer;
use crate::transport::{self, Connection, Endpoint as _};

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
    /// A connection upgraded successfully to a [`peer::Peer`].
    Upgraded(node::Id),
    /// An upgrade from failed.
    UpgradeFailed(node::Id, Report),
}

enum Internal {
    Accept,
    Connect(transport::ConnectInfo),
    SendMessage(node::Id, message::Send),
    Stop(node::Id),
    Upgrade(node::Id),
}

enum Input {
    Accepted(node::Id),
    Command(Command),
    Connected(node::Id),
    Receive(node::Id, message::Receive),
    Stopped(node::Id, Option<Report>),
    Upgraded(node::Id),
    UpgradeFailed(node::Id, Report),
}

enum Output {
    Event(Event),
    Internal(Internal),
}

impl From<Event> for Output {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

impl From<Internal> for Output {
    fn from(internal: Internal) -> Self {
        Self::Internal(internal)
    }
}

/// Wrapping a [`transport::Transport`] the `Supervisor` runs the p2p machinery to manage peers over
/// physical connections. Offering multiplexing of ingress and egress messages and a surface to
/// empower higher-level protocols to control the behaviour of the p2p substack.
pub struct Supervisor {
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

struct State<Conn> {
    connected: HashMap<node::Id, transport::Direction<Conn>>,
    peers: HashMap<node::Id, peer::Peer<peer::Running<Conn>>>,
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
    #[allow(clippy::too_many_lines)]
    pub fn run<T>(transport: &T, info: transport::BindInfo) -> Result<Self>
    where
        T: transport::Transport + Send + 'static,
    {
        let (command_tx, command_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let (input_tx, input_rx) = unbounded();
        let supervisor = Self {
            command_tx,
            event_rx,
        };
        let state: Arc<Mutex<State<<T as transport::Transport>::Connection>>> =
            Arc::new(Mutex::new(State {
                connected: HashMap::new(),
                peers: HashMap::new(),
            }));
        let (endpoint, mut incoming) = transport.bind(info)?;

        // ACCEPT
        let (accept_tx, accept_rx) = unbounded::<()>();
        {
            let input_tx = input_tx.clone();
            let state = state.clone();
            thread::spawn(move || loop {
                accept_rx.recv().unwrap();

                let conn = incoming.next().unwrap().unwrap();
                let id = match conn.public_key() {
                    PublicKey::Ed25519(ed25519) => node::Id::from(ed25519),
                    _ => panic!(),
                };
                // TODO(xla): Define and account for the case where a connection is present for the
                // id.
                state
                    .lock()
                    .unwrap()
                    .connected
                    .insert(id, transport::Direction::Incoming(conn))
                    .unwrap();

                input_tx.send(Input::Accepted(id)).unwrap();
            });
        }

        // CONNECT
        let (connect_tx, connect_rx) = unbounded::<transport::ConnectInfo>();
        {
            let input_tx = input_tx.clone();
            let state = state.clone();
            thread::spawn(move || loop {
                let info = connect_rx.recv().unwrap();
                let conn = endpoint.connect(info).unwrap();
                let id = match conn.public_key() {
                    PublicKey::Ed25519(ed25519) => node::Id::from(ed25519),
                    _ => panic!(),
                };

                // TODO(xla): Define and account for the case whwere a connection is present for
                // the id.
                state
                    .lock()
                    .unwrap()
                    .connected
                    .insert(id, transport::Direction::Outgoing(conn))
                    .unwrap();

                input_tx.send(Input::Connected(id)).unwrap();
            });
        }

        // SEND
        let (send_tx, send_rx) = unbounded::<(node::Id, message::Send)>();
        {
            let state = state.clone();
            thread::spawn(move || loop {
                let (id, msg) = send_rx.recv().unwrap();

                match state.lock().unwrap().peers.get(&id) {
                    // TODO(xla): Ideally acked that the message passed to the peer.
                    Some(peer) => peer.send(msg).unwrap(),
                    // TODO(xla): A missing peer needs to be bubbled up as that indicates there is
                    // a mismatch between the protocol tracked peers and the ones the supervisor holds
                    // onto. Something is afoot and it needs to be reconciled asap.
                    None => {}
                }
            });
        }

        // STOP
        let (stop_tx, stop_rx) = unbounded::<node::Id>();
        {
            let input_tx = input_tx.clone();
            let state = state.clone();
            thread::spawn(move || loop {
                let id = stop_rx.recv().unwrap();

                // TOOD(xla): A missing peer needs to be bubbled up as that indicates there is
                // a mismatch between the protocol tracked peers and the ones the supervisor holds
                // onto. Something is afoot and it needs to be reconciled asap.
                if let Some(peer) = state.lock().unwrap().peers.remove(&id) {
                    input_tx
                        .send(Input::Stopped(id, peer.stop().err()))
                        .unwrap();
                }
            });
        }

        // UPGRADE
        let (upgrade_tx, upgrade_rx) = unbounded();
        {
            let input_tx = input_tx.clone();
            let state = state.clone();
            thread::spawn(move || loop {
                let peer_id = upgrade_rx.recv().unwrap();
                let mut state = state.lock().unwrap();

                if let Some(conn) = state.connected.remove(&peer_id) {
                    let peer = peer::Peer::try_from(conn).unwrap();

                    // TODO(xla): Provide actual (possibly configured) list of streams.
                    match peer.run(vec![]) {
                        Ok(peer) => {
                            state.peers.insert(peer.id, peer).unwrap();

                            input_tx.send(Input::Upgraded(peer_id)).unwrap();
                        }
                        Err(err) => {
                            input_tx.send(Input::UpgradeFailed(peer_id, err)).unwrap();
                        }
                    }
                } else {
                    input_tx
                        .send(Input::UpgradeFailed(
                            peer_id,
                            Report::msg("connection not found"),
                        ))
                        .unwrap();
                }
            });
        }

        // MAIN
        thread::spawn(move || {
            let mut protocol = Protocol {
                connected: HashMap::new(),
                stopped: HashSet::new(),
                upgraded: HashSet::new(),
            };

            loop {
                let input = {
                    let state = state.lock().unwrap();
                    let mut selector = flume::Selector::new()
                        .recv(&command_rx, |res| Input::Command(res.unwrap()))
                        .recv(&input_rx, |input| input.unwrap());

                    for (id, peer) in &state.peers {
                        selector = selector.recv(&peer.state.receiver, move |res| {
                            Input::Receive(*id, res.unwrap())
                        });
                    }

                    selector.wait()
                };

                for output in protocol.transition(input) {
                    match output {
                        Output::Event(event) => event_tx.send(event).unwrap(),
                        Output::Internal(internal) => match internal {
                            Internal::Accept => accept_tx.send(()).unwrap(),
                            Internal::Connect(info) => connect_tx.send(info).unwrap(),
                            Internal::SendMessage(peer_id, msg) => {
                                send_tx.send((peer_id, msg)).unwrap()
                            }
                            Internal::Stop(peer_id) => stop_tx.send(peer_id).unwrap(),
                            Internal::Upgrade(peer_id) => upgrade_tx.send(peer_id).unwrap(),
                        },
                    }
                }
            }
        });

        Ok(supervisor)
    }

    /// Returns the next available message from the underlying channel.
    ///
    /// # Errors
    ///
    /// * If
    pub fn recv(&self) -> Result<Event> {
        self.event_rx
            .recv()
            .map_err(|err| eyre!("sender disconnected: {}", err))
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

struct Protocol {
    connected: HashMap<node::Id, Direction>,
    stopped: HashSet<node::Id>,
    upgraded: HashSet<node::Id>,
}

impl Protocol {
    fn transition(&mut self, input: Input) -> Vec<Output> {
        match input {
            Input::Accepted(id) => self.handle_accepted(id),
            Input::Command(command) => self.handle_command(command),
            Input::Connected(id) => self.handle_connected(id),
            Input::Receive(id, msg) => self.handle_receive(id, msg),
            Input::Stopped(id, report) => self.handle_stopped(id, report),
            Input::Upgraded(id) => self.handle_upgraded(id),
            Input::UpgradeFailed(id, err) => self.handle_upgrade_failed(id, err),
        }
    }

    fn handle_accepted(&mut self, id: node::Id) -> Vec<Output> {
        // TODO(xla): Ensure we only allow one connection per node. Unless a higher-level protocol
        // like PEX is taking care of it.
        self.connected.insert(id, Direction::Incoming);

        vec![
            Output::from(Event::Connected(id, Direction::Incoming)),
            Output::from(Internal::Upgrade(id)),
        ]
    }

    fn handle_command(&mut self, command: Command) -> Vec<Output> {
        match command {
            Command::Accept => vec![Output::from(Internal::Accept)],
            Command::Connect(info) => vec![Output::from(Internal::Connect(info))],
            Command::Disconnect(id) => {
                vec![Output::Internal(Internal::Stop(id))]
            }
            Command::Msg(peer_id, msg) => match self.upgraded.get(&peer_id) {
                Some(peer_id) => vec![Output::from(Internal::SendMessage(*peer_id, msg))],
                None => vec![],
            },
        }
    }

    fn handle_connected(&mut self, id: node::Id) -> Vec<Output> {
        // TODO(xla): Ensure we only allow one connection per node. Unless a higher-level protocol
        // like PEX is taking care of it.
        self.connected.insert(id, Direction::Outgoing);

        vec![
            Output::from(Event::Connected(id, Direction::Outgoing)),
            Output::from(Internal::Upgrade(id)),
        ]
    }

    fn handle_receive(&self, id: node::Id, msg: message::Receive) -> Vec<Output> {
        vec![Output::from(Event::Message(id, msg))]
    }

    fn handle_stopped(&mut self, id: node::Id, report: Option<Report>) -> Vec<Output> {
        self.upgraded.remove(&id);
        self.stopped.insert(id);

        vec![Output::from(Event::Disconnected(
            id,
            report.unwrap_or(Report::msg("successfully disconected")),
        ))]
    }

    fn handle_upgraded(&mut self, id: node::Id) -> Vec<Output> {
        self.upgraded.insert(id);

        vec![Output::from(Event::Upgraded(id))]
    }

    fn handle_upgrade_failed(&mut self, id: node::Id, err: Report) -> Vec<Output> {
        self.connected.remove(&id);

        vec![Output::from(Event::UpgradeFailed(id, err))]
    }
}
