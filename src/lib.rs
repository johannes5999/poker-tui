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
        assert!(gs.play_action(Raise(1)).is_err());

        gs.play_action(StartRound).unwrap();
        assert!(gs.play_action(StartRound).is_err());
    }

    #[test]
    fn should_fail_on_raise_out_of_bounds() {
        let mut gs = GameState::init(2).unwrap();
        gs.play_action(StartRound).unwrap();

        assert!(gs.play_action(Raise(0)).is_err());
        assert!(gs.play_action(Raise(100)).is_err());
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

    #[test]
    fn should_win_blind_if_raise_then_fold() {
        let mut sut = TwoPlayerGameTestContainer::init();
        sut.when_start_round();

        sut.when_player_plays(0, CallOrCheck);
        sut.then_score_is(98, 98);

        sut.when_player_plays(1, Fold);
        sut.then_score_is(102, 98);
    }

    #[test]
    fn should_win_more_after_raise_is_called() {
        let mut sut = TwoPlayerGameTestContainer::init();
        sut.when_start_round();

        sut.when_player_plays(0, Raise(9));
        sut.then_score_is(90, 98);

        sut.when_player_plays(1, CallOrCheck);
        sut.then_score_is(90, 90);
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

#[derive(Clone)]
struct PlayerChips {
    stack: u32,
    bet: u32,
}

struct ChipsState {
    player_chips: Vec<PlayerChips>,
    pot: u32,
}

impl ChipsState {
    fn init(players: usize) -> Self {
        Self {
            player_chips: vec![PlayerChips { stack: 100, bet: 0 }; players],
            pot: 0,
        }
    }

    fn bet_chips(&mut self, player: usize, amount: u32) {
        self.player_chips[player].stack -= amount;
        self.player_chips[player].bet += amount;
    }

    fn win_pot(&mut self, player: usize) {
        self.move_chips_to_pot();
        self.gain_chips(player, self.pot);
        self.pot = 0;
    }

    fn gain_chips(&mut self, player: usize, amount: u32) {
        self.player_chips[player].stack += amount
    }

    fn call(&mut self, player: usize) {
        self.bet_chips(player, self.highest_bet() - self.player_chips[player].bet)
    }

    fn move_chips_to_pot(&mut self) {
        for pc in &mut self.player_chips {
            self.pot += pc.bet;
            pc.bet = 0;
        }
    }

    fn highest_bet(&self) -> u32 {
        self.player_chips.iter().map(|pc| pc.bet).max().unwrap()
    }

    fn current_chips(&self, player: usize) -> u32 {
        self.player_chips[player].stack
    }
}

struct TurnState {
    current_player: usize,
    big_blind: usize,
    is_round_started: bool,
    total_players: usize,
}

impl TurnState {
    fn init(players: usize) -> Self {
        Self {
            current_player: 0,
            big_blind: 0,
            is_round_started: false,
            total_players: players,
        }
    }

    fn advance_player(&mut self) {
        self.current_player = (self.current_player + 1) % 2;
    }

    fn start_new_round(&mut self) {
        self.is_round_started = true;
        self.big_blind = self.next_big_blind();
        self.current_player = self.small_blind();
    }
    fn small_blind(&self) -> usize {
        (self.big_blind + 1) % 2
    }

    fn next_big_blind(&self) -> usize {
        (self.big_blind + 1) % 2
    }

    fn is_action_valid(&self, action: &PokerAction) -> bool {
        if self.is_round_started {
            action != &PokerAction::StartRound
        } else {
            action == &PokerAction::StartRound
        }
    }
}

struct GameState {
    chips_state: ChipsState,
    turn_state: TurnState,
    big_blind: usize,
    next_player: usize,
    is_round_started: bool,
}

impl GameState {
    fn init(players: usize) -> Option<Self> {
        if players > 1 {
            Some(GameState {
                chips_state: ChipsState::init(players),
                turn_state: TurnState::init(players),
                big_blind: 0,
                next_player: 0,
                is_round_started: false,
            })
        } else {
            None
        }
    }

    fn current_chips(&self, player: usize) -> u32 {
        self.chips_state.current_chips(player)
    }

    fn play_action(&mut self, action: PokerAction) -> Result<usize, ()> {
        if !self.turn_state.is_action_valid(&action) {
            return Err(());
        }

        match action {
            PokerAction::CallOrCheck => {
                self.chips_state.call(self.turn_state.current_player);
                self.turn_state.advance_player();
            }
            PokerAction::Fold => {
                self.chips_state
                    .win_pot((self.turn_state.current_player + 1) % 2);
                self.turn_state.is_round_started = false;
            }
            PokerAction::Raise(amount) => {
                if amount < 1 || amount > 99 {
                    return Err(());
                }
                self.chips_state
                    .bet_chips(self.turn_state.current_player, amount);
                self.turn_state.advance_player();
            }
            PokerAction::StartRound => {
                self.turn_state.start_new_round();
                self.bet_blinds();
            }
        }

        Ok(self.turn_state.current_player)
    }

    fn is_valid_action_at_current_time(&self, action: &PokerAction) -> bool {
        self.turn_state.is_action_valid(action)
    }

    fn bet_blinds(&mut self) {
        self.chips_state.bet_chips(self.turn_state.big_blind, 2);
        self.chips_state.bet_chips(self.turn_state.small_blind(), 1);
    }
}

#[derive(PartialEq, Eq)]
pub enum PokerAction {
    CallOrCheck,
    Fold,
    Raise(u32),
    StartRound,
}
