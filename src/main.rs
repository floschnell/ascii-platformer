extern crate termion;
use std::error::Error;
use std::io::{stdout, Read, Write};
use termion::raw::IntoRawMode;

fn main() {
  let stdout = stdout();
  let mut stdout = stdout.lock().into_raw_mode().unwrap();
  let mut stdin = termion::async_stdin().bytes();

  let (width, height) = match termion::terminal_size() {
    Ok((height, width)) => (height, width),
    Err(_) => panic!("Could not get terminal size!"),
  };
  let mut player = Player {
    x: 1.0,
    y: 1.0,
    speed_x: 0.0,
    speed_y: 0.0,
    on_ground: false,
    walking: false,
    walking_dir: 0,
  };
  let world = World {
    width: width - 1,
    height: height - 2,
  };

  loop {
    match stdin.next() {
      Some(Ok(b)) => {
        if b == b'q' {
          panic!("Quit");
        } else if b == b'w' {
          jump(&mut player)
        } else if b == b'a' {
          if player.walking && player.walking_dir > 0 {
            player.walking = false;
          } else {
            walk(&mut player, -1)
          }
        } else if b == b'd' {
          if player.walking && player.walking_dir < 0 {
            player.walking = false;
          } else {
            walk(&mut player, 1)
          }
        } else {
          ()
        }
      }
      _ => (),
    }
    simulate(&world, &mut player);
    match draw(&mut stdout, &world, &player) {
      Ok(_) => (),
      Err(e) => panic!("Error during draw: {}", e.msg),
    }
    std::thread::sleep(std::time::Duration::from_millis(20));
  }
}

struct World {
  width: u16,
  height: u16,
}

struct Player {
  x: f32,
  y: f32,
  speed_x: f32,
  speed_y: f32,
  on_ground: bool,
  walking: bool,
  walking_dir: i8,
}

struct DrawingError {
  msg: String,
}

impl std::convert::From<std::io::Error> for DrawingError {
  fn from(err: std::io::Error) -> Self {
    DrawingError {
      msg: String::from(err.description()),
    }
  }
}

fn walk(player: &mut Player, dir: i8) {
  player.walking = true;
  player.walking_dir = dir;
}

fn jump(player: &mut Player) {
  if player.on_ground {
    player.speed_y = -0.5;
  }
}

fn simulate(world: &World, player: &mut Player) {
  player.y = player.y + player.speed_y;
  if player.y < world.height as f32 - 1.0 {
    player.speed_y += 0.05;
    player.on_ground = false;
  } else {
    player.y = world.height as f32 - 1.0;
    player.speed_y = 0.0;
    player.on_ground = true;
  }
  player.x = player.x + player.speed_x;
  if player.on_ground {
    if player.walking {
      player.speed_x = player.speed_x + player.walking_dir as f32 * 0.1;
      if player.speed_x.abs() > 1.0 {
        player.speed_x = player.walking_dir as f32 * 1.0;
      }
    } else {
      player.speed_x = player.speed_x * 0.8;
    }
    if player.speed_x.abs() < 0.1 {
      player.speed_x = 0.0;
    }
  } else {
    player.speed_x = player.speed_x * 0.99;
  }
  if player.x < 1.0 {
    player.x = 1.0;
  } else if player.x > world.width as f32 - 1.0 {
    player.x = world.width as f32 - 1.0;
  }
}

fn draw(
  stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
  world: &World,
  player: &Player,
) -> Result<(), DrawingError> {
  let mut buffer = String::new();
  for y in 0..world.height + 1 {
    let mut cur_line = String::new();
    for x in 0..world.width + 1 {
      if x == player.x as u16 && y == player.y as u16 {
        cur_line.push_str("@");
      } else if x == 0 || x == world.width {
        cur_line.push_str("|");
      } else if y == 0 || y == world.height {
        cur_line.push_str("-");
      } else {
        cur_line.push_str(" ");
      }
    }
    buffer.push_str(cur_line.as_str());
    if y < world.height - 1 {
      buffer.push_str("\n\r");
    }
  }
  write!(stdout, "{}{}", termion::cursor::Goto(1, 1), buffer)?;
  Ok(())
}
