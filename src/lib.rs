use std::cmp::Ordering;

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

#[derive(Debug, PartialEq, Eq)]
struct Hand([Card; 5]);

impl Hand {
    fn as_sorted(&self) -> Hand {
        let mut cards = self.0.clone();
        cards.sort_by_key(|card| card.value);
        cards.reverse();
        Hand(cards)
    }

    fn as_values(&self) -> [u8; 5] {
        self.0.clone().map(|card| card.value)
    }

    fn has_pair(&self) -> Option<u8> {
        let values = self.as_sorted().as_values();
        let mut last_val = values[0];

        for value in values[1..].iter() {
            if *value == last_val {
                return Some(last_val);
            }
            last_val = *value;
        }
        None
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_pair = self.has_pair();
        let other_pair = other.has_pair();
        let pair_comp = self_pair.cmp(&other_pair);

        if pair_comp != Ordering::Equal {
            return pair_comp;
        }

        self.as_sorted()
            .as_values()
            .cmp(&other.as_sorted().as_values())
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::Suit::*;
    use super::*;

    #[test]
    fn should_equal_itself() {
        let hand = "H2 H5 S10 C9 D13";
        assert_hands_are_equal(hand, hand);
    }

    #[test]
    fn test_high_card() {
        assert_first_hand_wins("H2 H5 S10 C9 D13", "D3 D6 S12 H9 C4");
        assert_first_hand_wins("H2 H5 S10 C9 D13", "D3 D10 S13 H9 C2");
    }

    #[test]
    fn test_pair() {
        assert_first_hand_wins("H2 H5 S10 C10 S9", "D3 D8 C14 S4 H13");
        assert_first_hand_wins("H2 H5 S10 C10 S9", "D3 D9 C9 S4 H13");
        assert_first_hand_wins("H2 H5 S10 C10 S9", "H3 C4 H10 D10 D9");
    }

    fn assert_hands_are_equal(first: &str, second: &str) {
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
