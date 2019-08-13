use env_logger;
use grin_core::core::hash::Hashed;
use grin_core::pow::Difficulty;
use grin_p2p::handshake::Handshake;
use std::net::{SocketAddr, TcpStream, Shutdown};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use grin_core::genesis::{genesis_floo, genesis_main};
use grin_core::global::{set_mining_mode, ChainTypes};
use grin_p2p::types::{Capabilities, P2PConfig, PeerAddr};
use grin_p2p::Peer;
use log::*;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Error, Write};

use chrono::prelude::*;
extern crate chrono;

mod adapter;
use crate::adapter::FakeAdapter;
use crossbeam_queue::SegQueue;
use crossbeam_utils::sync::ShardedLock;
use structopt::StructOpt;

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::*;

type Storage = HashMap<PeerAddr, Option<Vec<PeerAddr>>>;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Args {
    #[structopt(short, long)]
    floonet: bool,
}

const NTHREADS: u8 = 100;

// mainnet nodes
// 5.9.152.75 mainnet.seed.grin-tech.org / mainnet.seed.grin.lesceller.com
// 139.162.168.18  mainnet.seed.grin.icu
// 94.130.229.193 mainnet.seed.713.mw
// 159.69.37.136 mainnet.seed.grin.prokapi.com
// 109.74.202.16  grinseed.yeastplume.org

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("grin-map.log").unwrap()),
        ]
    ).unwrap();


	info!("Hello, world!");

    let args = Args::from_args();

    let mut cfg = P2PConfig::default();
    cfg.host = "0.0.0.0".parse().unwrap();
    cfg.port = 13415;

    let mut peer_addr: PeerAddr;
    let mut handshake: Arc<Handshake>;

    let seeds = ["5.9.152.75", 
                 "139.162.168.18",
                 "94.130.229.193",
                 "159.69.37.136",
                 "109.74.202.16"
    ];
    set_mining_mode(ChainTypes::Mainnet);
    //peer_addr = PeerAddr(SocketAddr::new("109.74.202.16".parse().unwrap(), 3414)); // mainnet

    //peer_addr = PeerAddr(SocketAddr::new("127.0.0.1".parse().unwrap(), 3414)); // mainnet
    handshake = Arc::new(Handshake::new(genesis_main().hash(), cfg.clone()));
    

    let queue = Arc::new(SegQueue::new());
    let hm: HashMap<PeerAddr, Option<Vec<PeerAddr>>> = HashMap::new();
    let storage = Arc::new(ShardedLock::new(hm));

    let local_addr = PeerAddr(SocketAddr::new(cfg.host, cfg.port));

    let mut count = 1;

    for ip in &seeds{
        peer_addr = PeerAddr(SocketAddr::new(ip.parse().unwrap(), 3414)); // mainnet
        queue.push(peer_addr);
    }
    //   let mut threads = vec![];

    let t = worker(
        handshake.clone(),
        local_addr.clone(),
        queue.clone(),
        storage.clone(),
        count,
    );
    //    threads.push(t);

    loop {
        if Arc::strong_count(&queue) == 1 {
            break;
        }
        if queue.len() > 5 {
            info!(
                "Too many items in queue. Start new thread (was {})",
                Arc::strong_count(&queue)
            );
            if Arc::strong_count(&queue) < 250 {
                count += 1;
                worker(
                    handshake.clone(),
                    local_addr.clone(),
                    queue.clone(),
                    storage.clone(),
                    count,
                );
            } else {
                info!("reached thread limits");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }

    let mut public = 0;
    for (k, v) in &(*storage.read().unwrap()) {
        println!("\x1B[35;1m v\x1B[0m = {:?}", v);
        if v.is_some() {
            public += 1;
        }
    }
    println!(
        "Result: total {}, public: {}",
        storage.read().unwrap().len(),
        public
    );
    store(&*storage.read().unwrap()).unwrap();
}

fn worker(
    handshake: Arc<Handshake>,
    local_addr: PeerAddr,
    queue: Arc<SegQueue<PeerAddr>>,
    storage: Arc<ShardedLock<Storage>>,
    i: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        info!("thread {}: start", i);
        loop {
            info!("thread {}: New iterations", i);
            let peer_addr = match queue.pop() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("thread {}: Queue is emtpy. Quit .", i);
                    break;
                }
            };
            if storage.write().unwrap().contains_key(&peer_addr) {
                continue;
            }
            info!("Thread {}: got add {}", i, peer_addr);
            match connect(peer_addr, local_addr, &handshake) {
                Ok(addrs) => {
                    for a in &addrs {
                        queue.push(a.clone());
                    }
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, Some(addrs));
                }
                Err(e) => {
                    debug!("Cannot connect: {}", e);
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, None);
                }
            }
            info!("thread {}: end of loop", i);
        }
    })
}

fn connect(
    peer_addr: PeerAddr,
    local_addr: PeerAddr,
    handshake: &Handshake,
) -> Result<Vec<PeerAddr>, String> {
    let stream = TcpStream::connect_timeout(&peer_addr.0, Duration::from_secs(10))
        .map_err(|e| format!("{}", e))?;

    let adapter = Arc::new(FakeAdapter::new());
    //let adapter = Arc::new(p2p::DummyAdapter {});

    let peer = Peer::connect(
        stream,
        Capabilities::PEER_LIST,
        Difficulty::min(),
        local_addr,
        handshake,
        adapter.clone(),
    )
    .map_err(|e| format!("{:?}", e))?;

    info!("{} User_agent: {}; Version: {}",  &peer_addr, &peer.info.user_agent, &peer.info.version);

    let peer = Arc::new(peer);

    peer.send_peer_request(Capabilities::PEER_LIST)
        .map_err(|e| format!("{:?}", e))?;

    let r = adapter.get_peers().map_err(|e| format!("{:?}", e));
    thread::sleep(Duration::from_secs(2));
    peer.stop();
    r
}

fn store(hm: &Storage) -> Result<(), Error> {
    let now = Local::now();
    let fn_ = format!("{}.csv", now.format("%F-%H"));
    let mut f = File::create(fn_)?;
    for (k, v) in hm {
        write!(f, "{},[", k)?;
        if v.is_some() {
            let peers = v
                .as_ref()
                .unwrap()
                .into_iter()
                .map(|addr| addr.as_key())
                .collect::<Vec<String>>()
                .join(",");
            write!(f, "{}", peers)?;
        }
        write!(f, "]\n")?;
    }
    Ok(())
}
