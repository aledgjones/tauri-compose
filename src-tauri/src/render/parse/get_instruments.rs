use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::layouts::Layout;
use crate::AppContext;

impl AppContext {
    pub fn get_instruments(&self, layout: &Layout, flow: &Flow) -> Vec<&Instrument> {
        let mut instruments: Vec<&Instrument> = Vec::new();

        for player_id in &self.score.players.order {
            if flow.player_ids.contains(player_id) && layout.player_ids.contains(player_id) {
                let player = &self.score.players.by_id[player_id];
                for instrument_id in &player.instrument_ids {
                    let instrument = &self.score.instruments.by_id[instrument_id];
                    instruments.push(instrument);
                }
            }
        }

        instruments
    }
}
