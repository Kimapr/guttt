use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub trait GenericGame<P: Player> {
    fn create(player: P, pos: Position) -> Self;
    fn mov(&mut self, m: Rc<dyn Any>) -> MoveData<P>;
    fn get_valid_moves(&self) -> Vec<Rc<dyn Any>>;
    fn is_valid_move(&self, m: Rc<dyn Any>) -> bool;
    fn set_player(&mut self, player: P);
    fn get_player(&self) -> P;
}

#[derive(Debug)]
pub struct Position {
    cid: u8,
    x: u8,
    y: u8,
}

impl Default for Position {
  fn default() -> Self {
    Self::from_cid(0)
  }
}

impl Clone for Position {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Position {}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.cid == other.cid
    }
}

impl Position {
    pub fn from_xy(x: u8, y: u8) -> Position {
        assert!(
            x < 3 && y < 3,
            format!("vector ({},{}) is out of tic tac toe bounds!", x, y)
        );
        Position {
            cid: y * 3 + x,
            x,
            y,
        }
    }

    pub fn from_cid(cid: u8) -> Position {
        assert!(
            cid < 9,
            format!("cid {} is out of tic tac toe bounds!", cid)
        );
        Position {
            cid,
            x: cid % 3,
            y: cid / 3,
        }
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }

    pub fn get_cid(&self) -> u8 {
        self.cid
    }
}

pub trait Player: Copy {
    fn next_player(&self) -> Self;
    fn get_uuid(&self) -> String;
}

#[derive(Debug)]
pub struct MoveData<P: Player> {
    pub jump: Option<Vec<Position>>,
    pub next_player: P,
    pub result: GameResult<P>,
}

#[derive(Debug, Copy, Clone)]
pub enum GameResult<P: Player> {
    Won(P),
    Draw,
    Incomplete,
}

#[derive(Clone)]
pub enum SubgameState<T: GenericGame<P>,P: Player> {
    Won(P,T),
    Draw(T),
    Playing(T),
    None
}

impl<T: GenericGame<P>,P: Player> Default for SubgameState<T,P> {
    fn default() -> Self {
        SubgameState::None
    }
}
