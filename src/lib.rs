pub mod core_engine;

use core_engine::Card;
use core_engine::Deck;
use core_engine::Hand;
use TurnResult::*;

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

    pub fn start_play_hand(&self, deck: Deck) -> (HandState, usize) {
        let hs = HandState::init(self.players, self.big_blind, self.chips.clone(), deck);
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
    deck: Deck,
    players: usize,
    hands: Vec<(Card, Card)>,
    board: Vec<Card>,
}

pub enum TurnResult {
    NextPlayer(usize),
    WonHand(usize),
}

#[derive(Debug)]
pub struct RaiseByTooMuch();

impl HandState {
    fn init(players: usize, big_blind: usize, chips: Vec<u32>, deck: Deck) -> Self {
        let (deck, hands) = draw_starting_hands(players, deck);
        let mut slf = HandState {
            chips: ChipsState::init(chips),
            turn: TurnState::init(players, (big_blind + 1) % players),
            deck,
            players,
            hands,
            board: vec![],
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

    pub fn get_hand(&self, player: usize) -> (Card, Card) {
        self.hands[player].clone()
    }

    pub fn get_board(&self) -> Vec<Card> {
        self.board.clone()
    }

    pub fn play_action(&mut self, action: PokerAction) -> Result<TurnResult, RaiseByTooMuch> {
        match action {
            PokerAction::CallOrCheck => {
                self.chips.call(self.turn.current_player);
                if self.turn.advance_player() {
                    if self.turn.rounds > 3 {
                        let win = self.get_winning_player();
                        self.chips.win_pot(win);
                        return Ok(WonHand(win));
                    }

                    if self.turn.rounds == 1 {
                        self.board.extend(self.deck.draw_multiple(3));
                    } else {
                        self.board.push(self.deck.draw());
                    }
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

    fn get_winning_player(&self) -> usize {
        (0..self.players)
            .max_by_key(|p| self.best_hand_for_user(*p))
            .unwrap()
    }

    fn best_hand_for_user(&self, player: usize) -> Hand {
        best_hand_from_cards([
            self.hands[player].0,
            self.hands[player].1,
            self.board[0],
            self.board[1],
            self.board[2],
            self.board[3],
            self.board[4],
        ])
    }
}

fn best_hand_from_cards(cards: [Card; 7]) -> Hand {
    fn combos(cards: &[Card; 7], start: usize, prefix: Vec<Card>, collector: &mut Vec<Hand>) {
        if prefix.len() == 5 {
            collector.push(Hand([
                prefix[0], prefix[1], prefix[2], prefix[3], prefix[4],
            ]));
        } else if 7 - start + prefix.len() == 5 {
            let mut prefix = prefix;
            prefix.extend(&cards[start..]);
            collector.push(Hand([
                prefix[0], prefix[1], prefix[2], prefix[3], prefix[4],
            ]));
        } else {
            combos(cards, start + 1, prefix.clone(), collector);
            let mut prefix = prefix;
            prefix.push(cards[start]);
            combos(cards, start + 1, prefix, collector);
        }
    }
    let mut collector: Vec<Hand> = Vec::new();
    combos(&cards, 0, vec![], &mut collector);
    collector.into_iter().max().unwrap()
}

fn draw_starting_hands(players: usize, deck: Deck) -> (Deck, Vec<(Card, Card)>) {
    let mut deck = deck;
    let mut hands: Vec<(Card, Card)> = vec![];
    for _ in 0..players {
        let card1 = deck.draw();
        let card2 = deck.draw();
        hands.push((card1, card2));
    }
    (deck, hands)
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

    fn advance_player(&mut self) -> bool {
        let mut new_round = false;
        self.turns_since_action += 1;

        if self.should_start_next_betting_round() {
            self.current_player = self.first_player;
            self.rounds += 1;
            new_round = true;
            self.turns_since_action = 0;
        } else {
            self.current_player = (self.current_player + 1) % self.players;
        }

        while !self.active_players[self.current_player] {
            self.advance_player();
        }

        new_round
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
        let (mut hs, _) = GameState::init(2)
            .unwrap()
            .start_play_hand(Deck::ordered_deck());

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
    fn should_show_cards_at_correct_time() {
        let mut sut = GameTestContainer::init(2);
        sut.when_start_round_with_deck(deck_from_strings(&["D2 D7", "S14 C10", "H2 H3 H4 H5 H6"]));
        let mut hs = sut.hs.unwrap();

        let expected_boards: Vec<&str> = vec!["", "H2 H3 H4", "H2 H3 H4 H5", "H2 H3 H4 H5 H6"];

        for board in expected_boards {
            assert_hands_are_correct(&hs);
            assert_eq!(hs.get_board(), to_cards(&board));

            hs.play_action(CallOrCheck).unwrap();
            hs.play_action(CallOrCheck).unwrap();
        }
    }

    fn assert_hands_are_correct(hs: &HandState) {
        assert_eq!(
            hs.get_hand(0),
            (Card::try_from("D2").unwrap(), Card::try_from("D7").unwrap())
        );
        assert_eq!(
            hs.get_hand(1),
            (
                Card::try_from("S14").unwrap(),
                Card::try_from("C10").unwrap()
            )
        );
    }

    #[test]
    fn should_win_when_have_better_hand() {
        fn assert_player_wins_given_cards(players: usize, cards: &[&str], expected_winner: usize) {
            let mut sut = GameTestContainer::init(players);
            sut.when_start_round_with_deck(deck_from_strings(cards));
            sut.when_call_until_player_wins(expected_winner);

            for p in 0..players {
                if p == expected_winner {
                    sut.then_player_has_chips(p, 98 + 2 * players as u32);
                } else {
                    sut.then_player_has_chips(p, 98);
                }
            }
        }

        assert_player_wins_given_cards(2, &["H2 D7", "D5 C11", "C7 H4 C10 H14 H12"], 0);
        assert_player_wins_given_cards(2, &["H2 D7", "D5 C11", "C7 H4 C10 H11 H12"], 1);
        assert_player_wins_given_cards(3, &["H2 D7", "D5 C11", "H5 H8", "C7 H4 C10 H11 H12"], 2);
    }

    #[test]
    fn should_compare_hands_multiple_rounds() {
        const P1_WINS_HAND: &[&str; 3] = &["H13 D13", "H2 D7", "C8 C4 H3 S12 S10"];
        const P2_WINS_HAND: &[&str; 3] = &["H2 D7", "H13 D13", "C8 C4 H3 S12 S10"];

        let mut sut = GameTestContainer::init(2);
        sut.when_start_round_with_deck(deck_from_strings(P1_WINS_HAND));

        sut.when_call_until_player_wins(0);
        sut.then_score_is(&[102, 98]);

        sut.when_start_round_with_deck(deck_from_strings(P2_WINS_HAND));
        sut.when_player_plays(1, Raise(9));
        sut.when_call_until_player_wins(1);

        sut.then_score_is(&[92, 108]);
    }

    struct GameTestContainer {
        gs: GameState,
        hs: Option<HandState>,
        actual_next_player: Option<usize>,
    }

    impl GameTestContainer {
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
            self.when_start_round_with_deck(Deck::ordered_deck());
        }

        fn when_start_round_with_deck(&mut self, deck: Deck) -> () {
            let (hs, first) = self.gs.start_play_hand(deck);
            self.hs = Some(hs);
            self.actual_next_player = Some(first);
        }

        fn when_player_plays(&mut self, player: usize, action: PokerAction) -> () {
            assert_eq!(player, self.actual_next_player.unwrap());
            match self.hs.as_mut().unwrap().play_action(action).unwrap() {
                NextPlayer(p) => self.actual_next_player = Some(p),
                WonHand(_) => {
                    let x = self.hs.take();
                    self.gs = self.gs.apply_played_hand(x.unwrap());
                    self.actual_next_player = None
                }
            }
        }

        fn when_call_until_player_wins(&mut self, expected_winner: usize) -> () {
            for _ in 0..100 {
                if let WonHand(p) = self.hs.as_mut().unwrap().play_action(CallOrCheck).unwrap() {
                    let x = self.hs.take();
                    self.gs = self.gs.apply_played_hand(x.unwrap());
                    self.actual_next_player = None;
                    assert_eq!(p, expected_winner, "unexpected player won");
                    return;
                }
            }
            assert!(false, "round didn't finish after 100 checks");
        }
    }

    fn deck_from_strings(cards: &[&str]) -> Deck {
        Deck::init(cards.iter().map(to_cards).flatten().rev().collect())
    }

    fn to_cards(s: &&str) -> Vec<Card> {
        s.split_ascii_whitespace()
            .map(|c| Card::try_from(c).unwrap())
            .collect()
    }
}
