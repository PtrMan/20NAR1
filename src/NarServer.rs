use async_std::{
    prelude::*,
    io::BufReader,
    net::TcpStream,
    net::TcpListener,
    task,
};
use std::net::ToSocketAddrs;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;

use crate::Nar::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn acceptLoop(addr: impl ToSocketAddrs + async_std::net::ToSocketAddrs, tx: Sender<String>) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        let _handle = task::spawn(connectionLoop(stream, tx.clone()));
    }

    Ok(())
}

async fn connectionLoop(stream: TcpStream, tx: Sender<String>) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next().await {
        let line = line?;
        println!("recv:{}", line);
        tx.send(line).unwrap(); // send line into NAR
    }
    Ok(())
}

pub fn run() {
    let (tx, rx) = channel();

    let future = acceptLoop("127.0.0.1:2039", tx.clone());
    
    // worker thread which runs NAR
    thread::spawn(move || {
        let mut nar = createNar();

        while true {
            let received:String = rx.recv().unwrap();
            inputN(&mut nar, &received);
        }
    });
    
    task::block_on(future);
}
