#![allow(unused)]

use rc_pool::{RcVecPool, StrongRef, StrongRefTrait, WeakRef, WeakRefTrait};
use std::cell::RefCell;

struct Player<'t> {
    game: GameRef<'t>,
    name: String,
    friends: RefCell<Vec<PlayerWeakRef<'t>>>,
}

type PlayerRef<'t> = StrongRef<'t, Player<'t>>;
type PlayerWeakRef<'t> = WeakRef<'t, Player<'t>>;

struct Game<'t> {
    players: &'t RcVecPool<Player<'t>>,
}

type GameRef<'t> = &'t Game<'t>;

impl<'t> Game<'t> {
    fn new(players: &'t RcVecPool<Player<'t>>) -> Self {
        Self { players }
    }

    fn add_player(&'t self, name: &str) -> PlayerRef<'t> {
        let p = self
            .players
            .insert(Player {
                game: self,
                name: name.to_owned(),
                friends: Default::default(),
            })
            .unwrap();

        p
    }
}

impl<'t> PartialEq for Game<'t> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

fn main() {
    let players = RcVecPool::new_vec(100);
    let game = Game::new(&players);
    let p1 = game.add_player("Sune");
    let p2 = game.add_player("Berra");
    p1.friends.borrow_mut().push(p2.weak());
    p2.friends.borrow_mut().push(p1.weak());

    for _ in 0..2 {
        for p in game.players.iter() {
            println!("{}", p.name);

            for f in p.friends.borrow().iter().flat_map(|r| r.strong()) {
                println!("  {}", f.name);
            }

            println!();
        }
    }
}
