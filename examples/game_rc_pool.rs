#![allow(unused)]

use rc_pool::{RcPool, StrongRef, StrongRefTrait, WeakRef, WeakRefTrait};
use std::cell::RefCell;

struct Player<'t> {
    game: GameRef<'t>,
    name: String,
    friends: Vec<PlayerWeakRef<'t>>,
}

type PlayerRef<'t> = StrongRef<'t, Player<'t>>;
type PlayerWeakRef<'t> = WeakRef<'t, Player<'t>>;

struct Game<'t> {
    players: &'t RcPool<Player<'t>>,
}

impl<'t> Game<'t> {
    fn new(players: &'t RcPool<Player<'t>>) -> Self {
        Self { players }
    }

    fn add_player(&'t self, name: &str) -> PlayerRef<'t> {
        self.players.insert(Player {
            game: self,
            name: name.to_owned(),
            friends: Default::default(),
        })
    }
}

impl<'t> PartialEq for Game<'t> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

type GameRef<'t> = &'t Game<'t>;

fn main() {
    let players = RcPool::new(1);
    let game = Game::new(&players);
    let mut p1 = game.add_player("Sune");
    let mut p2 = game.add_player("Berra");
    p1.borrow_mut().friends.push(p2.weak());
    p2.borrow_mut().friends.push(p1.weak());

    for _ in 0..2 {
        for p in game.players.iter() {
            println!("{}", p.name);

            for f in p.friends.iter().flat_map(|r| r.strong()) {
                println!("  {}", f.name);
            }

            println!();
        }
    }
}
