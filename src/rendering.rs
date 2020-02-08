use termion::color;
use termion::clear;
use termion::cursor;
use crate::pl_xo::pl_XO;
use crate::terminal_render::*;
use crate::tic_tac_toe::*;
use std::any::Any;
use pl_XO::*;
use std::cmp::{min,max};
use std::ops::Range;
use std::rc::Rc;

//═║╬

struct DontRenderSelection;
struct ShadowedRender;

impl<T: GenericGame<P> + TerminalRender,P: Player + TerminalRender> TerminalRender for SuperTicTacToe<T,P> {
  fn get_size(&self) -> (usize,usize) {
    let (mut mw, mut mh) = (0,0);
    for i in 0..9 {
      let pos = Position::from_cid(i as u8);
      let state = self.get_cell_ref(pos);
      let (w,h) = state.get_size();
      mw = max(w,mw);
      mh = max(h,mh);
    }
    ((mw+2)*3+2,(mh)*3+2)
    // +8, +5
  }
  fn render(&self, term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let (mw,mh) = self.get_size();
    let (tw,th) = term.get_size();
    assert!((mw <= tw) && (mh <= th),"Canvas not big enough");
    let (pcw,pch) = ((tw-2)/3-2,(th-2)/3);
    let (cw,ch) = (pcw+2,pch);
    let (w,h) = ((cw)*3+2,(ch)*3+2);
    term.clip((tw-w)/2,(th-h)/2,w,h);
    let mut shadowed = false;
    
    for rv in args.iter() {
      if let Some(_) = rv.downcast_ref::<ShadowedRender>() {
        shadowed = true;
      }
    }
    
    let cl = format!("{0}║{0}║{0}\r\n",(" ").repeat(cw)).repeat(ch);
    {
      let (fg,bg) = term.get_color();
      term.set_fg(if shadowed { ColorEnum::LightBlack } else { ColorEnum::White });
      term.set(0,0,
        format!("{0}{1}{0}",
          cl,
          format!("{0}{1}{0}",
            format!("{0}╬{0}╬{0}\r\n",("═").repeat(cw)),
            cl
          )
        ).as_str()
      );
      term.set_color(fg,bg);
    };
    
    let mut allgood = true;
    
    for i in 0..9 {
      let pos = Position::from_cid(i as u8);
      let state = self.get_cell_ref(pos);
      if let SubgameState::Playing(state) = state {
        if !self.is_good_cell(pos) {
          allgood = false;
        }
      }
    }
    
    let mut rsel = false;
    
    for i in 0..9 {
      let pos = Position::from_cid(i as u8);
      let state = self.get_cell_ref(pos);
      let (ox,oy) = ((pos.get_x() as usize)*(cw+1)+1,(pos.get_y() as usize)*(ch+1));
      let (fg,bg) = term.get_color();
      term.clip(ox,oy,pcw,pch);
      let mut arg: Vec<Rc<dyn Any>> = vec!();
      if shadowed {arg.push(Rc::new(ShadowedRender))};
      let rvs = state.render(term,arg);
      let mut renders = true;
      for rv in rvs.iter() {
        if let Some(_) = rv.downcast_ref::<DontRenderSelection>() {
          renders = false;
        }
      }
      term.unclip();
      if self.is_good_cell(pos) && !allgood && renders {
        rsel = true;
        term.set_fg(ColorEnum::LightMagenta);
        term.set((ox-1) as isize,(oy) as isize,("█\r\n").repeat(pch).as_str());
        term.set((ox+pcw) as isize,(oy) as isize,("█\r\n").repeat(pch).as_str());
      }
      term.set_color(fg,bg);
    }
    
    term.unclip();
    let mut vec: Vec<Rc<dyn Any>> = vec!();
    if rsel {
      vec.push(Rc::new(DontRenderSelection));
    }
    vec
  }
}

fn mtp(x: (usize,usize),y: (usize,usize)) -> (usize,usize) {
  let ((x1,y1),(x2,y2)) = (x,y);
  (max(x1,x2),max(y1,y2))
}

impl<T: GenericGame<P> + TerminalRender, P: Player + TerminalRender> TerminalRender for SubgameState<T,P> {
  fn get_size(&self) -> (usize,usize) {
    match self {
      SubgameState::Won(player,game) => mtp(player.get_size(),game.get_size()),
      SubgameState::Draw(game) => game.get_size(),
      SubgameState::Playing(game) => game.get_size(),
      _ => panic!("passing None to get_size"),
    }
  }
  fn render(&self, term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    match self {
      SubgameState::Won(player,game) => {
        let mut gargs: Vec<Rc<dyn Any>> = vec!(Rc::new(ShadowedRender));
        gargs.extend_from_slice(args.as_slice());
        let mut rargs = game.render(term,gargs.clone());
        rargs.extend_from_slice(player.render(term,args).as_slice());
        rargs
      },
      SubgameState::Draw(game) => {
        let mut gargs: Vec<Rc<dyn Any>> = vec!(Rc::new(ShadowedRender));
        gargs.extend_from_slice(args.as_slice());
        game.render(term,gargs)
      },
      SubgameState::Playing(game) => {
        game.render(term,args)
      },
      _ => panic!("passing None to render"),
    }
  }
}

impl<P: Player + TerminalRender> TerminalRender for Entanglement<P>
{
  fn get_size(&self) -> (usize,usize) {
    let (w,h) = self.get_player().get_size();
    (w+1,h+1)
  }
  fn render(&self, term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let (w,h) = term.get_size();
    let (ew,eh) = self.get_size();
    assert!(w >= ew && h >= eh, "Canvas not big enough");
    let mut shadowed = false;
    
    for rv in args.iter() {
      if let Some(_) = rv.downcast_ref::<ShadowedRender>() {
        shadowed = true;
      }
    }

    let (fg,bg) = term.get_color();
    term.clip(0,0,w-1,h-1);
    self.get_player().render(term,args);
    term.unclip();
    let ss = self.get_subscript().to_string();
    term.set_fg(if shadowed {ColorEnum::LightBlack} else {ColorEnum::White});
    term.set((w-ss.len()) as isize,(h-1) as isize, ss.as_str());
    term.set_color(fg,bg);
    vec!()
  }
}

impl<P: Player + TerminalRender> TerminalRender for Measurement<P>
{
  fn get_size(&self) -> (usize,usize) {
    let (w,h) = self.get_player().get_size();
    (w+1,h+1)
  }
  fn render(&self, term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let (w,h) = term.get_size();
    let (ew,eh) = self.get_size();
    assert!(w >= ew && h >= eh, "Canvas not big enough");
    let mut shadowed = false;
    
    for rv in args.iter() {
      if let Some(_) = rv.downcast_ref::<ShadowedRender>() {
        shadowed = true;
      }
    }

    let (fg,bg) = term.get_color();
    term.clip(0,0,w-1,h-1);
    self.get_player().render(term,args);
    term.unclip();
    let ss = self.get_subscript().to_string();
    term.set_fg(if shadowed {ColorEnum::LightBlack} else {ColorEnum::White});
    term.set((w-ss.len()) as isize,(h-1) as isize, ss.as_str());
    term.set_color(fg,bg);
    vec!()
  }
}

impl<P: Player + TerminalRender> TerminalRender for QuantumTicTacToe<P> {
  fn get_size(&self) -> (usize,usize) {
    let (mut mw, mut mh) = (0,0);
    for i in 0..9 {
      let pos = Position::from_cid(i as u8);
      let mark = self.get_classic_mark(pos);
      if let Some(mark) = mark {
        let (w,h) = mark.get_size();
        mw = max(w,mw);
        mh = max(h,mh);
      } else {
        let mark = Measurement{player: self.get_player(),subscript: 0};
        let (w,h) = mark.get_size();
        mw = max(w,mw);
        mh = max(h,mh);
      }
      let mut c = false;
      for e in self.get_ents_in_cell(pos) {
        c = true;
        let (w,h) = e.get_size();
        mw = max(mw,w*3+2);
        mh = max(mh,h*3+2);
      }
      if !c {
        let e = Entanglement{player: self.get_player(), p1: Position::from_cid(0),p2: Position::from_cid(1), subscript: 0};
        let (w,h) = e.get_size();
        mw = max(mw,w*3+2);
        mh = max(mh,h*3+2);
      }
    }
    (mw*3+2,mh*3+2)
    // +8, +5
  }
  fn render(&self, term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let (mw,mh) = self.get_size();
    let (tw,th) = term.get_size();
    assert!((mw <= tw) && (mh <= th),"Canvas not big enough");
    let (pcw,pch) = ((tw-2)/3,(th-2)/3);
    let (cw,ch) = (pcw,pch);
    let (w,h) = ((cw)*3+2,(ch)*3+2);
    term.clip((tw-w)/2,(th-h)/2,w,h);
    let mut shadowed = false;
    
    for rv in args.iter() {
      if let Some(_) = rv.downcast_ref::<ShadowedRender>() {
        shadowed = true;
      }
    }
    
    let cl = format!("{0}║{0}║{0}\r\n",(" ").repeat(cw)).repeat(ch);
    {
      let (fg,bg) = term.get_color();
      term.set_fg(if shadowed { ColorEnum::Yellow } else { ColorEnum::LightYellow });
      term.set(0,0,
        format!("{0}{1}{0}",
          cl,
          format!("{0}{1}{0}",
            format!("{0}╬{0}╬{0}\r\n",("═").repeat(cw)),
            cl
          )
        ).as_str()
      );
      term.set_color(fg,bg);
    };
    for i in 0..9 {
      let pos = Position::from_cid(i as u8);
      let state = self.get_classic_mark(pos);
      let (ox,oy) = ((pos.get_x() as usize)*(cw+1),(pos.get_y() as usize)*(ch+1));
      let (fg,bg) = term.get_color();
      term.clip(ox,oy,pcw,pch);
      
      if let Some(mark) = state {
        let mut arg: Vec<Rc<dyn Any>> = vec!();
        if shadowed {arg.push(Rc::new(ShadowedRender))};
        mark.render(term,arg);
      }
      
      {
        let (tw,th) = term.get_size();
        let (pcw,pch) = ((tw-2)/3,(th-2)/3);
        let (cw,ch) = (pcw,pch);
        let (w,h) = ((cw)*3+2,(ch)*3+2);
        for (i,e) in self.get_ents_in_cell(pos).into_iter().enumerate() {
          let pos = Position::from_cid(i as u8);
          let (ox,oy) = ((pos.get_x() as usize)*(cw+1),(pos.get_y() as usize)*(ch+1));
          term.clip(ox,oy,pcw,pch);
          let mut arg: Vec<Rc<dyn Any>> = vec!();
          if shadowed {arg.push(Rc::new(ShadowedRender))};
          e.render(term,arg);
          term.unclip();
        }
      }
      
      term.unclip();
      term.set_color(fg,bg);
    }
    
    term.unclip();
    vec!()
  }
}

impl<P: Player> TerminalRender for DummyGame<P> {}

impl<T:GenericGame<P> + TerminalRender,P: Player + TerminalRender> TerminalRender for SuperDummyGame<T,P> {
  fn get_size(&self) -> (usize,usize) {
    self.get_game_ref().get_size()
  }
  fn render(&self, term: &mut TerminalCanvas,args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let (w,h) = term.get_size();
    term.set(0,0,
      format!("{}\r\n",(" ").repeat(w)).repeat(h).as_str()
    );
    self.get_game_ref().render(term,args)
  }
}

impl TerminalRender for pl_XO {
  fn get_size(&self) -> (usize,usize) {
    (1,1)
  }
  fn render(&self,term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let mut shadowed = false;
    
    for rv in args.iter() {
      if let Some(_) = rv.downcast_ref::<ShadowedRender>() {
        shadowed = true;
      }
    }
    let (w,h) = term.get_size();
    let w = w as isize;
    let h = h as isize;

    let m = min(w,h);
    let (fg,bg) = term.get_color();
    term.clip(((w-m)/2) as usize,((h-m)/2) as usize,m as usize,m as usize);
    match self {
      X => {
        term.set_fg(if shadowed { ColorEnum::LightBlack } else { ColorEnum::LightRed });
        match m {
          1 => {
            term.set(0,0,"X");
          },
          0 => {
            panic!("Canvas not big enough");
          },
          _ => {
            match m%2 {
              1 => {
                //╳╱╲
                let center = (m-1)/2;
                term.set(center,center,"╳");
                for x in (Range::<isize>{start: 0, end: center}) {
                  let l: isize = m-1-x;
                  term.set(x,x,"╲");
                  term.set(l,x,"╱");
                  term.set(x,l,"╱");
                  term.set(l,l,"╲");
                }
              },
              _ => {
                let center = m/2;
                for x in (Range::<isize>{start: 0, end: center}) {
                  let l: isize = m-1-x;
                  term.set(x,x,"╲");
                  term.set(l,x,"╱");
                  term.set(x,l,"╱");
                  term.set(l,l,"╲");
                }
              },
            }
          },
        }
      },
      O => {
        term.set_fg(if shadowed { ColorEnum::LightBlack } else { ColorEnum::LightBlue });
        match m {
          1 => {
            term.set(0,0,"O");
          },
          2 => {
            //╭╮╯╰─│
            term.set(0,0,"╭╮\r\n╰╯");
          }
          0 => {
            panic!("Canvas not big enough");
          }
          _ => {
            let l = (m as usize)-2;
            term.set(0,0,format!("╭\r\n{}╰", ("│\r\n").repeat(l)).as_str());
            term.set((m as isize)-1,0,format!("╮\r\n{}╯", ("│\r\n").repeat(l)).as_str());
            term.set(1,0,("─").repeat(l).as_str());
            term.set(1,(m as isize)-1,("─").repeat(l).as_str());
          }
        }
      }
      C => {
        term.set_fg(if shadowed { ColorEnum::LightBlack } else { ColorEnum::LightGreen });
        match m {
          1 => {
            term.set(0,0,"C");
          },
          2 => {
            //╭╮╯╰─│
            term.set(0,0,"╭─\r\n╰─");
          }
          0 => {
            panic!("Canvas not big enough");
          }
          _ => {
            let l = (m as usize)-2;
            term.set(0,0,format!("╭\r\n{}╰", ("│\r\n").repeat(l)).as_str());
            term.set(1,0,("─").repeat(l+1).as_str());
            term.set(1,(m as isize)-1,("─").repeat(l+1).as_str());
          }
        }
      }
    };
    term.unclip();
    term.set_color(fg,bg);
    vec!()
  }
}
