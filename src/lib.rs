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
    fn should_win_blind_when_other_player_folds() {
        let mut sut = TwoPlayerGameTestContainer::init();

        sut.then_player_has_chips(0, 99);
        sut.then_player_has_chips(1, 98);

        sut.when_player_plays(0, PokerAction::Fold);

        sut.then_player_has_chips(0, 99);
        sut.then_player_has_chips(1, 101);
    }

    struct TwoPlayerGameTestContainer {
        gs: GameState,
    }

    impl TwoPlayerGameTestContainer {
        fn init() -> TwoPlayerGameTestContainer {
            TwoPlayerGameTestContainer {
                gs: GameState::init(2).unwrap(),
            }
        }

        fn then_player_has_chips(&self, player: usize, expected_chips: u32) {
            assert_eq!(self.gs.current_chips(player), expected_chips);
        }

        fn when_player_plays(&mut self, player: u32, action: PokerAction) -> () {
            self.gs = std::mem::replace(&mut self.gs, Self::empty_game_state()).play_action(action);
        }

        fn empty_game_state() -> GameState {
            GameState {
                player_chips: vec![],
            }
        }
    }
}

struct GameState {
    player_chips: Vec<u32>,
}

impl GameState {
    fn init(players: usize) -> Option<Self> {
        if players > 1 {
            Some(GameState {
                player_chips: vec![99, 98],
            })
        } else {
            None
        }
    }

    fn current_chips(&self, player: usize) -> u32 {
        self.player_chips[player]
    }

    fn play_action(self, action: PokerAction) -> Self {
        let mut player_chips = self.player_chips;
        player_chips[1] = 101;
        GameState { player_chips }
    }
}
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

struct Game {
    player: u32,
    chips: HashMap<u32, u32>,
    pot: u32,
}
impl Game {
    fn new() -> Self {
        Self {
            player: 0,
            chips: HashMap::new(),
            pot: 0,
        }
    }

    fn register_player(self, player: &impl Player) -> (Self, u32) {
        let mut chips = self.chips;
        chips.insert(self.player, 100);
        (
            Self {
                player: self.player + 1,
                chips,
                ..self
            },
            self.player,
        )
    }

    fn play_step(self) -> Self {
        self.put_chips_in_pot(0, 2).put_chips_in_pot(1, 1)
    }

    fn put_chips_in_pot(self, player: u32, amount: u32) -> Self {
        let mut chips = self.chips;
        chips.insert(player, chips.get(&player).unwrap() - amount);
        Self {
            chips,
            pot: self.pot + amount,
            ..self
        }
    }
    fn current_chips(&self, pid: &u32) -> u32 {
        *self.chips.get(pid).unwrap()
    }

    fn play_round(self) -> Self {
        let mut chips = self.chips;
        chips.insert(0, chips.get(&0).unwrap() + self.pot);
        Self {
            chips,
            pot: 0,
            ..self
        }
    }
}

pub enum PokerAction {
    CallOrCheck,
    Fold,
}

pub trait Player {
    fn do_play() -> PokerAction;
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn should_win_blind_when_other_player_folds() {
        let player1 = AlwaysCallingPlayer {};
        let player2 = AlwaysFoldingPlayer {};
        let (game, player1id) = Game::new().register_player(&player1);
        let (game, player2id) = game.register_player(&player2);

        assert_ne!(player1id, player2id);
        assert_eq!(game.current_chips(&player1id), 100);
        assert_eq!(game.current_chips(&player2id), 100);

        let game = game.play_step();

        assert_eq!(game.current_chips(&player1id), 98);
        assert_eq!(game.current_chips(&player2id), 99);

        let game = game.play_round();

        assert_eq!(game.current_chips(&player1id), 101);
        assert_eq!(game.current_chips(&player2id), 99);
    }

    #[test]
    fn should_not_play_if_less_than_two_players() {}

    use PokerAction::*;

    struct AlwaysCallingPlayer {}

    impl Player for AlwaysCallingPlayer {
        fn do_play() -> PokerAction {
            CallOrCheck
        }
    }

    struct AlwaysFoldingPlayer {}

    impl Player for AlwaysFoldingPlayer {
        fn do_play() -> PokerAction {
            Fold
        }
    }
}
