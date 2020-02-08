use super::*;
use std::cmp::{min,max};

impl<P: Player> PartialEq<Entanglement<P>> for Entanglement<P> {
  fn eq(&self, other: &Entanglement<P>) -> bool {
    (((self.p1==other.p1) && (self.p2==other.p2)) ||
    ((self.p1==other.p2) && (self.p2==other.p1))) &&
    (self.player.get_uuid() == other.player.get_uuid()) &&
    (self.subscript == other.subscript)
  }
}
impl<P: Player> Eq for Entanglement<P> {}

pub trait Subscripted<P: Player> {
  fn get_player(&self) -> P;
  fn get_subscript(&self) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub struct Entanglement<P: Player> {
  pub p1: Position,
  pub p2: Position,
  pub player: P,
  pub subscript: u8
}

impl<P: Player> Subscripted<P> for Entanglement<P> {
  fn get_player(&self) -> P {
    self.player
  }
  fn get_subscript(&self) -> u8 {
    self.subscript
  }
}


impl<P: Player> Entanglement<P> {
  fn measure_n(&self,first: bool) -> (Position,Measurement<P>) {
    if first {
      (self.p1, Measurement{
        player:self.player,
        subscript:self.subscript
      })
    } else {
      (self.p2, Measurement{
        player:self.player,
        subscript:self.subscript
      })
    }
  }
  fn measure_push(&self,pos: Position) -> (Position,Measurement<P>) {
    if self.p2 == pos {
      (self.p1, Measurement{
        player:self.player,
        subscript:self.subscript
      })
    } else if self.p1 == pos {
      (self.p2, Measurement{
        player:self.player,
        subscript:self.subscript
      })
    } else {panic!("fixme")}
  }
}

impl<P: Player> PartialEq<Measurement<P>> for Measurement<P> {
  fn eq(&self, other: &Measurement<P>) -> bool {
    (self.player.get_uuid() == other.player.get_uuid())
  }
}

impl<P: Player> Subscripted<P> for Measurement<P> {
  fn get_player(&self) -> P {
    self.player
  }
  fn get_subscript(&self) -> u8 {
    self.subscript
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Measurement<P: Player> {
  pub player: P,
  pub subscript: u8
}

#[derive(Clone)]
pub struct QuantumTicTacToe<P: Player> {
  classic_marks: [Option<Measurement<P>>;9],
  ents: Vec<Entanglement<P>>,
  player: P,
  fplayer: P,
  subscript: u8,
  do_meas: Option<Entanglement<P>>
}

#[derive(Copy, Clone, Debug)]
pub struct QuantumTicTacToeMoveEnt {
  p1: Position,
  p2: Position,
}

#[derive(Copy, Clone, Debug)]
pub struct QuantumTicTacToeMoveMeas {
  first: bool
}

impl<P: Player> GenericGame<P> for QuantumTicTacToe<P> {
  fn create(player: P, pos: Position) -> Self {
    Self::new(player)
  }
  fn mov(&mut self, m: Rc<dyn Any>) -> MoveData<P> {
    if let Some(e) = self.do_meas {
      let mo = m.downcast_ref::<QuantumTicTacToeMoveMeas>().expect(
          "type mismatch: QuantumTicTacToe expects QuantumTicTacToeMoveMeas, found I don't care what",
      );
      if let Some(meas) = self.try_measure(e,mo.first) {
        let mut po: Vec<Position> = vec!();
        for (pos,me,e) in meas {
          self.remove_ent(e);
          self.classic_marks[pos.get_cid() as usize] = Some(me);
          po.push(pos);
        }
        self.fplayer = self.player;
        self.do_meas = None;
        MoveData {
          jump: Some(po),
          next_player: self.player,
          result: self.check_win()
        }
      } else {
        panic!("wtf?");
      }
    } else {
      let mo = m.downcast_ref::<QuantumTicTacToeMoveEnt>().expect(
          "type mismatch: QuantumTicTacToe expects QuantumTicTacToeMoveEnt, found I don't care what",
      );
      assert!(self.is_valid_move(Rc::clone(&m)),"invalid move!");
      if self.player.get_uuid() == self.fplayer.get_uuid() {
        self.subscript += 1;
      }
      let ent = Entanglement{p1: mo.p1, p2: mo.p2, player: self.player, subscript: self.subscript};
      self.ents.push(ent);
      if let Some(meas) = self.try_measure(ent,true) {
        self.do_meas = Some(ent);
        self.set_player(self.player.next_player());
        MoveData {
          jump: None,
          next_player: self.player,
          result: self.check_win()
        }
      } else {
        self.set_player(self.player.next_player());
        MoveData {
          jump: None,
          next_player: self.player,
          result: self.check_win()
        }
      }
    }
  }
  fn get_valid_moves(&self) -> Vec<Rc<dyn Any>> {
    let mut v: Vec<Rc<dyn Any>> = Vec::with_capacity(2);
    if let Some(e) = self.do_meas {
      v.push(Rc::new(QuantumTicTacToeMoveMeas{first: true}));
      v.push(Rc::new(QuantumTicTacToeMoveMeas{first: false}));
    } else {
      for i1 in 0..8 {
        for i2 in (i1+1)..9 {
          let m: Rc<dyn Any> = Rc::new(QuantumTicTacToeMoveEnt{p1: Position::from_cid(i1 as u8), p2: Position::from_cid(i2 as u8)});
          if self.is_valid_move(Rc::clone(&m)) {
            v.push(m);
          }
        }
      }
    }
    v
  }
  fn is_valid_move(&self, m: Rc<dyn Any>) -> bool {
    if let Some(e) = self.do_meas {
      let moo = m.downcast_ref::<QuantumTicTacToeMoveMeas>();
      if let Some(mo) = moo {
        true
      } else {false}
    } else {
      let moo = m.downcast_ref::<QuantumTicTacToeMoveEnt>();
      if let Some(mo) = moo {
        let ent = Entanglement{p1: mo.p1, p2: mo.p2, player: self.player, subscript: self.subscript};
        for e in self.ents.iter() {
          if (((ent.p1==e.p1) && (ent.p2==e.p2)) || ((ent.p1==e.p2) && (ent.p2==e.p1))) {
            return false;
          }
        }
        !(self.get_classic_mark(ent.p1).is_some() || self.get_classic_mark(ent.p2).is_some())
      } else {false}
    }
  }
  fn set_player(&mut self, player: P) {
    self.player = player;
  }
  fn get_player(&self) -> P {
    self.player
  }
}

impl<P: Player> QuantumTicTacToe<P> {
  pub fn new(player: P) -> Self {
    QuantumTicTacToe{
      classic_marks: [None;9],
      ents: vec!(),
      player,
      fplayer: player,
      subscript: 0,
      do_meas: None
    }
  }
  fn remove_ent(&mut self, ent: Entanglement<P>) {
    for (i,e) in self.ents.iter().enumerate() {
      if *e == ent {
        self.ents.remove(i);
        break;
      }
    }
  }
  pub fn get_classic_mark(&self, pos: Position) -> Option<Measurement<P>> {
    self.classic_marks[pos.get_cid() as usize]
  }
  pub fn get_ents(&self) -> &Vec<Entanglement<P>> {
    &self.ents
  }
  pub fn get_ents_in_cell(&self,pos:Position) -> Vec<Entanglement<P>> {
    let mut cells = vec!();
    for e in self.ents.iter() {
      if e.p1 == pos || e.p2 == pos {
        cells.push(*e)
      }
    }
    cells
  }
  fn try_measure(&self,m: Entanglement<P>,first: bool) -> Option<Vec<(Position,Measurement<P>,Entanglement<P>)>> {
    let mut res: Vec<(Position,Measurement<P>,Entanglement<P>)> = vec!();
    let mut opr: Vec<(Position,Measurement<P>,Entanglement<P>)> = vec!();
    let mut visited: Vec<Position> = vec!();
    {
      let (p,me) = m.measure_n(first);
      opr.push((p,me,m));
    }
    loop {
      let r = opr.pop();
      if let Some(r) = r {
        res.push(r);
        let (pos,meas,e) = r;
        if !visited.contains(&pos) {
          let es = self.get_ents_in_cell(pos);
          for c in es {
            if (c != e) {;
              let (p,me) = c.measure_push(pos);
              opr.push((p,me,c));
            }
          }
          visited.push(pos);
        }
      } else {break;}
    }
    let (apos,_) = m.measure_n(!first);
    let mut c = None;
    for (pos,_,_) in res.iter() {
      if apos == *pos {
        c = Some(res);
        break;
      }
    }
    c
  }

    fn check_win(&self) -> GameResult<P> {
        let mut rows: Vec<(P,u8)> = vec!();
        for i in 0..3 {
            let x = i as u8;
            // Check diagonals
            if x < 2 {
                if let Some(m1) = self.get_classic_mark(Position::from_xy(1, 1))
                {
                    match x {
                        0 => {
                            if let Some(m2) = self.get_classic_mark(Position::from_xy(0, 0))
                            {
                                if let Some(m3) = self.get_classic_mark(Position::from_xy(2, 2))
                                {
                                  let b = 
                                    m1 == m2
                                        && m2 == m3;
                                  if b {
                                    rows.push((m1.player,max(m1.subscript,max(m2.subscript,m3.subscript))));
                                  }
                                }
                            }
                        }
                        1 => {
                            if let Some(m2) = self.get_classic_mark(Position::from_xy(2, 0))
                            {
                                if let Some(m3) = self.get_classic_mark(Position::from_xy(0, 2))
                                {
                                  let b = 
                                    m1 == m2
                                        && m2 == m3;
                                  if b {
                                    rows.push((m1.player,max(m1.subscript,max(m2.subscript,m3.subscript))));
                                  }
                                }
                            }
                        }
                        _ => (),
                    }
                };
            };

            //Check vertical line
            if let Some(m1) = self.get_classic_mark(Position::from_xy(x, 1)) {
                if let Some(m2) = self.get_classic_mark(Position::from_xy(x, 0))
                {
                    if let Some(m3) = self.get_classic_mark(Position::from_xy(x, 2))
                    {
                                  let b = 
                                    m1 == m2
                                        && m2 == m3;
                                  if b {
                                    rows.push((m1.player,max(m1.subscript,max(m2.subscript,m3.subscript))));
                                  }
                    }
                }
            };

            //Check horizontal line
            if let Some(m1) = self.get_classic_mark(Position::from_xy(1, x)) {
                if let Some(m2) = self.get_classic_mark(Position::from_xy(0, x))
                {
                    if let Some(m3) = self.get_classic_mark(Position::from_xy(2, x))
                    {
                                  let b = 
                                    m1 == m2
                                        && m2 == m3;
                                  if b {
                                    rows.push((m1.player,max(m1.subscript,max(m2.subscript,m3.subscript))));
                                  }
                    }
                }
            };
        }
        if rows.len() > 0 {
          let mut irows = rows.iter();
          let mut minrow = irows.next().unwrap();
          for row in irows {
            if row.1 < minrow.1 {
              minrow = row
            }
          }
          let mut c = 0;
          let mut players: Vec<String> = vec!();
          for row in rows.iter() {
            if (minrow.1 == row.1) && (!players.contains(&minrow.0.get_uuid())) {
              c=c+1;
              players.push(minrow.0.get_uuid())
            }
          }
          if c > 1 {
            return GameResult::Draw;
          } else {
            return GameResult::Won(minrow.0);
          }
        }
        if self.get_valid_moves().len() == 0 {
          GameResult::Draw
        } else {
          GameResult::Incomplete
        }
    }
}