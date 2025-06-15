use std::collections::HashMap;

pub mod core_engine;
#[cfg(test)]
mod tests {
    use super::*;
    use PokerAction::*;

    #[test]
    fn should_not_be_able_to_play_with_less_than_2_players() {
        assert!(GameState::init(0).is_none());
        assert!(GameState::init(1).is_none());
    }

    #[test]
    fn should_fail_on_illegal_moves() {
        let mut gs = GameState::init(2).unwrap();

        assert!(gs.play_action(CallOrCheck).is_err());
        assert!(gs.play_action(Fold).is_err());

        gs.play_action(StartRound).unwrap();
        assert!(gs.play_action(StartRound).is_err());
    }

    #[test]
    fn should_start_and_deduct_blind() {
        let mut sut = TwoPlayerGameTestContainer::init();

        sut.then_score_is(100, 100);

        sut.when_start_round();
        sut.then_score_is(99, 98);
    }

    #[test]
    fn should_win_blind_when_other_player_folds() {
        let mut sut = TwoPlayerGameTestContainer::init();
        sut.when_start_round();

        sut.when_player_plays(0, Fold);
        sut.then_score_is(99, 101);

        sut.when_start_round();
        sut.then_score_is(97, 100);

        sut.when_player_plays(1, Fold);
        sut.then_score_is(100, 100);
    }

    struct TwoPlayerGameTestContainer {
        gs: GameState,
        actual_next_player: usize,
    }

    impl TwoPlayerGameTestContainer {
        fn init() -> TwoPlayerGameTestContainer {
            TwoPlayerGameTestContainer {
                gs: GameState::init(2).unwrap(),
                actual_next_player: 99,
            }
        }

        fn then_score_is(&self, p0_chips: u32, p1_chips: u32) {
            self.then_player_has_chips(0, p0_chips);
            self.then_player_has_chips(1, p1_chips);
        }

        fn then_player_has_chips(&self, player: usize, expected_chips: u32) {
            assert_eq!(self.gs.current_chips(player), expected_chips);
        }

        fn when_start_round(&mut self) -> () {
            self.actual_next_player = self.gs.play_action(PokerAction::StartRound).unwrap();
        }

        fn when_player_plays(&mut self, player: usize, action: PokerAction) -> () {
            assert_eq!(player, self.actual_next_player);
            self.actual_next_player = self.gs.play_action(action).unwrap();
        }
    }
}

struct GameState {
    player_chips: Vec<u32>,
    big_blind: usize,
    next_player: usize,
    is_round_started: bool,
}

impl GameState {
    fn init(players: usize) -> Option<Self> {
        if players > 1 {
            Some(GameState {
                player_chips: vec![100, 100],
                big_blind: 0,
                next_player: 0,
                is_round_started: false,
            })
        } else {
            None
        }
    }

    fn current_chips(&self, player: usize) -> u32 {
        self.player_chips[player]
    }

    fn play_action(&mut self, action: PokerAction) -> Result<usize, ()> {
        match action {
            PokerAction::CallOrCheck => {
                return Err(());
            }
            PokerAction::Fold => {
                if !self.is_round_started {
                    return Err(());
                }
                self.player_chips[self.big_blind] += 3;
                self.is_round_started = false;
            }
            PokerAction::StartRound => {
                if self.is_round_started {
                    return Err(());
                }
                self.is_round_started = true;
                self.big_blind = self.next_big_blind();
                let small = self.small_blind();
                self.next_player = small;
                self.player_chips[self.big_blind] -= 2;
                self.player_chips[small] -= 1;
            }
        }

        Ok(self.next_player)
    }

    fn small_blind(&self) -> usize {
        (self.big_blind + 1) % 2
    }

    fn next_big_blind(&self) -> usize {
        (self.big_blind + 1) % 2
    }
}

pub enum PokerAction {
    CallOrCheck,
    Fold,
    StartRound,
}
