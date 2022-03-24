use assert_fs::TempDir;
use chain_impl_mockchain::chaintypes::ConsensusVersion;
use quibitous_automation::quibitous::{
    get_available_port,
    grpc::{client::QuibitousWatchClient, QuibitousClient},
    ConfigurationBuilder, QuibitousParams, QuibitousProcess, Starter,
};
use quibitous_automation::testing::SyncNode;
use quibitous_lib::interfaces::TrustedPeer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
const DEFAULT_SLOT_DURATION: u8 = 2;
const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub struct Config {
    addr: SocketAddr,
}

impl Config {
    pub fn attach_to_local_node(port: u16) -> Self {
        Self {
            addr: SocketAddr::new(LOCALHOST, port),
        }
    }

    pub fn client(&self) -> QuibitousClient {
        QuibitousClient::new(self.addr)
    }

    pub fn watch_client(&self) -> QuibitousWatchClient {
        QuibitousWatchClient::new(self.addr)
    }
}

pub mod client {
    use quibitous_automation::quibitous::grpc::client::QuibitousWatchClient;

    use super::*;
    pub struct ClientBootstrap {
        pub client: QuibitousClient,
        pub watch_client: QuibitousWatchClient,
        pub server: QuibitousProcess,
        pub config: QuibitousParams,
    }

    pub fn default() -> ClientBootstrap {
        bootstrap(
            ConfigurationBuilder::new()
                .with_slot_duration(DEFAULT_SLOT_DURATION)
                .to_owned(),
        )
    }

    pub fn bootstrap(config: ConfigurationBuilder) -> ClientBootstrap {
        let dir = TempDir::new().unwrap();
        let config = config.build(&dir);
        let server = Starter::new()
            .temp_dir(dir)
            .config(config.clone())
            .start()
            .unwrap();
        let attached_config = Config::attach_to_local_node(config.get_p2p_listen_port());
        let client = attached_config.client();
        let watch_client = attached_config.watch_client();
        ClientBootstrap {
            client,
            server,
            config,
            watch_client,
        }
    }
}

pub mod server {
    use super::*;
    const SERVER_RETRY_WAIT: Duration = Duration::from_secs(1);
    const TIMEOUT: Duration = Duration::from_secs(60);

    pub struct ServerBootstrap {
        pub server: QuibitousProcess,
        pub config: QuibitousParams,
        pub mock_port: u16,
    }

    pub fn default() -> ServerBootstrap {
        bootstrap(
            get_available_port(),
            ConfigurationBuilder::new()
                .with_slot_duration(DEFAULT_SLOT_DURATION)
                .with_block0_consensus(ConsensusVersion::GenesisOptimum)
                .to_owned(),
        )
    }

    pub fn bootstrap(mock_port: u16, mut config: ConfigurationBuilder) -> ServerBootstrap {
        let dir = TempDir::new().unwrap();
        let trusted_peer = TrustedPeer {
            address: format!("/ip4/{}/tcp/{}", LOCALHOST, mock_port)
                .parse()
                .unwrap(),
            id: None,
        };
        let config = config.with_trusted_peers(vec![trusted_peer]).build(&dir);
        let server = Starter::new()
            .temp_dir(dir)
            .config(config.clone())
            .start_async()
            .unwrap();
        ServerBootstrap {
            server,
            config,
            mock_port,
        }
    }

    impl ServerBootstrap {
        pub fn wait_server_online(&self) {
            let started = std::time::Instant::now();
            loop {
                if self.server.is_running() {
                    return;
                }
                if started.elapsed() > TIMEOUT {
                    println!("{}", self.server.log_content());
                    panic!("Timeout elapsed while waiting for server to go online");
                }
                std::thread::sleep(SERVER_RETRY_WAIT);
            }
        }
    }
}
