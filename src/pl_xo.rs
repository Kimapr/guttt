use crate::tic_tac_toe::Player;
use crate::terminal_render::*;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum pl_XO {
  X,
  O,
  C
}

use pl_XO::*;

impl Player for pl_XO {
  fn next_player(&self) -> Self {
    match self {
      X => O,
      O => C,
      C => X,
    }
  }
  fn get_uuid(&self) -> String {
    String::from(match self {
      X => "two_players_X",
      O => "two_players_O",
      C => "two_players_C"
    })
  }
}
