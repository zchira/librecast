use std::io::prelude::*;
use rodio::{Decoder, OutputStream, source::Source, OutputStreamHandle, Sink};


pub struct PlayerEngine {
    pub stream_addr: Option<String>,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink
}

impl Default for PlayerEngine {
    fn default() -> Self {
        PlayerEngine::new()
    }
}

impl PlayerEngine {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink  = Sink::try_new(&stream_handle).unwrap();
        PlayerEngine {
            stream_addr: None,
            stream,
            stream_handle,
            sink
        }
    }

    pub fn open(&mut self, stream_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stream_addr = Some(stream_addr.to_string());
        self.play()?;
        Ok(())
    }

    fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(addr) = self.stream_addr.as_ref() {
            let fifo_read = FifoRead::from_str(&addr)?;
            let source = Decoder::new(fifo_read).unwrap();

            // self.sink.play();
            self.sink.append(source);
            self.sink.set_volume(self.get_volume());
            // self.sink.play();
            // let _play = self.stream_handle.play_raw(source.convert_samples())?;
        }
        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn pause(&self) {
        self.sink.pause()
    }

    pub fn resume(&self) {
        self.sink.play()
    }

    pub fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn increase_volume(&mut self) {
        if self.get_volume() < 1.0 {
            self.sink.set_volume(self.get_volume() + 0.1);
        }
    }

    pub fn decrease_volume(&mut self) {
        if self.get_volume() > 0.0 {
            let mut new_val = self.get_volume() - 0.1;
            new_val = if new_val < 0.0 { 0.0 } else { new_val };
            self.sink.set_volume(new_val);
        }
    }

}

struct FifoRead {
    pub reader: Box<dyn Read + Send + Sync + 'static>
}

impl FifoRead {
    fn from_str(addr: &str) -> Result<Self, ureq::Error> {
        let r = ureq::get(addr).call();
        match r {
            Ok(r) => {
                Ok(
                    FifoRead {
                        reader: r.into_reader()
                    }
                  )
            },
            Err(e) => Err(e),
        }
    }

    // fn new() -> Self {
    //     FifoRead {
    //         reader: ureq::get("https://stream.daskoimladja.com:9000/stream").call().unwrap().into_reader()
    //     }
    // }
}

impl Read for FifoRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
       self.reader.read(buf)
    }
}

impl Seek for FifoRead {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}
