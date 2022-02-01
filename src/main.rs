use std::{
    fmt::Display,
    io::stdout,
    ops::Range,
    time::{Duration, SystemTime},
};

use async_std::{
    net::{TcpStream, TcpListener},
    task::block_on,
};
use futures::{future::FutureExt, select, AsyncReadExt, AsyncWriteExt, StreamExt, pin_mut};
use futures_timer::Delay;

use crossterm::{
    cursor::{Hide, MoveTo, Show, MoveLeft},
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, self},
    execute,
    style::{Color, Print},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Result,
};
use serde::{Serialize, Deserialize};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Character {
    unique_id: u16,

    x: i16,
    y: i16,
    denotation: char,
    color: Color,
    hp: i16,
    attack_skill: i16,
    damage_range: Range<i16>,

    defence_class: i16,
    attack_range: i16,

    attack_rate: i16,
    attack_cooldown: i16,

    movement_rate: i16,
    movement_cooldown: i16,

    is_attacking: bool,
}

#[derive(Debug, PartialEq)]
enum CommandState {
    Menu,
    MainGame,
    CharacterSelected(char),
    Chat,
}

fn generate_barbarian(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'B',
        color: c,
        hp: 12,
        attack_skill: 3,
        defence_class: 9,
        attack_range: 1,
        damage_range: (1..7),
        attack_rate: 5,
        attack_cooldown: 5,
        movement_rate: 7,
        movement_cooldown: 7,
        is_attacking: false,
    }
}

fn generate_archer(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'A',
        color: c,
        hp: 6,
        attack_skill: 2,
        defence_class: 7,
        attack_range: 5,
        damage_range: (1..4),
        attack_rate: 10,
        attack_cooldown: 10,
        movement_rate: 13,
        movement_cooldown: 13,
        is_attacking: false,
    }
}

fn generate_giant(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'G',
        color: c,
        hp: 30,
        attack_skill: 4,
        defence_class: 12,
        attack_range: 1,
        damage_range: (6..12),
        attack_rate: 15,
        attack_cooldown: 15,
        movement_rate: 30,
        movement_cooldown: 30,
        is_attacking: false,
    }
}

async fn events() {
    let mut pieces: Vec<Character> = Vec::new();
    //generate_war(&mut pieces);

    let mut mode: Color = Color::Green;
    let mut command_state: CommandState = CommandState::Menu;
    let mut reader = EventStream::new();

    let mut game_session_code : String = String::new();
    print_at(50, 0, format!("Game Code: | {}b | {}", 0, game_session_code));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listening_port = listener.local_addr().unwrap().port();
    print_at(20, 0, format!("Port: {}", listening_port));

    let mut port: u16 = 0;

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_0)).fuse();
        let mut term_event = reader.next().fuse();
        let nla_event = listener.accept().fuse();
        pin_mut!(nla_event);

        select! {
            _ = delay => {
                // updates every tick of delay

                //let term_size = terminal::size().unwrap();
                print_at(10 + (now() % 60) as u16, 2, format!(" Now: {:?} ", now() ));

                update_movement(&mut pieces);
                update_attacks(&mut pieces);
                
                let bin = bincode::serialize(&pieces).unwrap();
                call(&bin, port).await;

                print_at(100, 2, format!("{:?}", mode));
                logging_tail().await;

                render_grid(5, 4, &pieces, &command_state);
            },
            nla_handler = nla_event => {
                let stream = match nla_handler {
                    Ok(e) => e.0,
                    Err(_) => continue,
                };

                handle_connection(stream, &mut pieces).await;
            },
            term_handler = term_event => {
                let mut key_code : KeyCode = KeyCode::Null;

                match term_handler {
                    Some(Ok(evt)) => {

                        match evt {
                            Event::Key(key) => {
                                key_code = key.code
                            },
                            //Event::Mouse(_) => (),
                            Event::Resize(w, h) => {
                                cls();
                                print_at(50, 2, format!("Terminal Size : {w}x{h}"));
                            },
                            _ => ()
                        }

                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }

                match command_state {
                    CommandState::Menu => {
                        match key_code {
                            KeyCode::Char('n') => {
                                print_at(20, 1, "Connect to server port: ".to_string());
                                port = match read_line().unwrap().parse() {
                                    Ok(n) => n,
                                    Err(_) => 0
                                };
                                if port > 0 {
                                    game_session_code = call(b"new game", port).await;
                                    print_at(50, 0, format!("Game Code: {game_session_code}"));
    
                                    command_state = CommandState::MainGame;
                                }
                            },
                            KeyCode::Char('t') => { command_state = CommandState::Chat; }
                            KeyCode::Char('c') => { cls(); }
                            KeyCode::Char('q') => break,
                            _ => (),
                        }

                    },
                    CommandState::MainGame => {
                        if let KeyCode::Char(c) = key_code {
                            print_at(40,0,format!("Char: {}", c));
                            match c {
                                'q' => command_state = CommandState::Menu,
                                'r' => mode = Color::Red,
                                'l' => mode = Color::Green,
                                _ => command_state = CommandState::CharacterSelected(c)
                            }
                        }
                    },
                    CommandState::CharacterSelected(c) => {
                        let mut character = match c {
                            'b' => generate_barbarian(0, mode),
                            'a' => generate_archer(0, mode),
                            'g' => generate_giant(0, mode),
                            _ => {
                                // invalid entry, back out to main game
                                command_state = CommandState::MainGame;
                                continue
                            },
                        };

                        if let KeyCode::Char(r) = key_code {
                            if r >= '1' && r <= '9' {
                                let r_val : u32 = r as u32;
                                character.y = ((r_val - 48) * 2) as i16;
                                pieces.push(character);

                                let code = format!("{c}{r}");
                                let code = code.as_bytes();
                                call(&code, port).await;
                                command_state = CommandState::MainGame;
                            }
                        } else {
                            match key_code {
                                KeyCode::Esc => { command_state = CommandState::MainGame; },
                                _ => (),
                            }
                        }
                    },
                    CommandState::Chat => todo!(),
                }

            }
        };
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    execute!(stdout(), EnableMouseCapture, Hide)?;

    color_set(Color::Reset, Color::Black);
    cls();

    async_std::task::block_on(events());

    execute!(stdout(), DisableMouseCapture, Show)?;

    disable_raw_mode()?;

    // print_at(0, termSize.1-1, ": ");

    color_reset();
    Ok(())
}

fn render_grid(x: u16, y: u16, pieces: &Vec<Character>, command_state: &CommandState) {
    // play area is a 70x20

    print_at(0, 0, format!("{:?}         ", command_state));

    rect_outline('█', x - 1, y - 1, 70 + 2, 20 + 2);
    for i in 1..10 {
        print_at(x - 1, y + (i * 2), i.to_string());
        //print_at(x+71, y+(i*2), i.to_string());
    }
    color_set(Color::White, Color::Black);
    rect_filled(" ", x, y, 70, 20);
    draw_line('░', x + 5, y, x + 5, y + 20);
    draw_line('░', x + 65, y, x + 65, y + 20);

    for p in pieces {
        if p.hp > 0 {
            color_set(p.color, Color::Black);
            print_at(x + p.x as u16, y + p.y as u16, p.denotation);
        }
    }

    color_reset();
}

fn cls() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
}

fn print_at<T: Display>(x: u16, y: u16, s: T) {
    execute!(stdout(), MoveTo(x, y), Print(s),).unwrap();
}

fn color_set(fg: Color, bg: Color) {
    execute!(
        stdout(),
        crossterm::style::SetBackgroundColor(bg),
        crossterm::style::SetForegroundColor(fg),
    )
    .unwrap();
}

fn color_reset() {
    execute!(stdout(), crossterm::style::ResetColor,).unwrap();
}

fn rect_filled(draw: &str, x: u16, y: u16, width: u16, height: u16) {
    let fill = draw.repeat(width as usize);
    print_at(60, 1, format!("s |{draw}| x{x} y{y} w{width} h{height}"));
    for i in y..y + height {
        print_at(x, i, &fill);
    }
}

fn rect_outline(draw: char, x: u16, y: u16, width: u16, height: u16) {
    /* x, y                            x+w, y
     +---------------------------------+
     |                                 |
     |                                 |
     |                                 |
     |                                 |
     |                                 |
     +---------------------------------+
    x, y+h                          x+w, y+h */

    draw_line(draw, x, y, x + width - 1, y);
    draw_line(draw, x, y + height - 1, x + width - 1, y + height - 1);
    draw_line(draw, x, y, x, y + height - 1);
    draw_line(draw, x + width - 1, y, x + width - 1, y + height - 1);
}

fn draw_line(draw: char, x1: u16, y1: u16, x2: u16, y2: u16) {
    // draw_line is end point inclusive
    let points = calc_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32);

    for p in points {
        print_at(p.0 as u16, p.1 as u16, draw);
    }
}

fn calc_line(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut coordinates: Vec<(i32, i32)> = vec![];
    let dx = i32::abs(x2 - x1);
    let dy = i32::abs(y2 - y1);
    let sx = {
        if x1 < x2 {
            1
        } else {
            -1
        }
    };
    let sy = {
        if y1 < y2 {
            1
        } else {
            -1
        }
    };

    let mut error = (if dx > dy { dx } else { -dy }) / 2;
    let mut current_x = x1;
    let mut current_y = y1;
    loop {
        coordinates.push((current_x, current_y));

        if current_x == x2 && current_y == y2 {
            break;
        }

        let error2 = error;

        if error2 > -dx {
            error -= dy;
            current_x += sx;
        }
        if error2 < dy {
            error += dx;
            current_y += sy;
        }
    }
    coordinates
}

fn calc_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    if y1 == y2 {
        return 0.0001;
    };

    // √[(x₂ - x₁)² + (y₂ - y₁)²]
    let x = x2 - x1;
    let x = x.pow(2) as f32;
    let y = y2 - y1;
    let y = y.pow(2) as f32;

    return (x + y).sqrt();
}

fn update_movement(pieces: &mut Vec<Character>) {
    let mut ids: Vec<usize> = (0..pieces.len()).collect();
    fastrand::shuffle(&mut ids);

    for i in ids {
        if pieces[i].hp < 1 {
            continue;
        }

        // need to play test if letting movement cooldown continue during attacking or not has a positive/negative effect on play
        if pieces[i].is_attacking {
            continue;
        }

        pieces[i].movement_cooldown -= 1;

        if pieces[i].movement_cooldown <= 0 {
            // find the shortest distance to the nearest enemy
            let mut shortest_distance: f32 = 99999.999;
            let mut closest_enemy: usize = pieces.len();

            for j in 0..pieces.len() {
                // check that the item is not an enemy
                if pieces[i].color != pieces[j].color && pieces[j].hp > 0 {
                    let dist = calc_distance(
                        pieces[i].x as i32,
                        pieces[i].y as i32,
                        pieces[j].x as i32,
                        pieces[j].y as i32,
                    );
                    if dist < shortest_distance {
                        shortest_distance = dist;
                        closest_enemy = j;
                    }
                }
            }

            if closest_enemy < pieces.len() {
                let mut movex = 0;
                let mut movey = 0;

                // enemy located
                if pieces[i].x < pieces[closest_enemy].x {
                    movex = 1 as i16;
                }
                if pieces[i].x > pieces[closest_enemy].x {
                    movex = -1 as i16;
                }
                if pieces[i].y < pieces[closest_enemy].y {
                    movey = 1 as i16;
                }
                if pieces[i].y > pieces[closest_enemy].y {
                    movey = -1 as i16;
                }

                let mut valid_move = true;
                for j in 0..pieces.len() {
                    if i == j {
                        continue;
                    } // ignore self
                    if pieces[j].hp <= 0 {
                        continue;
                    } // ignore dead

                    if pieces[i].x + movex == pieces[j].x && pieces[i].y + movey == pieces[j].y {
                        valid_move = false;
                        break;
                    }
                }
                if valid_move {
                    pieces[i].x += movex;
                    pieces[i].y += movey;
                    pieces[i].movement_cooldown = pieces[i].movement_rate;
                }
            }
        }
    }
}

fn update_attacks(pieces: &mut Vec<Character>) {
    let mut ids: Vec<usize> = (0..pieces.len()).collect();
    fastrand::shuffle(&mut ids);

    for i in ids {
        if pieces[i].hp < 1 {
            continue;
        }

        if pieces[i].attack_cooldown > 0 {
            pieces[i].attack_cooldown -= 1;
        } else {
            //block_on(logging(format!("{}{:0x} ready to attack", pieces[i].denotation, pieces[i].unique_id)));
            pieces[i].is_attacking = false;

            for j in 0..pieces.len() {
                // check that the item is not an enemy and is alive
                if pieces[i].color != pieces[j].color && pieces[j].hp > 0 {
                    // check that the items are in range of each other for effect
                    if pieces[i].x >= pieces[j].x - pieces[i].attack_range
                        && pieces[i].x <= pieces[j].x + pieces[i].attack_range
                        && pieces[i].y >= pieces[j].y - pieces[i].attack_range
                        && pieces[i].y <= pieces[j].y + pieces[i].attack_range
                    {
                        block_on(logging(format!(
                            "{}{:0x} will attack {}{:0x}",
                            pieces[i].denotation,
                            pieces[i].unique_id,
                            pieces[j].denotation,
                            pieces[j].unique_id
                        )));

                        // pause moving while attacking - glass cannons don't want to be walking to their death
                        pieces[i].is_attacking = true;

                        // attack rolls
                        // 2d6 + attack_skill > enemy defence_class

                        let attack_roll = fastrand::i16(1..7) + fastrand::i16(1..7) + pieces[i].attack_skill;
                        block_on(logging(format!(
                            "{}{:0x} rolled to attack: {} vs enemy defence: {}",
                            pieces[i].denotation,
                            pieces[i].unique_id,
                            attack_roll,
                            pieces[j].defence_class
                        )));

                        if attack_roll >= pieces[j].defence_class {
                            block_on(logging(format!(
                                "{}{:0x} passed attack roll",
                                pieces[i].denotation, pieces[i].unique_id
                            )));

                            // passed check, do damage
                            let damage = fastrand::i16(
                                pieces[i].damage_range.start..pieces[i].damage_range.end,
                            );
                            pieces[j].hp -= damage;

                            block_on(logging(format!(
                                "{}{:0x} causes {} damage, leaving {} hp",
                                pieces[i].denotation, pieces[i].unique_id, damage, pieces[j].hp
                            )));

                            if pieces[j].hp <= 0 {
                                pieces[i].is_attacking = false;
                                block_on(logging(format!(
                                    "{}{:0x} defeated enemy",
                                    pieces[i].denotation, pieces[i].unique_id
                                )));
                            }
                        }

                        pieces[i].attack_cooldown = pieces[i].attack_rate;
                    }
                }
            }
        }
    }
}

async fn logging(s: String) {
    let mut file = async_std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("logging.txt")
        .await
        .unwrap();
    //write!(&mut file, s);
    let log = format!("{}  {s}\n", now());
    let _ = AsyncWriteExt::write_all(&mut file, log.as_bytes()).await;
}

async fn logging_tail() {
    let mut file = match async_std::fs::OpenOptions::new()
        .read(true)
        .open("logging.txt")
        .await {
            Ok(f) => f,
            Err(_) => return, // file not found, nothing to show
        };

    let mut output = String::with_capacity(1_000_000);

    let _ = AsyncReadExt::read_to_string(&mut file, &mut output).await;

    let lines: Vec<&str> = output.split_terminator('\n').collect();

    let mut start_line = lines.len();
    if start_line > 20 {
        start_line -= 20;
    }

    let mut j = 0;
    for i in start_line..lines.len() {
        print_at(80, (j + 4) as u16, format!("{}               ", lines[i]));
        j += 1;
    }
}

async fn call(send : &[u8], port : u16) -> String {
    logging(String::from_utf8(send.to_vec()).unwrap_or(format!("{:?}", send))).await;
    match TcpStream::connect(format!("127.0.0.1:{port}")).await {
        Ok(stream) => {
            print_at(35, 0, "           ");
            let mut stream = stream;
            match AsyncWriteExt::write_all(&mut stream, send).await {
                Ok(x) => print_at(80, 5, format!("Ok {:?}     ", x)),
                Err(e) => print_at(80, 5, format!("Err {:?}     ", e)),
            }
            //print_at(50, 0, "Waiting for game...");
            let mut buf = vec![0u8; 1024];
            let n = AsyncReadExt::read(&mut stream, &mut buf).await.unwrap();
            let result = String::from_utf8(buf[..n].to_vec()).unwrap();
            result
        },
        Err(_) => {
            print_at(35, 0, "!Network 📶!");
            "".to_string()
        },
    }

}

async fn handle_connection(mut stream: TcpStream, pieces : &mut Vec<Character>) {
    logging("Incoming connection".to_string()).await;
    let mut buffer = vec![0; 1024];

    let size = match stream.read(&mut buffer).await {
        Ok(x) => { logging(format!("Ok {:?}", x)).await; x },
        Err(x) => { logging(format!("Err {:?}", x)).await; 0 },
    };

    let request = match String::from_utf8(buffer[..size].to_vec()) {
        Ok(r) => r,
        Err(_) => "".to_string(),
    };

    logging(format!("Buffer: {:?}", request)).await;

    let mut response : String = String::new();

    let request_str = request.as_str();
    if request_str == "new game" {
        let id = fastrand::u128(..).to_string();
        println!("{id}");
        response = id
    } else if request_str.len() == 2 {
        let mut request_bytes = request_str.bytes();
        let chr = request_bytes.next().unwrap();
        let y: i16 = (request_bytes.next().unwrap() as i16 - 48) * 2;
        logging(format!("Character: {chr}   y: {y}")).await;

        match chr {
            b'g' => { pieces.push( generate_giant(y as i16, Color::Red) ); },
            b'b' => { pieces.push( generate_barbarian(y as i16, Color::Red) ); },
            b'a' => { pieces.push( generate_archer(y as i16, Color::Red) ); },
            _ => ()
        }
    }

    if response.len() > 0 {
        logging(format!("Sending response")).await;
    
        stream.write(response.as_bytes()).await.unwrap();
        match stream.flush().await {
            Ok(x) => { logging(format!("Ok {:?}", x)).await; },
            Err(x) => { logging(format!("Err {:?}", x)).await; },
        }
    }

    logging(format!("Connection handled")).await;
}

pub fn read_line() -> Result<String> {
    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Enter => {
                break;
            }
            KeyCode::Char(c) => {
                execute!(stdout(), Print(c),).unwrap();
                line.push(c);
            }
            KeyCode::Backspace => {
                execute!(stdout(), MoveLeft(1), Print(' '), MoveLeft(1),).unwrap();

                line.pop();
            }
            _ => {}
        }
    }

    Ok(line)
}