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

    pub fn duration_display(&self) -> String {
        self.player.duration_display()
    }

    pub fn is_paused(&self) -> bool {
        !self.playing
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.player.pause()
    }

    pub fn resume(&mut self) {
        self.playing = true;
        self.player.play()
    }

    pub fn seek(&self, time: f64) {
        self.player.seek(time);
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

    pub fn get_error(&self) -> Option<String> {
        self.player.error()
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
