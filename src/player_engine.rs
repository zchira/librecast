use std::io::prelude::*;
use url2audio::Player;

pub struct PlayerEngine {
    pub stream_addr: Option<String>,
    pub player: Player,
    playing: bool
}

impl Default for PlayerEngine {
    fn default() -> Self {
        PlayerEngine::new()
    }
}

impl PlayerEngine {
    pub fn new() -> Self {
        let player = Player::new();
        PlayerEngine {
            stream_addr: None,
            player,
            playing: false
        }
    }

    pub fn open(&mut self, stream_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.player.open(stream_addr);
        self.stream_addr = Some(stream_addr.to_string());
        self.player.play();
        self.playing = true;
        Ok(())
    }

    fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(addr) = self.stream_addr.as_ref() {
            self.player.open(addr);
            self.player.play();
            self.playing = true;
        }
        Ok(())
    }
    
    pub fn current_position(&self) -> f64 {
        self.player.current_position()
    }

    pub fn current_position_display(&self) -> String {
        self.player.current_position_display()
    }

    pub fn duration(&self) -> f64 {
        self.player.duration()
    }

    pub fn is_paused(&self) -> bool {
        !self.playing
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.player.pause()
    }

    pub fn resume(&self) {
        self.player.play()
    }

    /// seek 30 seconds forward
    pub fn seek_forward(&self) {
        self.player.seek_relative(30.0);
    }

    /// seek 10 seconds backward
    pub fn seek_backward(&self) {
        self.player.seek_relative(-10.0);
    }

    pub fn get_volume(&self) -> f32 {
        // self.sink.volume()
        1.0
    }

    pub fn increase_volume(&mut self) {
        // if self.get_volume() < 1.0 {
        //     self.sink.set_volume(self.get_volume() + 0.1);
        // }
    }

    pub fn decrease_volume(&mut self) {
        // if self.get_volume() > 0.0 {
        //     let mut new_val = self.get_volume() - 0.1;
        //     new_val = if new_val < 0.0 { 0.0 } else { new_val };
        //     self.sink.set_volume(new_val);
        // }
    }

}

// struct FifoRead {
//     pub reader: Box<dyn Read + Send + Sync + 'static>
// }
//
// impl FifoRead {
//     fn from_str(addr: &str) -> Result<Self, ureq::Error> {
//         let r = ureq::get(addr).call();
//         match r {
//             Ok(r) => {
//                 Ok(
//                     FifoRead {
//                         reader: r.into_reader()
//                     }
//                   )
//             },
//             Err(e) => Err(e),
//         }
//     }
//
//     // fn new() -> Self {
//     //     FifoRead {
//     //         reader: ureq::get("https://stream.daskoimladja.com:9000/stream").call().unwrap().into_reader()
//     //     }
//     // }
// }
//
// impl Read for FifoRead {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//        self.reader.read(buf)
//     }
// }
//
// impl Seek for FifoRead {
//     fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
//         match pos {
//             std::io::SeekFrom::Start(i) => {},
//             std::io::SeekFrom::End(_) => todo!(),
//             std::io::SeekFrom::Current(_) => todo!(),
//         }
//         Ok(0)
//     }
// }
