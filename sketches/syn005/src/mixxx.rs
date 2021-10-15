use lib::prelude::*;

#[derive(Debug, Clone)]
pub struct MixxxDeck {
    pub song: String,
    pub bpm: f32,
    pub active: bool,
}

impl MixxxDeck {
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}

impl Default for MixxxDeck {
    fn default() -> Self {
        Self {
            song: "".to_owned(),
            bpm: 60.0,
            active: false,
        }
    }
}

#[derive(Debug)]
pub struct Mixxx {
    pub decks: Vec<MixxxDeck>,
}

impl Mixxx {
    pub fn osc(&mut self, msg: OscMessage) {
        if let OscMessage::Mixxx(msg) = msg {
            match msg {
                MixxxMessage::Song {
                    deck,
                    artist,
                    title,
                } => { 
                    let song = format!("{} - {}", artist, title);
                    if self.decks[deck].song != song {
                        log::debug!("Deck {} loaded {}", deck, song);
                    }
                    self.decks[deck].song = song;
                },
                MixxxMessage::Bpm { deck, bpm } => self.decks[deck].bpm = bpm,
            }
        }
    }
}

impl Default for Mixxx {
    fn default() -> Self {
        Self {
            decks: vec![MixxxDeck::default(); 4],
        }
    }
}
