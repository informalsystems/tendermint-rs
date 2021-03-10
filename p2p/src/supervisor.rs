use std::collections::{HashMap, HashSet};
use std::convert::TryFrom as _;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;

use eyre::{eyre, Context, Report, Result};
use flume::{unbounded, Receiver, Sender};

use tendermint::node;
use tendermint::public_key::PublicKey;

use crate::message;
use crate::peer;
use crate::transport::{self, Connection, Endpoint as _};

pub enum Direction {
    Incoming,
    Outgoing,
}

pub enum Command {
    Accept,
    Connect(SocketAddr),
    Disconnect(node::Id),
    Msg(node::Id, message::Send),
}

pub enum Event {
    Connected(node::Id, Direction),
    Disconnected(node::Id, Report),
    Message(node::Id, message::Receive),
    Upgraded(node::Id),
    UpgradeFailed(node::Id, Report),
}

enum Internal<Conn> {
    Accept,
    Upgrade(transport::Direction<Conn>),
}

enum Input<Conn> {
    Accepted(node::Id, Conn),
    Command(Command),
    Connected(node::Id, Conn),
    Receive(node::Id, message::Receive),
    Upgraded(node::Id, Result<peer::Peer<peer::Running<Conn>>>),
}

enum Output<Conn> {
    Event(Event),
    Internal(Internal<Conn>),
}

impl<Conn> From<Event> for Output<Conn> {
    fn from(event: Event) -> Self {
        Output::Event(event)
    }
}

impl<Conn> From<Internal<Conn>> for Output<Conn> {
    fn from(internal: Internal<Conn>) -> Self {
        Output::Internal(internal)
    }
}

pub struct Supervisor {
    command: Sender<Command>,
    events: Receiver<Event>,
}

impl Supervisor {
    pub fn run<T>(transport: T) -> Result<Self>
    where
        T: transport::Transport + Send + 'static,
    {
        let (command, commands) = unbounded();
        let (event_tx, events) = unbounded();
        let supervisor = Self { command, events };

        let (endpoint, mut incoming) = transport.bind(transport::BindInfo {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 12345),
            advertise_addrs: vec![SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                12345,
            )],
            public_key: PublicKey::from_raw_ed25519(&[
                215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225,
                114, 243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
            ])
            .unwrap(),
        })?;

        // ACCEPT
        let (accept_tx, accept_rx) = unbounded::<()>();
        let (accepted_tx, accepted_rx) = unbounded();
        thread::spawn(move || loop {
            accept_rx.recv().unwrap();

            let conn = incoming.next().unwrap().unwrap();
            let id = match conn.public_key() {
                PublicKey::Ed25519(ed25519) => node::Id::from(ed25519),
                _ => panic!(),
            };

            accepted_tx.send(Input::Accepted(id, conn)).unwrap();
        });

        // CONNECT
        let (_connect_tx, connect_rx) = unbounded::<transport::ConnectInfo>();
        let (connected_tx, connected_rx) = unbounded();
        thread::spawn(move || loop {
            let info = connect_rx.recv().unwrap();
            let conn = endpoint.connect(info).unwrap();
            let id = match conn.public_key() {
                PublicKey::Ed25519(ed25519) => node::Id::from(ed25519),
                _ => panic!(),
            };

            connected_tx.send(Input::Connected(id, conn)).unwrap();
        });

        // UPGRADE
        let (upgrade_tx, upgrade_rx) =
            unbounded::<transport::Direction<<T as transport::Transport>::Connection>>();
        let (upgraded_tx, upgraded_rx) = unbounded();
        thread::spawn(move || loop {
            let conn = upgrade_rx.recv().unwrap();
            let peer = peer::Peer::try_from(conn).unwrap();

            upgraded_tx
                .send(Input::Upgraded(peer.id, peer.run(vec![])))
                .unwrap();
        });

        // MAIN
        thread::spawn(move || {
            let mut state: State<<T as transport::Transport>::Connection> = State {
                connected: HashMap::new(),
                stopped: HashSet::new(),
                upgraded: HashMap::new(),
            };

            loop {
                let input = {
                    let mut selector = flume::Selector::new()
                        .recv(&accepted_rx, |accepted| accepted.unwrap())
                        .recv(&commands, |res| Input::Command(res.unwrap()))
                        .recv(&connected_rx, |connected| connected.unwrap())
                        .recv(&upgraded_rx, |upgrade| upgrade.unwrap());

                    for (id, peer) in &state.upgraded {
                        selector = selector.recv(&peer.state.receiver, move |res| {
                            Input::Receive(*id, res.unwrap())
                        });
                    }

                    selector.wait()
                };

                for output in state.transition(input) {
                    match output {
                        Output::Event(event) => event_tx.send(event).unwrap(),
                        Output::Internal(internal) => match internal {
                            Internal::Accept => accept_tx.send(()).unwrap(),
                            Internal::Upgrade(conn) => upgrade_tx.send(conn).unwrap(),
                        },
                    }
                }
            }
        });

        Ok(supervisor)
    }

    pub fn recv(&self) -> Result<Event> {
        match self.events.recv() {
            Ok(msg) => Ok(msg),
            Err(err) => Err(eyre!("sender disconnected: {}", err)),
        }
    }

    pub fn command(&self, cmd: Command) -> Result<()> {
        self.command.send(cmd).wrap_err("command send failed")
    }
}

struct State<Conn>
where
    Conn: Connection,
{
    connected: HashMap<node::Id, Direction>,
    stopped: HashSet<node::Id>,
    upgraded: HashMap<node::Id, peer::Peer<peer::Running<Conn>>>,
}

impl<Conn> State<Conn>
where
    Conn: Connection,
{
    fn transition(&mut self, input: Input<Conn>) -> Vec<Output<Conn>> {
        match input {
            Input::Accepted(id, conn) => self.handle_accepted(id, conn),
            Input::Command(command) => self.handle_command(command),
            Input::Connected(id, conn) => self.handle_connected(id, conn),
            Input::Receive(id, msg) => self.handle_receive(id, msg),
            Input::Upgraded(id, res) => self.handle_upgraded(id, res),
        }
    }

    fn handle_accepted(&mut self, id: node::Id, conn: Conn) -> Vec<Output<Conn>> {
        // TODO(xla): Ensure we only allow one connection per node. Unless a higher-level protocol
        // like PEX is taking care of it.
        self.connected.insert(id, Direction::Incoming);

        vec![
            Output::from(Event::Connected(id, Direction::Incoming)),
            Output::from(Internal::Upgrade(transport::Direction::Incoming(conn))),
        ]
    }

    fn handle_command(&mut self, command: Command) -> Vec<Output<Conn>> {
        match command {
            Command::Accept => vec![Output::from(Internal::Accept)],
            Command::Connect(_addr) => vec![],
            Command::Disconnect(id) => {
                let peer = self.upgraded.remove(&id).unwrap();
                // FIXME(xla): This side-effect handling does not belong here it should be handled
                // outside of the state machine.
                let stopped = peer.stop().unwrap();
                self.stopped.insert(id);

                vec![Output::from(Event::Disconnected(
                    id,
                    Report::msg("successfully disconected"),
                ))]
            }
            Command::Msg(peer_id, msg) => {
                let peer = self.upgraded.get(&peer_id).unwrap();

                peer.send(msg).unwrap();

                vec![]
            }
        }
    }

    fn handle_connected(&mut self, id: node::Id, conn: Conn) -> Vec<Output<Conn>> {
        // TODO(xla): Ensure we only allow one connection per node. Unless a higher-level protocol
        // like PEX is taking care of it.
        self.connected.insert(id, Direction::Outgoing);

        vec![
            Output::from(Event::Connected(id, Direction::Outgoing)),
            Output::from(Internal::Upgrade(transport::Direction::Outgoing(conn))),
        ]
    }

    fn handle_receive(&self, id: node::Id, msg: message::Receive) -> Vec<Output<Conn>> {
        vec![Output::from(Event::Message(id, msg))]
    }

    fn handle_upgraded(
        &mut self,
        id: node::Id,
        res: Result<peer::Peer<peer::Running<Conn>>>,
    ) -> Vec<Output<Conn>> {
        match res {
            Ok(peer) => {
                self.upgraded.insert(id, peer);

                vec![Output::from(Event::Upgraded(id))]
            }
            Err(err) => {
                self.connected.remove(&id);

                vec![Output::from(Event::UpgradeFailed(id, err))]
            }
        }
    }
}
