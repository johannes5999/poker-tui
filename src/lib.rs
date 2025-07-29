pub mod core_engine;

pub struct GameState {
    chips: Vec<u32>,
    big_blind: usize,
    players: usize,
}

#[derive(PartialEq, Eq)]
pub enum PokerAction {
    CallOrCheck,
    Fold,
    Raise(u32),
}

impl GameState {
    pub fn init(players: usize) -> Option<Self> {
        if players > 1 {
            Some(GameState {
                chips: vec![100; players],
                big_blind: players - 1,
                players,
            })
        } else {
            None
        }
    }

    pub fn start_play_hand(&self) -> (HandState, usize) {
        let hs = HandState::init(self.players, self.big_blind, self.chips.clone());
        let first = hs.turn.first_player;
        (hs, first)
    }

    pub fn apply_played_hand(&self, hand: HandState) -> Self {
        Self {
            chips: hand.chips.get_stacks(),
            big_blind: (self.big_blind + 1) % self.players,
            players: self.players,
        }
    }

    pub fn current_chips(&self, player: usize) -> u32 {
        self.chips[player]
    }
}

pub struct HandState {
    chips: ChipsState,
    turn: TurnState,
    players: usize,
}

pub enum TurnResult {
    NextPlayer(usize),
    WonHand(usize),
}

use TurnResult::*;

#[derive(Debug)]
pub struct RaiseByTooMuch();

impl HandState {
    fn init(players: usize, big_blind: usize, chips: Vec<u32>) -> Self {
        let mut slf = HandState {
            chips: ChipsState::init(chips),
            turn: TurnState::init(players, (big_blind + 1) % players),
            players,
        };
        slf.bet_blinds(big_blind);
        slf
    }

    fn bet_blinds(&mut self, big_blind: usize) {
        let small_blind = if big_blind == 0 {
            self.players - 1
        } else {
            big_blind - 1
        };
        self.chips.bet_chips(big_blind, 2);
        self.chips.bet_chips(small_blind, 1);
    }

    pub fn current_chips(&self, player: usize) -> u32 {
        self.chips.current_chips(player)
    }

    pub fn play_action(&mut self, action: PokerAction) -> Result<TurnResult, RaiseByTooMuch> {
        match action {
            PokerAction::CallOrCheck => {
                self.chips.call(self.turn.current_player);
                self.turn.advance_player();
                if self.turn.rounds > 3 {
                    self.chips.win_pot(0);
                    return Ok(WonHand(0));
                }
            }
            PokerAction::Fold => {
                if self.turn.fold_current_player() {
                    self.chips.win_pot(self.turn.current_player);
                    return Ok(WonHand(0));
                }
            }
            PokerAction::Raise(amount) => {
                if amount < 1 || amount > 99 {
                    return Err(RaiseByTooMuch());
                }
                self.chips.bet_chips(self.turn.current_player, amount);
                self.turn.advance_player_raise();
            }
        }

        Ok(NextPlayer(self.turn.current_player))
    }
}
#[derive(Clone)]
struct ChipsState {
    player_chips: Vec<PlayerChips>,
    pot: u32,
}

#[derive(Clone)]
struct PlayerChips {
    stack: u32,
    bet: u32,
}

impl ChipsState {
    fn init(chips: Vec<u32>) -> Self {
        Self {
            player_chips: chips
                .iter()
                .map(|&c| PlayerChips { stack: c, bet: 0 })
                .collect(),
            pot: 0,
        }
    }

    fn get_stacks(&self) -> Vec<u32> {
        self.player_chips.iter().map(|pc| pc.stack).collect()
    }

    fn bet_chips(&mut self, player: usize, amount: u32) {
        self.player_chips[player].stack -= amount;
        self.player_chips[player].bet += amount;
    }

    fn win_pot(&mut self, player: usize) {
        self.move_chips_to_pot();
        self.player_chips[player].stack += self.pot;
        self.pot = 0;
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

#[derive(Clone)]
struct TurnState {
    current_player: usize,
    first_player: usize,
    players: usize,
    active_players: Vec<bool>,
    turns_since_action: usize,
    rounds: usize,
}

impl TurnState {
    fn init(players: usize, first_player: usize) -> Self {
        Self {
            current_player: first_player,
            first_player,
            players,
            active_players: vec![true; players],
            turns_since_action: 0,
            rounds: 0,
        }
    }

    fn advance_player_raise(&mut self) {
        self.turns_since_action = 0;
        self.advance_player();
    }

    fn advance_player(&mut self) {
        self.turns_since_action += 1;

        if self.should_start_next_betting_round() {
            self.current_player = self.first_player;
            self.rounds += 1;
            self.turns_since_action = 0;
        } else {
            self.current_player = (self.current_player + 1) % self.players;
        }

        while !self.active_players[self.current_player] {
            self.advance_player();
        }
    }

    fn should_start_next_betting_round(&self) -> bool {
        self.turns_since_action == self.players
    }

    fn fold_current_player(&mut self) -> bool {
        self.active_players[self.current_player] = false;
        self.advance_player();
        self.active_players.iter().filter(|&&a| a).count() == 1
    }
}

#[cfg(test)]
mod two_player_tests {
    use crate::core_engine::*;

    use super::*;
    use PokerAction::*;

    #[test]
    fn should_not_be_able_to_play_with_less_than_2_players() {
        assert!(GameState::init(0).is_none());
        assert!(GameState::init(1).is_none());
    }

    #[test]
    fn should_fail_on_raise_out_of_bounds() {
        let (mut hs, _) = GameState::init(2).unwrap().start_play_hand();

        assert!(hs.play_action(Raise(0)).is_err());
        assert!(hs.play_action(Raise(100)).is_err());
    }

    #[test]
    fn should_start_and_deduct_blind() {
        let mut sut = GameTestContainer::init(2);

        sut.then_score_is(&[100, 100]);

        sut.when_start_round();
        sut.then_score_is(&[99, 98]);
    }

    #[test]
    fn should_win_blind_when_other_player_folds() {
        let mut sut = GameTestContainer::init(2);
        sut.when_start_round();

        sut.when_player_plays(0, Fold);
        sut.then_score_is(&[99, 101]);

        sut.when_start_round();
        sut.then_score_is(&[97, 100]);

        sut.when_player_plays(1, Fold);
        sut.then_score_is(&[100, 100]);
    }

    #[test]
    fn should_win_blind_if_raise_then_fold() {
        let mut sut = GameTestContainer::init(2);
        sut.when_start_round();

        sut.when_player_plays(0, CallOrCheck);
        sut.then_score_is(&[98, 98]);

        sut.when_player_plays(1, Raise(2));
        sut.when_player_plays(0, Fold);

        sut.then_score_is(&[98, 102]);
    }

    #[test]
    fn should_win_more_after_raise_is_called() {
        let mut sut = GameTestContainer::init(2);
        sut.when_start_round();

        sut.when_player_plays(0, Raise(9));
        sut.then_score_is(&[90, 98]);

        sut.when_player_plays(1, CallOrCheck);
        sut.then_score_is(&[90, 90]);

        sut.when_player_plays(0, Raise(10));
        sut.when_player_plays(1, Fold);
        sut.then_score_is(&[110, 90]);
    }

    #[test]
    fn should_win_blind_three_players() {
        let mut sut = GameTestContainer::init(3);

        sut.when_start_round();
        sut.then_score_is(&[100, 99, 98]);

        sut.when_player_plays(0, Fold);
        sut.when_player_plays(1, Fold);

        sut.then_score_is(&[100, 99, 101]);
    }

    #[test]
    fn should_allow_one_player_to_fold() {
        let mut sut = GameTestContainer::init(3);
        sut.when_start_round();
        sut.when_player_plays(0, CallOrCheck);
        sut.when_player_plays(1, Fold);
        sut.when_player_plays(2, CallOrCheck);
        sut.then_score_is(&[98, 99, 98]);

        sut.when_player_plays(0, CallOrCheck);
        sut.when_player_plays(2, Raise(2));
        sut.when_player_plays(0, Fold);
        sut.then_score_is(&[98, 99, 103]);
    }

    #[test]
    fn should_start_second_round_of_betting_with_player0() {
        let mut sut = GameTestContainer::init(3);
        sut.when_start_round();

        sut.when_player_plays(0, CallOrCheck);
        sut.when_player_plays(1, Raise(2));
        sut.when_player_plays(2, CallOrCheck);
        sut.when_player_plays(0, CallOrCheck);

        sut.then_next_turn_is(0);

        sut.when_player_plays(0, Raise(2));
        sut.when_player_plays(1, CallOrCheck);
        sut.when_player_plays(2, CallOrCheck);

        sut.then_next_turn_is(0);
    }

    #[test]
    fn should_win_if_have_better_hand() {
        let gen = given_cards_are(&[
            &["H13 D13", "H2 D7", "C8 C4 H3 S12 S10"],
            &["H2 D7", "H13 D13", "C8 C4 H3 S12 S10"],
        ]);
        let mut sut = GameTestContainer::init_gen(2, gen);
        sut.when_start_round();

        sut.when_call_until_round_over();
        sut.then_score_is(&[102, 98]);
    }

    struct GameTestContainer {
        gs: GameState,
        hs: Option<HandState>,
        actual_next_player: Option<usize>,
    }

    impl GameTestContainer {
        fn init_gen(players: usize, deck_gen: impl DeckGenerator) -> GameTestContainer {
            Self::init(players)
        }

        fn init(players: usize) -> GameTestContainer {
            GameTestContainer {
                gs: GameState::init(players).unwrap(),
                hs: None,
                actual_next_player: None,
            }
        }

        fn then_score_is(&self, expected: &[u32]) {
            for (player, score) in expected.iter().enumerate() {
                self.then_player_has_chips(player, *score);
            }
        }

        fn then_player_has_chips(&self, player: usize, expected_chips: u32) {
            let actual_chips = self
                .hs
                .as_ref()
                .map(|c| c.current_chips(player))
                .unwrap_or(self.gs.current_chips(player));
            assert_eq!(actual_chips, expected_chips);
        }

        fn then_next_turn_is(&self, expected: usize) {
            assert_eq!(self.actual_next_player, Some(expected));
        }

        fn when_start_round(&mut self) -> () {
            let (hs, first) = self.gs.start_play_hand();
            self.hs = Some(hs);
            self.actual_next_player = Some(first);
            //self.actual_next_player = self.gs.play_action(PokerAction::StartRound).unwrap();
        }

        fn when_player_plays(&mut self, player: usize, action: PokerAction) -> () {
            assert_eq!(player, self.actual_next_player.unwrap());
            match self.hs.as_mut().unwrap().play_action(action).unwrap() {
                NextPlayer(p) => self.actual_next_player = Some(p.clone()),
                WonHand(_) => {
                    let x = self.hs.take();
                    self.gs = self.gs.apply_played_hand(x.unwrap());
                    self.actual_next_player = None
                }
            }
        }

        fn when_call_until_round_over(&mut self) -> () {
            for _ in 0..100 {
                if let WonHand(_) = self.hs.as_mut().unwrap().play_action(CallOrCheck).unwrap() {
                    return;
                }
            }
            assert!(false, "round didn't finish after 100 checks");
        }
    }

    fn given_cards_are(cards: &[&[&str]]) -> StackedDeckGenerator {
        fn to_cards(s: &&str) -> Vec<Card> {
            s.split_ascii_whitespace()
                .map(|c| Card::try_from(c).unwrap())
                .collect()
        }

        StackedDeckGenerator {
            cards: cards
                .iter()
                .map(|c| c.iter().map(to_cards).flatten().rev().collect())
                .collect(),
        }
    }

    struct StackedDeckGenerator {
        cards: Vec<Vec<core_engine::Card>>,
    }

    impl DeckGenerator for StackedDeckGenerator {
        fn shuffle(&mut self) -> Deck {
            Deck::init(self.cards.pop().unwrap())
        }
    }
}
