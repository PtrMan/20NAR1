use async_std::{
    prelude::*,
    io::BufReader,
    io::BufWriter,
    net::TcpStream,
    net::TcpListener,
    task,
};
use std::net::ToSocketAddrs;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::rc::Rc;
use std::cell::{RefCell};

use crate::Term::{Term, convTermToStr};
use crate::Nar::*;
use crate::NarInputFacade;
use crate::NarWorkingCycle::QHandler;
use crate::NarSentence::SentenceDummy;
use crate::NarSentence::convSentenceTermPunctToStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn acceptLoop(addr: impl ToSocketAddrs + async_std::net::ToSocketAddrs, tx: Sender<String>, global:Arc<Mutex<Global>>) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        let _handle = task::spawn(connectionLoop(stream, tx.clone(), global.clone()));
    }

    Ok(())
}

async fn connectionLoop(stream: TcpStream, tx: Sender<String>, global:Arc<Mutex<Global>>) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut lines = reader.lines();

    let mut myIdCounter; // id counter used to track broadcasted messages
    {
        let mut data = global.lock().unwrap(); // unwrap to panic if it can't unlock
        myIdCounter = data.idCounter; // sync up
    }

    while let Some(line) = lines.next().await {
        let line = line?;
        println!("recv:{}", line);
        tx.send(line).unwrap(); // send line into NAR

        // send broadcasted to stream
        let mut toSend:Vec<String> = Vec::new();
        {
            let mut data = global.lock().unwrap(); // unwrap to panic if it can't unlock
            for iItem in &data.arr { // iterate over global items and send to writer
                if iItem.0 >= myIdCounter { // is it a message which this tread didn't yet observe?
                    myIdCounter = iItem.0; // update counter so the message is next time ignored
                    let bcastedMsg:String = iItem.1.clone(); // copy broadcasted string to local variable
                    toSend.push(bcastedMsg);
                }
            }
        }
        
        for iLine in &toSend {
            // send broadcasted to stream
            writer.write((iLine.to_owned()+"\n").as_bytes()).await;
            writer.flush().await;
        }

        // send broadcasted to stream
        /*
        loop {
            let recv:Option<String> = bcast.clone().next().await;

            if !recv.is_some() {
               break;
            }
            writer.write((recv.unwrap() + "\n").as_bytes()).await;
            writer.flush().await;
        }*/

        // 23.10.2020 : this is working fine, is commented because it is a test
        //println!("HERE");
        //
        //writer.write(("//TEST\n").as_bytes()).await;
        //writer.flush().await;

    }
    Ok(())
}

pub struct Global {
    pub arr:Vec<(i64, String)>,
    pub idCounter:i64,
}

pub fn run() {
    let (tx, rx) = channel();
    let global:Arc<Mutex<Global>> = Arc::new(Mutex::new(Global{arr:Vec::new(),idCounter:0}));

    let future = acceptLoop("127.0.0.1:2039", tx.clone(), global.clone());
    
    // worker thread which runs NAR
    thread::spawn(move || {
        let mut nar = createNar();
        nar.mem.globalQaHandlers.push(Rc::new(RefCell::new(QHandlerImpl{global:Arc::clone(&global)}))); // register Q&A handler to send answers to all clients

        loop {
            let received:String = rx.recv().unwrap();
            let mut quit = false;
            let resLines: Vec<String> = NarInputFacade::input(&mut nar, &received, &mut quit);
            for iLine in &resLines {
                let mut data = global.lock().unwrap(); // unwrap to panic when it can't unlock
                let idCounter = data.idCounter;
                data.arr.push((idCounter, iLine.clone()));
                data.idCounter+=1;

                // limit size of data
                let dataMaxLength = 50;
                if data.arr.len() > dataMaxLength {
                    data.arr = data.arr[data.arr.len()-dataMaxLength..].to_vec();
                }
            }

            for iLine in &resLines {
                println!("{}", iLine);
            }
        }
    });
    
    task::block_on(future);
}

// handler to send answer to clients
pub struct QHandlerImpl {
    pub global:Arc<Mutex<Global>>,
}

impl QHandler for QHandlerImpl {
    fn answer(&mut self, question:&Term, answer:&SentenceDummy) {
        // print question and send answer
        let msg = "TRACE answer: ".to_owned() + &convTermToStr(&question) + "? " + &convSentenceTermPunctToStr(&answer, true);

        let mut data = self.global.lock().unwrap(); // unwrap to panic when it can't unlock
        let idCounter = data.idCounter;
        data.arr.push((idCounter, msg.clone()));
        data.idCounter+=1;
    }
}

