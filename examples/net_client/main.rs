// CLIENT
use std::time::Duration;

use amethyst::{
    core::{
        ecs::{System, SystemBundle},
        frame_limiter::FrameRateLimitStrategy,
        Time,
    },
    network::simulation::{
        tcp::TcpNetworkBundle, NetworkSimulationEvent, NetworkSimulationTime, TransportResource,
    },
    prelude::*,
    shrev::{EventChannel, ReaderId},
    utils::application_root_dir,
    Result,
};
use log::{error, info};

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("examples/net_client/");

    //    // UDP
    //    let socket = UdpSocket::bind("0.0.0.0:3455")?;
    //    socket.set_nonblocking(true)?;

    //    // TCP: No listener needed for the client.

    //    // Laminar
    //    let socket = LaminarSocket::bind("0.0.0.0:3455")?;

    let mut game_data = DispatcherBuilder::default();
    //        // UDP
    //        .add_bundle(UdpNetworkBundle::new(Some(socket), 2048))?
    // TCP
    game_data
        .add_bundle(TcpNetworkBundle::new(None, 2048))
        //        // Laminar
        //        .add_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .add_bundle(SpamBundle);

    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

/// Default empty state
pub struct GameState;
impl SimpleState for GameState {}

#[derive(Debug)]
struct SpamBundle;

impl SystemBundle for SpamBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<()> {
        let mut chan = EventChannel::<NetworkSimulationEvent>::default();
        let reader = chan.register_reader();
        resources.insert(chan);

        resources.insert(TransportResource::default());
        resources.insert(NetworkSimulationTime::default());

        builder.add_system(Box::new(SpamSystem { reader }));

        Ok(())
    }
}

/// A simple system that receives a ton of network events.
#[derive(Debug)]
struct SpamSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl System<'static> for SpamSystem {
    fn build(&'static mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TransformSystem")
                .read_resource::<NetworkSimulationTime>()
                .read_resource::<Time>()
                .read_resource::<EventChannel<NetworkSimulationEvent>>()
                .write_resource::<TransportResource>()
                .build(
                    move |_commands, world, (sim_time, time, event, net), query| {
                        let server_addr = "127.0.0.1:3457".parse().unwrap();
                        for frame in sim_time.sim_frames_to_run() {
                            info!("Sending message for sim frame {}.", frame);
                            let payload = format!(
                                "CL: sim_frame:{},abs_time:{}",
                                frame,
                                time.absolute_time_seconds()
                            );
                            net.send(server_addr, payload.as_bytes());
                        }

                        for event in event.read(&mut self.reader) {
                            match event {
                                NetworkSimulationEvent::Message(_addr, payload) => {
                                    info!("Payload: {:?}", payload)
                                }
                                NetworkSimulationEvent::Connect(addr) => {
                                    info!("New client connection: {}", addr)
                                }
                                NetworkSimulationEvent::Disconnect(addr) => {
                                    info!("Server Disconnected: {}", addr)
                                }
                                NetworkSimulationEvent::RecvError(e) => {
                                    error!("Recv Error: {:?}", e);
                                }
                                NetworkSimulationEvent::SendError(e, msg) => {
                                    error!("Send Error: {:?}, {:?}", e, msg);
                                }
                                _ => {}
                            }
                        }
                    },
                ),
        )
    }
}
