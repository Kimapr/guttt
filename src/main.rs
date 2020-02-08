pub extern crate libguttt;
pub use libguttt as tic_tac_toe;

mod terminal_render;
mod pl_xo;
mod rendering;

use tic_tac_toe::*;
use terminal_render::*;
use pl_xo::pl_XO;
use pl_XO::*;

use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc,Mutex};
use std::mem::drop;
use std::cmp::min;
use std::thread;
use std::time::{Instant,Duration};
use std::io::{Write, stdout, stdin};
use std::any::Any;
use std::fmt::Debug;
use std::cmp::max;
use std::sync::mpsc::channel;
use std::mem::MaybeUninit;

extern crate termion;
extern crate unicode_segmentation;
extern crate rand;
use rand::seq::IteratorRandom;
use rand::Rng;
use termion::color;
use termion::clear;
use termion::cursor;
use termion::screen::{ToAlternateScreen,ToMainScreen,AlternateScreen};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;
struct MoveScore(f64,Rc<dyn Any>);

#[derive(Clone)]
enum BotMode {
  Time(u64,Option<Box<BotMode>>),
  Count(u64,Option<Box<BotMode>>)
}

fn best_move<T: GenericGame<P> + Clone,P: Player, R: rand::Rng>(game: T, player: P, rng: &mut R, dat: BotMode) -> Option<Rc<dyn Any>> {
  let vmoves = game.get_valid_moves();
  if vmoves.len() == 0 {
    return None;
  }
  let mut move_score = Vec::<MoveScore>::new();
  let rmc = vmoves.len();
  let mut maxtime: Option<Duration> = None;
  let mut maxcount: Option<u64> = None;
  let bot: Option<BotMode>;
  match dat.clone() {
    BotMode::Time(n,b) => {bot = if let Some(rb) = b {Some(*rb)} else {None};maxtime = Some(Duration::from_millis(n/(rmc as u64)));},
    BotMode::Count(n,b) => {bot = if let Some(rb) = b {Some(*rb)} else {None};maxcount = Some(n);}
  }
  for m in vmoves.iter() {
    let mut score: f64 = 0.0;
    let st = Instant::now();
    let mut count: u64 = 0;
    loop {
      let mut gres = GameResult::<P>::Incomplete;
      let mut game = game.clone();
      game.mov(Rc::clone(m));
      let mut movesn: f64 = 0.0;
      loop {
        let rmove: Option<Rc<dyn Any>>;
        match bot.clone() {
          Some(botmode) => {
            rmove = best_move(game.clone(),player,rng,botmode);
          },
          None => {
            let moves = game.get_valid_moves();
            rmove = if let Some(rm) = moves.iter().choose(rng) {
              Some(Rc::clone(rm))
            } else {
              None
            };
          }
        }
        if let Some(rm) = rmove {
          movesn += 1.0;
          gres = game.mov(Rc::clone(&rm)).result;
        } else {
          break;
        }
      }
      match gres {
        GameResult::Won(pl) => {
          if player.get_uuid() == pl.get_uuid() {
            score = score + 1.0;
          } else {
            score = score - 2.0;
          }
        },
        GameResult::Draw => {score -= 1.0},
        Incomplete => ()
      }
      count += 1;
      match &dat {
        BotMode::Time(_,_) => {
          if st.elapsed() >= maxtime.unwrap() {
            break;
          }
        },
        BotMode::Count(_,_) => {
          if count >= maxcount.unwrap() {
            break;
          }
        }
      }
    }
    move_score.push(MoveScore(score,Rc::clone(m)))
  }
  let mut biggest = std::f64::MIN;
  let mut bm: Rc<dyn Any> = Rc::new(true);
  for m in move_score.iter() {
    if m.0 > biggest {
      bm = Rc::clone(&m.1);
      biggest = m.0;
    }
  }
  Some(bm)
}

fn good_size(mw: usize,mh: usize) -> (usize,usize) {
  loop {
    let (w,h) = termion::terminal_size().expect("error!");
    if ((w as usize) >= mw) && ((h as usize) >= mh) {
      return (w as usize,h as usize);
    }
  }
}

const MAX_DT_IDLE: u64 = 1000/10;
const MAX_DT_SCROLL: u64 = 1000/1000;

fn main() {
  {
    type PanicFunc = Box<dyn Fn(&std::panic::PanicInfo) + Sync + Send + 'static>;
    static mut oldhook: MaybeUninit<PanicFunc> = MaybeUninit::uninit();
    unsafe{
      oldhook = MaybeUninit::new(std::panic::take_hook());
    }
    std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
      stdout().into_raw_mode().unwrap().suspend_raw_mode();
      println!("{}Oops! It seems that something BAD happened to the game...",ToMainScreen);
      unsafe{
        (*oldhook.as_ptr())(info);
      }
      std::process::abort();
    }));
  }
  let gamegen = || {
    SuperDummyGame::<
      SuperTicTacToe<
        //SuperTicTacToe<
          //SuperTicTacToe<DummyGame<pl_XO>,pl_XO>,
          QuantumTicTacToe<pl_XO>,
        //pl_XO>,
      pl_XO>,
    pl_XO>::create(X,Default::default())
  };
  let game_mutex: Arc<Mutex<_>> = Arc::new(Mutex::new(gamegen()));
  let gmut = Arc::clone(&game_mutex);
  let (tx,rx) = channel::<bool>();
  let gamer_thread = thread::spawn(move || {
    let mut rng = rand::thread_rng();
    loop {
      if let Some(msg) = rx.try_recv().ok() {
        break;
      }
      //println!("> lg");
      let mut game = gmut.lock().unwrap();
      //println!("> slg");
      if let SubgameState::Playing(_) = game.get_game_ref() {
        let mut gameclone = (&*game).clone();
        let player = gameclone.get_player();
        //println!("> lgd");
        drop(game);

         /*
        game = gmut.lock().unwrap();
        let moves = game.get_valid_moves();
        let rmove = moves.iter().choose(&mut rng);
        if let Some(rm) = rmove {
          game.mov(Rc::clone(rm));
        } else {
          break;
        }
        drop(game);
        thread::sleep(Duration::from_millis(100));
         */

        // /*
        if let Some(m) = best_move(gameclone,player,&mut rng,
          BotMode::Time(300,None)
        ) {
          //println!("> lg2");
          game = gmut.lock().unwrap();
          //println!("> slg2");
          if (game.is_valid_move(Rc::clone(&m))) {
            game.mov(m);
          }
          //println!("> lgd2");
          drop(game);
        }
        // */

      } else {
        //println!("> lgd");
        drop(game);
        thread::sleep(Duration::from_millis(10000));
        //println!("> lg3");
        game = gmut.lock().unwrap();
        //println!("> slg3");
        *game = gamegen();
      }
    }
  });
  let (mut w,mut h) = good_size(5,6);
  let (mut vw,mut vh) = (w-4,h-5);
  //println!("lg");
  let game = game_mutex.lock().unwrap();
  //println!("slg");
  let (mut rw,mut rh) = game.get_size();
  //println!("lgd");
  drop(game);
  rw = max(min(vw,vh*2),rw);
  rh = max(min(vw/2,vh),rh);
  let mut rs = max(rw,rh*2);
  rw = rs;
  rh = rs/2;
  
  let mut canv = TerminalCanvas::new(rw,rh,ColorEnum::Reset,ColorEnum::Reset);
  let mut term = TerminalCanvas::new(w,h,ColorEnum::Green,ColorEnum::Reset);
  let mut n = 0;
  let mut off: (isize,isize) = (0,0);
  let mut running = true;
  
  let mut screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
  write!(screen,"{}",cursor::Hide);
  let mut input = termion::async_stdin().keys();
  let mut fps = 0;
  let mut toff: (isize,isize) = (0,0);
  let id = ["-","\\","|","/"];
  while running {
    let fstart = Instant::now();
    //println!("lg2");
    let game = game_mutex.lock().unwrap();
    if (fstart.elapsed().as_millis() > 500) {
      panic!("Waiting too much");
    }
    //println!("slg2");
    let (w2,h2) = good_size(5,6);
    if ((w != w2) || (h != h2)) {
      term = TerminalCanvas::new(w2,h2,ColorEnum::Green,ColorEnum::Reset);
      w = w2;
      h = h2;
      vw = w-4;
      vh = h-5;
    }
    let (rw2,rh2) = game.get_size();
    if ((rw2 > rw) || (rw2 > rh2)) {
      rw = rw2;
      rh = rh2;
      rw = max(min(vw,vh*2),rw);
      rh = max(min(vw/2,vh),rh);
      rs = max(rw,rh*2);
      rw = rs;
      rh = rs/2;
      canv = TerminalCanvas::new(rw,rh,ColorEnum::Reset,ColorEnum::Reset);
    }
    game.render(&mut canv,vec!());
    //println!("lgd2");
    if let SubgameState::Playing(sgame) = game.get_game_ref() {
      let player = sgame.get_player();
      term.clip(1,h-1,1,1);
      let (fg,bg) = term.get_color();
      player.render(&mut term,vec!());
      term.set_color(fg,bg);
      term.unclip();
    }
    drop(game);
    let (mut ox,mut oy) = off;
    term.set_color(ColorEnum::Green,ColorEnum::Reset);
    term.set(1,1,format!("{}{}{}",
      format!("▄{}▄\r\n",("▄").repeat(vw)),
      format!("█{}█\r\n",(" ").repeat(vw)).repeat(vh),
      format!("▀{}▀\r\n",("▀").repeat(vw))
    ).as_str());
    term.clip(1,h-1,1,1);
    term.unclip();
    term.set(3,(h as isize)-1,format!("FPS: {} {}   ",fps,id[n]).as_str());
    term.clip(2,2,vw,vh);
    term.set_canv(-ox,-oy,&canv);
    term.unclip();
    {
      write!(screen,"{}{}", cursor::Goto(1,1),term);
      screen.flush().unwrap();
    };
    n=(n+1)%4;
    let (mut tox, mut toy) = toff;
    let ms = min(vw,vh) as isize;
    loop {
      match input.next() {
        Some(key) => match key.unwrap() {
          Key::Up => {toy = toy-(vh as isize)/2},
          Key::Down => {toy = toy+(vh as isize)/2},
          Key::Left => {tox = tox-(vw as isize)/2},
          Key::Right => {tox = tox+(vw as isize)/2},
          Key::Esc => {running = false;},
          _ => (),
        },
        None => {break;}
      }
    }
    tox = min((rw as isize)-(vw as isize),max(0,tox));
    toy = min((rh as isize)-(vh as isize),max(0,toy));
    toff = (tox,toy);
    let mut mdt = MAX_DT_IDLE;
    let dt = fstart.elapsed().as_millis();
    for _i in 0..max(1,((dt as usize)*min(vw,vh)*2/1000)) {
      for _i2 in 0..2 {
        if vw < rw {
          if ox < tox {
            ox += 1;
            mdt = MAX_DT_SCROLL;
          }
          if ox > tox {
            ox -= 1;
            mdt = MAX_DT_SCROLL;
          }
        } else {
          ox = -(((vw-rw)/2) as isize);
          tox = ox;
        }
      }
      if vh < rh {
        if oy < toy {
          oy += 1;
          mdt = MAX_DT_SCROLL;
        }
        if oy > toy {
          oy -= 1;
          mdt = MAX_DT_SCROLL;
        }
      } else {
        oy = -(((vh-rh)/2) as isize);
        toy = oy;
      }
    }
    off = (ox,oy);
    if (dt as u64) < mdt {
      thread::sleep(Duration::from_millis(mdt-(dt as u64)));
    }
    fps = 1000/(max(1,fstart.elapsed().as_millis()));
  }
  drop(screen);
  println!("{}Thanks for using Generic Super Tic-Tac-Toe by Kimapr", cursor::Show);
  tx.send(true);
}
