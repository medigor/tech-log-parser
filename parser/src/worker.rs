use std::{
    error::Error,
    fs::File,
    io::{Read, Seek},
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    thread::JoinHandle,
};

pub struct FileReadWorker {
    worker: Option<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>>,
    sender: Sender<Option<Vec<u8>>>,
    receiver: Receiver<(usize, Vec<u8>)>,
}

impl FileReadWorker {
    pub fn new<P: AsRef<Path>>(file_name: P) -> Result<Self, Box<dyn Error>> {
        let (parser_sender, thread_receiver) = mpsc::channel::<Option<Vec<u8>>>();
        let (thread_sender, parser_receiver) = mpsc::channel::<(usize, Vec<u8>)>();

        for _ in 0..3 {
            let mut buf = Vec::<u8>::with_capacity(1024 * 1024);
            buf.extend((0..buf.capacity()).map(|_| 0));
            parser_sender.send(Some(buf))?;
        }

        let file_name = file_name.as_ref().to_owned();

        let worker = std::thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut file = File::open(file_name)?;
                file.seek(std::io::SeekFrom::Start(3))?;
                loop {
                    let Some(mut buf) = thread_receiver.recv()? else {
                        return Ok(());
                    };
                    let offset = buf.len() / 2;
                    let size = file.read(&mut buf[offset..])?;
                    thread_sender.send((size, buf))?;
                }
            },
        );

        Ok(Self {
            worker: Some(worker),
            sender: parser_sender,
            receiver: parser_receiver,
        })
    }

    #[inline(always)]
    pub fn send(&self, buf: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.sender.send(Some(buf))?;
        Ok(())
    }

    #[inline(always)]
    pub fn recv(&self) -> Result<(usize, Vec<u8>), Box<dyn Error>> {
        Ok(self.receiver.recv()?)
    }

    #[inline(always)]
    pub fn is_finished(&mut self) -> Result<bool, Box<dyn Error>> {
        let Some(worker) = &self.worker else {
            return Ok(true);
        };

        if !worker.is_finished() {
            return Ok(false);
        }

        let Some(worker) = self.worker.take() else {
            return Ok(true);
        };

        match worker.join().expect("thread paniced") {
            Ok(_) => (),
            Err(err) => return Err(err.to_string().into()),
        }
        Ok(true)
    }
}

impl Drop for FileReadWorker {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let _ = self.sender.send(None);
        }
    }
}
