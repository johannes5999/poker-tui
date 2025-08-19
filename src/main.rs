use poker_tui::{
    core_engine::Deck,
    GameState,
    PokerAction::{self, *},
};
use std::io;

fn main() {
    println!("Welcome to PokerTUI!");

    println!("How many players will be playing?");

    let mut players = String::new();

    io::stdin()
        .read_line(&mut players)
        .expect("Expected an input");

    let players: usize = players
        .trim()
        .parse()
        .expect("Please provide a positive number");

    let mut gs = GameState::init(players).unwrap();

    loop {
        println!("\n\nNEW HAND\n\n");
        gs = play_hand(gs);
    }
}

fn play_hand(gs: GameState) -> GameState {
    let (mut hs, mut cur) = gs.start_play_hand(Deck::shuffled_deck());
    loop {
        let board: Vec<String> = hs.get_board().iter().map(|c| c.pretty_print()).collect();
        println!(
            "Player {}, you're up. Your hand is {} {}. The Board is {}. You have {} chips. What do you play?",
            cur,
            hs.get_hand(cur).0.pretty_print(),
            hs.get_hand(cur).1.pretty_print(),
            board.join(" "),
            hs.current_chips(cur),
        );

        let mut action_str = String::new();
        io::stdin()
            .read_line(&mut action_str)
            .expect("Expected an input");
        let action = parse_action(&action_str);

        match action {
            Some(a) => {
                print!("{}", pretty_print_action(&a, cur));
                match hs.play_action(a) {
                    Ok(poker_tui::TurnResult::NextPlayer(p)) => {
                        println!(" They have {} chips left", hs.current_chips(cur));
                        cur = p
                    }
                    Ok(poker_tui::TurnResult::WonHand(p)) => {
                        println!("Player {} won the round", p);
                        return gs.apply_played_hand(hs);
                    }
                    Err(_) => println!("Raised by too much"),
                }
            }
            None => {
                println!("Invalid action {}", action_str);
            }
        }
    }
}

fn parse_action(as_str: &str) -> Option<PokerAction> {
    let action = as_str.chars().next()?;
    match action.to_ascii_lowercase() {
        'c' => Some(CallOrCheck),
        'f' => Some(Fold),
        'r' => Some(Raise((as_str[2..].trim()).parse().ok()?)),
        _ => None,
    }
}

fn pretty_print_action(action: &PokerAction, player: usize) -> String {
    match action {
        CallOrCheck => format!("Player {player} called or checked."),
        Fold => format!("Player {player} folded"),
        Raise(v) => format!("Player {player} raised by {v} chips."),
    }
}
