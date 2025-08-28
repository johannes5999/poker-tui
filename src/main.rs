use poker_tui::{
    core_engine::Deck,
    GameState, HandSnapshot, HandVisibility,
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
        println!("");
        for line in pretty_print_hand_snapshot(hs.spectator_snapshot()) {
            println!("    {}", line);
        }

        let mut action_str = String::new();
        io::stdin()
            .read_line(&mut action_str)
            .expect("Expected an input");
        let action = parse_action(&action_str);

        match action {
            Some(a) => {
                println!("{}", pretty_print_action(&a, cur));
                match hs.play_action(a) {
                    Ok(poker_tui::TurnResult::NextPlayer(p)) => cur = p,
                    Ok(poker_tui::TurnResult::WonHand(p)) => {
                        println!();
                        println!("###########################");
                        println!("# Player {} won the round #", p);
                        println!("###########################");
                        println!();
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
        CallOrCheck => format!("\nPlayer {player} called or checked."),
        Fold => format!("\nPlayer {player} folded"),
        Raise(v) => format!("\nPlayer {player} raised by {v} chips."),
    }
}

fn pretty_print_hand_snapshot(snapshot: HandSnapshot) -> Vec<String> {
    let divider = "-".repeat(snapshot.players * 12 - 3);
    let pot = format!("Current pot: {} chips", snapshot.pot);

    let board = (0..5)
        .map(|i| {
            snapshot
                .board
                .get(i)
                .map(|c| format!("{:<4}", c.pretty_print()))
                .unwrap_or("???".to_owned())
        })
        .collect::<Vec<_>>()
        .join(" ");

    let bets = snapshot
        .chips
        .iter()
        .map(|pc| format!("bet: {:>4}", pc.bet))
        .collect::<Vec<_>>()
        .join(" | ");

    let hands = snapshot
        .hands
        .iter()
        .map(pretty_print_hand)
        .collect::<Vec<_>>()
        .join(" | ");

    let stacks = snapshot
        .chips
        .iter()
        .map(|pc| format!("{:>9}", pc.stack))
        .collect::<Vec<_>>()
        .join(" | ");

    let player_pointer = "            ".repeat(snapshot.current_player) + "    🔼";

    let call_to_action = format!("Player {}, what do you do?", snapshot.current_player);
    let call_or_check = if snapshot.expected_call == 0 {
        "(C)heck".to_owned()
    } else {
        format!("(C)all {}", snapshot.expected_call)
    };
    let actions = format!("{}  (R)aise (F)old", call_or_check);

    vec![
        divider.clone(),
        String::new(),
        pot,
        String::new(),
        board,
        String::new(),
        hands,
        bets,
        stacks,
        player_pointer,
        String::new(),
        divider,
        String::new(),
        call_to_action,
        actions,
    ]
}

fn pretty_print_hand(h: &HandVisibility) -> String {
    match *h {
        HandVisibility::Visible(c1, c2) => {
            format!("{:<4} {:<4}", c1.pretty_print(), c2.pretty_print())
        }
        HandVisibility::Folded => "  FOLD   ".to_owned(),
    }
}
