mod game_logic;
mod rand;
use game_logic::*;
use wasm_bindgen::{prelude::*, JsCast};

const TPS:f32 = 12.0; //game board is ticked (snake moves) on every tick
const CANV_W:u32 = 1200;
const CANV_H:u32 = (CANV_W/3)*2;     //enforces 3:2 aspect ratio for window
const CELL_W:u32 = CANV_W/GRID_W as u32;
const CELL_H:u32 = CANV_H/GRID_H as u32;
const SFX_VOL:f64 = 0.75;


// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut GAME_STATE:GameState = GameState::init();

static mut PAGE_ELEMS:core::mem::MaybeUninit::<PageElements> =
  core::mem::MaybeUninit::<PageElements>::uninit();

// macro_rules! console_log {
//   ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

// #[wasm_bindgen]
// extern "C" {
//   #[wasm_bindgen(js_namespace = console)]
//   fn log(s: &str);
// }

enum BkColour {Dark,Light}

struct PageElements {
  canvas:web_sys::HtmlCanvasElement,
  context:web_sys::CanvasRenderingContext2d,
  score:web_sys::HtmlElement,
  high_score:web_sys::HtmlElement,
  body:web_sys::HtmlBodyElement,
  sound_effects:SoundEffectElements
}

struct SoundEffectElements {
  apple:web_sys::HtmlMediaElement,
  grace:web_sys::HtmlMediaElement,
  game_over:web_sys::HtmlMediaElement
}

struct GameState {
  board:Board,
  high_score:[u32;2],
  use_alt_high_score:usize, // usize to index into high_score,
  is_game_over:bool,        // acts as bool otherwise
  is_game_paused:bool,
  should_build_walls:bool,
  should_mute_sfx:bool,
}

impl GameState {
  const fn init() -> Self {
    GameState {
      board: Board::new(),
      high_score: [0;2],
      use_alt_high_score: 0,
      is_game_over: false,
      is_game_paused: true,
      should_build_walls: true,
      should_mute_sfx: false,
    }
  }
  fn reset_game(&mut self) {
    unsafe{PAGE_ELEMS.assume_init_ref().change_background(BkColour::Dark)};
    let mut board = Board::new();
    if self.should_build_walls {
      board.generate_walls();
      self.use_alt_high_score = 1;
    } else {self.use_alt_high_score = 0;}
    board.generate_snake().expect_throw("Failed to generate Snake");
    draw_board(&board,unsafe{&(*PAGE_ELEMS.assume_init_ref()).context});
    *self = GameState{
      board,
      is_game_over:false,
      is_game_paused: true,
      ..*self
    };
    update_score_display(
      self.board.query_score(),
      self.high_score[self.use_alt_high_score]
    );
  }
}

impl PageElements {
  fn init() -> Self {
    let document = web_sys::window().unwrap_throw()
    .document().unwrap_throw();
    let canvas = document
      .get_element_by_id("canvas").unwrap_throw()
      .dyn_into::<web_sys::HtmlCanvasElement>().unwrap_throw();
    let context =  canvas
      .get_context("2d").unwrap_throw().unwrap_throw()
      .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap_throw();
    let score = document
      .get_element_by_id("score").unwrap_throw()
      .dyn_into::<web_sys::HtmlElement>().unwrap_throw();
    let high_score = document
      .get_element_by_id("highscore").unwrap_throw()
      .dyn_into::<web_sys::HtmlElement>().unwrap_throw();
    let body = document.body().unwrap_throw()
    .dyn_into::<web_sys::HtmlBodyElement>().unwrap_throw();
    let sound_effects = SoundEffectElements::init(document);
    PageElements {canvas,context,score,high_score,body,sound_effects}
  }
  fn change_background(&self,colour:BkColour) {
    match colour {
      BkColour::Dark  =>{
        self.body.set_background("./assets/backgrounds/dark.png")
      },
      BkColour::Light =>{
        self.body.set_background("./assets/backgrounds/light.png")
      }
    }
  }
}

impl SoundEffectElements {
  fn init(document:web_sys::Document) -> Self {
    let sfx_block = document.get_element_by_id("sfxblock").unwrap_throw();
    let (apple,grace,game_over) = (
      document
        .create_element("audio").unwrap_throw()
        .dyn_into::<web_sys::HtmlMediaElement>().unwrap_throw(),
      document
        .create_element("audio").unwrap_throw()
        .dyn_into::<web_sys::HtmlMediaElement>().unwrap_throw(),
      document
        .create_element("audio").unwrap_throw()
        .dyn_into::<web_sys::HtmlMediaElement>().unwrap_throw()
    );
    let mut ids = ["apple", "grace", "gameover"].into_iter();
    let mut srcs = [
      "./assets/sounds/apple.wav",
      "./assets/sounds/grace.wav",
      "./assets/sounds/gameover.wav",
    ].into_iter();
    for sfx in [&apple, &grace, &game_over].into_iter() {
      sfx.set_id(ids.next().unwrap_throw());
      sfx.set_src(srcs.next().unwrap_throw());
      sfx.set_preload("auto");
      sfx.set_volume(SFX_VOL);
      sfx.load();
      sfx_block.append_child(sfx).unwrap_throw();
    }
    SoundEffectElements{apple,grace,game_over}
  }
}

#[wasm_bindgen(js_name=queryTPS)]
pub fn query_tps() -> f32 {
  TPS
}

fn update_score_display(score:u32, high_score:u32) {
  let pe = unsafe{PAGE_ELEMS.assume_init_ref()};
  pe.score.set_inner_html(format!("Score: {score}").as_str());
  pe.high_score.set_inner_html(format!("High Score: {high_score}").as_str());
}


#[wasm_bindgen(js_name = rustGameLoop)]
pub fn rust_gameloop() {
  let (gs,pe,sfx) = unsafe{(
    &mut GAME_STATE,
    PAGE_ELEMS.assume_init_ref(),
    &(*PAGE_ELEMS.assume_init_ref()).sound_effects
  )};
  if gs.is_game_paused || gs.is_game_over {return;}
  match gs.board.do_game_tick() {
    GameEvent::GameOver => {
      gs.is_game_over = true;
      pe.change_background(BkColour::Light);
      if !gs.should_mute_sfx {let _ = sfx.game_over.play().unwrap_throw();}
    }
    GameEvent::AppleCollected => {
        if gs.board.query_score() > gs.high_score[gs.use_alt_high_score] {
          gs.high_score[gs.use_alt_high_score] = gs.board.query_score();
        }
        update_score_display(
          gs.board.query_score(),
          gs.high_score[gs.use_alt_high_score]
        );
        if !gs.should_mute_sfx {let _ = sfx.apple.play().unwrap_throw();}
      }
    GameEvent::GraceTick => {
      if !gs.should_mute_sfx {let _ = sfx.grace.play().unwrap_throw();}
    }
    GameEvent::GeneralMovement => {}
  }
  draw_board(&gs.board,&pe.context);
}

#[wasm_bindgen(js_name = sendKeypress)]
pub fn recieve_keypress(key:u8) {
  let gs = unsafe {&mut GAME_STATE};
  match key {
    82 => gs.reset_game(),                                // R
    87 => gs.should_build_walls = !gs.should_build_walls, // W
    77 => gs.should_mute_sfx = !gs.should_mute_sfx,       // M
    32 if gs.is_game_paused => {                          // Space
      gs.is_game_paused = false;
      gs.board.generate_apple().expect_throw("Failed to generate Apple");
    }
    _  => {}
  }
  if !gs.is_game_over && !gs.is_game_paused {
    match key {
      38 => gs.board.change_facing_direction(Direction::Up), // Up Arrow
      40 => gs.board.change_facing_direction(Direction::Dn), // Down Arrow
      37 => gs.board.change_facing_direction(Direction::Lf), // Left Arrow
      39 => gs.board.change_facing_direction(Direction::Rt), // Right Arrow
      _  => {}
    }
  }
}

fn draw_board(board:&Board, context:&web_sys::CanvasRenderingContext2d) {
  fn inner_offset(cell:GridCell) -> (f64,f64) {
    match cell {
      GridCell::Snake(_,Direction::Dn) => (0.5  ,0.625),
      GridCell::Snake(_,Direction::Up) => (0.5  ,0.375),
      GridCell::Snake(_,Direction::Rt) => (0.625,0.5  ),
      GridCell::Snake(_,Direction::Lf) => (0.375,0.5  ),
      _                                => (0.5  ,0.5  )
    }
  }
  fn colour(cell:GridCell,is_head:bool) -> [[JsValue;2];2] {
    match cell {
        GridCell::Nothing => [
          [JsValue::from_str("#1d2021"),JsValue::from_str("#282828")],
          [JsValue::from_str("#282828"),JsValue::from_str("#32302f")]
        ],
        GridCell::Snake(_, _) if is_head => [
          [JsValue::from_str("#689d6a"),JsValue::from_str("#8ec07c")],
          [JsValue::from_str("#427b58"),JsValue::from_str("#689d6a")]
        ],
        GridCell::Snake(_, _) => [
          [JsValue::from_str("#98971a"),JsValue::from_str("#b6b926")],
          [JsValue::from_str("#79740e"),JsValue::from_str("#98971a")]
        ],
        GridCell::Apple => [
          [JsValue::from_str("#cc241d"),JsValue::from_str("#f74833")],
          [JsValue::from_str("#9d0006"),JsValue::from_str("#cc241d")]
        ],
        GridCell::Wall => [
          [JsValue::from_str("#3c3836"),JsValue::from_str("#50493c")],
          [JsValue::from_str("#d5c6a1"),JsValue::from_str("#ebdbb2")]
        ]
    }
  }
  let inner_scale:f64 = 0.375;
  let inner_scale_sqrt = inner_scale.sqrt();
  let grace_pallet:usize = if board.query_grace() {1} else {0};

  // create a iterator of all drawn elements
  let cells = (0..GRID_H).flat_map(|y|(0..GRID_W).map(move|x|{
    (board.peek(x,y),(x,y))
  }));
  let snake_body = cells.clone()
  .filter(|cell|match cell {(GridCell::Snake(..),..)=>true, _=>false});
  let apples = cells.clone()
  .filter(|cell|match cell {(GridCell::Apple,..)=>true, _=>false});
  let walls = cells.clone()
  .filter(|cell|match cell {(GridCell::Wall,..)=>true, _=>false});
  let nothings = cells
  .filter(|cell|match cell {(GridCell::Nothing,..)=>true, _=>false});
  let drawn_elements = snake_body.chain(apples).chain(walls).chain(nothings);

  // Draw Background (Nothing base colour)
  context.set_fill_style(&colour(GridCell::Nothing,false)[grace_pallet][0]);
  context.fill_rect(0f64,0f64,CANV_W as f64,CANV_H as f64);

  { // Draw all cells
    let mut prev_type = GridCell::Nothing; //Garenteed to overwrite instantly
    for elem in drawn_elements.clone() {
      let (gen_curr_type,(x,y)) = match elem {
        (GridCell::Snake(..),(x,y))=> (GridCell::Snake(0,Direction::Rt),(x,y)),
        (c,(x,y))                   => (c,(x,y))
      };
      let curr_type = match elem {(c,_) => c};
      if curr_type == GridCell::Nothing {break}; //Don't redraw background.
      if gen_curr_type != prev_type {
        context.set_fill_style(&colour(gen_curr_type,false)[grace_pallet][0]);
        prev_type = curr_type;
      }
      context.fill_rect(
        (x as u32 * CELL_W) as f64,
        (y as u32 * CELL_H) as f64,
        CELL_W as f64,
        CELL_H as f64
      );
    }
    prev_type = GridCell::Nothing;
    for elem in drawn_elements {
      let (gen_curr_type,(x,y)) = match elem {
          (GridCell::Snake(..),(x,y))=>(GridCell::Snake(0,Direction::Rt),(x,y)),
          (c,(x,y))                  =>(c,(x,y))
      };
      let curr_type = match elem {(c,_) => c};
      if gen_curr_type != prev_type {
        context.set_fill_style(&colour(gen_curr_type,false)[grace_pallet][1]);
        prev_type = curr_type;
      }
      context.fill_rect(
        (x as u32 * CELL_W) as f64 +
          inner_offset(curr_type).0 * CELL_W as f64 * (1f64 - inner_scale_sqrt),
        (y as u32 * CELL_H) as f64 +
          inner_offset(curr_type).1 * CELL_H as f64 * (1f64 - inner_scale_sqrt),
        CELL_W as f64 * inner_scale_sqrt,
        CELL_H as f64 * inner_scale_sqrt
      );
    }
  }

  {// Draw over the head of the snake with the snake head colours.
    let (x,y) = board.query_head_location()
      .expect_throw("Failed to locate head");
    context.set_fill_style(
      &colour(GridCell::Snake(0,Direction::Rt),true)[grace_pallet][0]
    );
    context.fill_rect(
      (x as u32 * CELL_W) as f64,
      (y as u32 * CELL_H) as f64,
      CELL_W as f64,
      CELL_H as f64
    );
    context.set_fill_style(
      &colour(GridCell::Snake(0,Direction::Rt),true)[grace_pallet][1]
    );
    context.fill_rect(
      (x as u32 * CELL_W) as f64 +
        inner_offset(board.peek(x,y)).0*CELL_W as f64*(1f64 - inner_scale_sqrt),
      (y as u32 * CELL_H) as f64 +
        inner_offset(board.peek(x,y)).1*CELL_H as f64*(1f64 - inner_scale_sqrt),
      CELL_W as f64 * inner_scale_sqrt,
      CELL_H as f64 * inner_scale_sqrt
    );
  }

}

#[wasm_bindgen(start)]
pub fn main() {
  unsafe {PAGE_ELEMS.write(PageElements::init());}
  let (pe,gs) = unsafe{(PAGE_ELEMS.assume_init_ref(),&mut GAME_STATE)};
  pe.canvas.set_attribute("Width", CANV_W.to_string().as_str()).unwrap_throw();
  pe.canvas.set_attribute("Height", CANV_H.to_string().as_str()).unwrap_throw();
  pe.canvas.set_attribute("tabindex","1").unwrap_throw();
  gs.reset_game();
  draw_board(&gs.board, &pe.context);
}