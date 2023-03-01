use rc_pool::{RcPool, StrongRef, StrongRefTrait, WeakRef, WeakSliceExt};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

trait Game
where
    Self: 'static,
{
    type Player: Player<Game = Self>;
    fn add_player(&'static self, name: &str) -> Self::Player;
    fn list_players(&'static self);
}

trait Player {
    type Game: Game<Player = Self>;
    fn add_friend(&mut self, player: &Self);
}

const MANUAL_DROP: bool = true;
type PlayerPool = RcPool<RcPoolPlayer, MANUAL_DROP>;
type PlayerPoolRef = &'static PlayerPool;

struct RcPoolGame {
    players: PlayerPoolRef,
}

impl PartialEq for RcPoolGame {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

impl Game for RcPoolGame {
    type Player = StrongRef<'static, RcPoolPlayer, MANUAL_DROP>;

    fn add_player(&'static self, name: &str) -> Self::Player {
        self.players.insert(RcPoolPlayer {
            game: self,
            name: name.into(),
            friends: Vec::new(),
        })
    }

    fn list_players(&'static self) {
        for p in self.players.iter() {
            println!("{}", p.name);

            for f in p.friends.iter_strong() {
                println!("  {}", f.name);
            }

            println!();
        }
    }
}

struct RcPoolPlayer {
    #[allow(unused)]
    game: &'static RcPoolGame,

    name: String,
    friends: Vec<WeakRef<'static, RcPoolPlayer, MANUAL_DROP>>,
}

impl Player for StrongRef<'static, RcPoolPlayer, MANUAL_DROP> {
    type Game = RcPoolGame;

    fn add_friend(&mut self, player: &<Self::Game as Game>::Player) {
        self.get_mut().friends.push(player.weak())
    }
}

#[derive(Default)]
struct RcGame {
    players: RefCell<Vec<Rc<RcPlayer>>>,
}

impl PartialEq for RcGame {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

impl Game for RcGame {
    type Player = Rc<RcPlayer>;

    fn add_player(&'static self, name: &str) -> Self::Player {
        let p = Rc::new(RcPlayer {
            game: self,
            name: name.into(),
            friends: RefCell::new(Vec::new()),
        });

        self.players.borrow_mut().push(p.clone());
        p
    }

    fn list_players(&'static self) {
        for p in self.players.borrow().iter() {
            println!("{}", p.name);

            for f in p.friends.borrow().iter_strong() {
                println!("  {}", f.name);
            }

            println!();
        }
    }
}

struct RcPlayer {
    #[allow(unused)]
    game: &'static RcGame,

    name: String,
    friends: RefCell<Vec<Weak<RcPlayer>>>,
}

impl Player for Rc<RcPlayer> {
    type Game = RcGame;

    fn add_friend(&mut self, player: &<Self::Game as Game>::Player) {
        self.friends.borrow_mut().push(player.weak())
    }
}

fn test_game<G: Game>(game: &'static G) {
    let mut p1 = game.add_player("Sune");
    let mut p2 = game.add_player("Berra");
    let mut p3 = game.add_player("Arne");
    let mut p4 = game.add_player("Svenne");

    p1.add_friend(&p2);
    p1.add_friend(&p3);
    p1.add_friend(&p4);
    p2.add_friend(&p1);
    p2.add_friend(&p3);
    p3.add_friend(&p2);
    p3.add_friend(&p4);
    p4.add_friend(&p1);

    drop(p4);

    game.list_players();
}

fn main() {
    {
        let rc_game = Box::leak(Box::new(RcGame::default()));
        test_game(rc_game);
    }

    {
        let players = Box::leak(Box::new(RcPool::new(1000)));
        let rc_pool_game = Box::leak(Box::new(RcPoolGame { players }));
        test_game(rc_pool_game);
    }
}
