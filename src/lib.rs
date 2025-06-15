use std::{cmp::Ordering, collections::HashMap};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Suit {
    Hearts,
    Spades,
    Diamonds,
    Clubs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Card {
    suit: Suit,
    value: u8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Ranking {
    HighCard,
    Pair(u8),
    TwoPairs(u8, u8),
    ThreeOfAKind(u8),
    Straight(u8),
    Flush,
    FullHouse(u8, u8),
    FourOfAKind(u8),
    StraightFlush(u8),
}

use Ranking::*;

#[allow(dead_code)]
#[derive(Debug)]
struct Hand([Card; 5]);

impl Hand {
    fn as_sorted(&self) -> Hand {
        let mut cards = self.0.clone();
        cards.sort_by_key(|card| card.value);
        cards.reverse();
        Hand(cards)
    }

    fn as_values(&self) -> [u8; 5] {
        self.as_sorted().0.clone().map(|card| card.value)
    }

    fn get_values_with_ace_as_one(&self) -> [u8; 5] {
        let mut values = self.as_values();

        for val in &mut values {
            if *val == 14 {
                *val = 1;
            }
        }
        values.sort();
        values.reverse();
        values
    }

    fn get_ranking(&self) -> Ranking {
        return self
            .try_get_flush()
            .or_else(|| self.try_get_straight())
            .or_else(|| self.try_get_group_based_ranking())
            .unwrap_or(HighCard);
    }

    fn try_get_flush(&self) -> Option<Ranking> {
        let suit = &self.0[0].suit;
        if self.0.iter().all(|card| card.suit == *suit) {
            if let Some(Straight(s)) = self.try_get_straight() {
                Some(StraightFlush(s))
            } else {
                Some(Flush)
            }
        } else {
            None
        }
    }

    fn try_get_straight(&self) -> Option<Ranking> {
        fn find_straight(values: [u8; 5]) -> Option<u8> {
            if values.windows(2).all(|w| w[0] == w[1] + 1) {
                Some(values[0])
            } else {
                None
            }
        }

        find_straight(self.as_values())
            .or_else(|| find_straight(self.get_values_with_ace_as_one()))
            .map(Straight)
    }

    fn try_get_group_based_ranking(&self) -> Option<Ranking> {
        match self.get_groups()[..] {
            [(v, 4)] => Some(FourOfAKind(v)),
            [(v1, 3), (v2, 2)] => Some(FullHouse(v1, v2)),
            [(v, 3)] => Some(ThreeOfAKind(v)),
            [(v1, 2), (v2, 2)] => Some(TwoPairs(v1, v2)),
            [(v, 2)] => Some(Pair(v)),
            _ => None,
        }
    }

    fn get_groups(&self) -> Vec<(u8, usize)> {
        let mut groups: HashMap<u8, usize> = HashMap::new();

        let values = self.as_values();
        let mut last_val = values[0];

        for value in values[1..].iter() {
            if *value == last_val {
                *groups.entry(*value).or_insert(1) += 1;
            }
            last_val = *value;
        }

        let mut groups: Vec<(u8, usize)> = groups.iter().map(|(&k, &v)| (k, v)).collect();
        groups.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
        groups
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_ranking = self.get_ranking();
        let other_ranking = other.get_ranking();
        let ranking_comp = self_ranking.cmp(&other_ranking);

        if ranking_comp != Ordering::Equal {
            return ranking_comp;
        }

        self.as_values().cmp(&other.as_values())
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Hand {}

#[cfg(test)]
mod tests {
    use super::Suit::*;
    use super::*;

    #[test]
    fn test_hands_with_same_value_are_equal() {
        assert_hands_are_equal("H4 D4 D13 C12 H10", "D10 C13 C4 H12 S4");
    }

    #[test]
    fn test_all_pairs_of_hands() {
        let hands = [
            "D3 D6 S12 H9 C4",
            "D3 D10 S13 H9 C2",
            "H2 H5 S10 C9 D13",
            "D3 D9 C9 S4 H13",
            "H3 C4 H10 D10 D9",
            "H2 H5 S10 C10 S9",
            "D6 H6 H3 S3 H10",
            "H2 C2 C7 S7 H9",
            "H3 H7 S8 S3 C3",
            "C14 C2 D3 D4 S5",
            "H4 S5 C8 H7 H6",
            "D5 D8 D2 D12 D9",
            "S5 C5 H5 S8 C8",
            "S5 C5 H8 S8 C8",
            "H4 S4 C4 D4 S9",
            "H14 H2 H3 H4 H5",
            "C14 C13 C12 C11 C10",
        ];

        for (i, hand) in hands.iter().enumerate() {
            for lesser_hand in hands[..i].iter() {
                assert_first_hand_wins(hand, lesser_hand);
            }
            assert_hands_are_equal(hand, hand);
        }
    }

    fn assert_hands_are_equal(first: &str, second: &str) {
        let comp = create_hand(first).cmp(&create_hand(second));
        assert_eq!(comp, Ordering::Equal);
        assert_eq!(create_hand(first), create_hand(second));
    }

    fn assert_first_hand_wins(first: &str, second: &str) {
        let first_hand = create_hand(first);
        let second_hand = create_hand(second);

        assert!(
            first_hand > second_hand,
            "Hand >>{first}<< should win over >>{second}<< but didn't"
        );
        assert!(
            second_hand < first_hand,
            "Hand >>{first}<< didn't win over >>{second}<< in the second try"
        );
    }

    fn create_hand(hand_as_str: &str) -> Hand {
        let mut x = hand_as_str.split(' ').map(create_card);
        Hand([
            x.next().expect("Not enough cards"),
            x.next().expect("Not enough cards"),
            x.next().expect("Not enough cards"),
            x.next().expect("Not enough cards"),
            x.next().expect("Not enough cards"),
        ])
    }

    fn create_card(card_as_str: &str) -> Card {
        let suit = card_as_str.chars().next();

        let suit = match suit.expect("can't make a card from an empty string") {
            'H' => Hearts,
            'S' => Spades,
            'D' => Diamonds,
            'C' => Clubs,
            _ => panic!("Not a valid suit"),
        };

        let value: u8 = card_as_str[1..].parse().expect("Not a valid number");
        Card { suit, value }
    }
}
