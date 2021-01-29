use std::collections::HashMap;
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

enum Input<Conn> {
    Accepted(node::Id, Conn),
    Command(Command),
    Connected(node::Id, Conn),
    Receive(node::Id, message::Receive),
    Upgraded(node::Id, Result<peer::Peer<peer::Running<Conn>>>),
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
            let peer = peer::Peer::from(conn);

            upgraded_tx
                .send(Input::Upgraded(peer.id, peer.run(vec![])))
                .unwrap();
        });

        // MAIN
        thread::spawn(move || {
            let mut state: State<<T as transport::Transport>::Connection> = State {
                connected: HashMap::new(),
                stopped: HashMap::new(),
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

                for event in state.transition(input) {
                    event_tx.send(event).unwrap();
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
    connected: HashMap<node::Id, transport::Direction<Conn>>,
    stopped: HashMap<node::Id, peer::Peer<peer::Stopped>>,
    upgraded: HashMap<node::Id, peer::Peer<peer::Running<Conn>>>,
}

impl<Conn> State<Conn>
where
    Conn: Connection,
{
    fn transition(&mut self, input: Input<Conn>) -> Vec<Event> {
        match input {
            Input::Accepted(id, conn) => self.handle_accepted(id, conn),
            Input::Command(command) => self.handle_command(command),
            Input::Connected(id, conn) => self.handle_connected(id, conn),
            Input::Receive(id, msg) => self.handle_receive(id, msg),
            Input::Upgraded(id, res) => self.handle_upgraded(id, res),
        }
    }

    fn handle_accepted(&mut self, id: node::Id, conn: Conn) -> Vec<Event> {
        self.connected
            .insert(id, transport::Direction::Incoming(conn));

        vec![Event::Connected(id, Direction::Incoming)]
    }

    fn handle_command(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::Accept => vec![],
            Command::Connect(_addr) => vec![],
            Command::Disconnect(id) => {
                let peer = self.upgraded.remove(&id).unwrap();
                let stopped = peer.stop().unwrap();
                self.stopped.insert(id, stopped);

                vec![Event::Disconnected(
                    id,
                    Report::msg("successfully disconected"),
                )]
            }
            Command::Msg(peer_id, msg) => {
                let peer = self.upgraded.get(&peer_id).unwrap();

                peer.send(msg).unwrap();

                vec![]
            }
        }
    }

    fn handle_connected(&mut self, id: node::Id, conn: Conn) -> Vec<Event> {
        self.connected
            .insert(id, transport::Direction::Outgoing(conn));

        vec![Event::Connected(id, Direction::Outgoing)]
    }

    fn handle_receive(&self, id: node::Id, msg: message::Receive) -> Vec<Event> {
        vec![Event::Message(id, msg)]
    }

    fn handle_upgraded(
        &mut self,
        id: node::Id,
        res: Result<peer::Peer<peer::Running<Conn>>>,
    ) -> Vec<Event> {
        match res {
            Ok(peer) => {
                self.upgraded.insert(id, peer);

                vec![Event::Upgraded(id)]
            }
            Err(err) => {
                self.connected.remove(&id);

                vec![Event::UpgradeFailed(id, err)]
            }
        }
    }
}
