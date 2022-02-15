use std::time::Duration;

use async_std::{
    net::{TcpListener, TcpStream},
    task::block_on,
};
use crossterm::style::Color;
use futures::{pin_mut, select, AsyncReadExt, AsyncWriteExt, FutureExt};
use futures_timer::Delay;

use crate::lib::*;
use crate::utils::*;

pub async fn server(listener: TcpListener) {
    let mut pieces: Vec<Character> = Vec::new();

    loop {
        let mut delay = Delay::new(Duration::from_millis(10)).fuse();
        let nla_event = listener.accept().fuse();
        pin_mut!(nla_event);

        select! {
            _ = delay => {
                if pieces.len() > 1 {
                    update_movement(&mut pieces);
                    update_attacks(&mut pieces);
                }
            },
            nla_handler = nla_event => {
                let stream = match nla_handler {
                    Ok(e) => e.0,
                    Err(_) => continue,
                };
                handle_connection(stream, &mut pieces).await;
            },

        }
    }
}

async fn handle_connection(mut stream: TcpStream, pieces: &mut Vec<Character>) {
    let mut buffer = vec![0; 1024];

    let size = match stream.read(&mut buffer).await {
        Ok(x) => x,
        Err(x) => {
            logging(format!("👂 Err Read {:?}", x)).await;
            0
        }
    };

    let request = match String::from_utf8(buffer[..size].to_vec()) {
        Ok(r) => r,
        Err(_) => "".to_string(),
    };

    let mut response: Vec<u8> = Vec::new();

    let request_str = request.as_str();
    if request_str == "new game" {
        let id = format!("{:x}", fastrand::u128(..));
        println!("{id}");
        response = id.into_bytes();
    } else if request_str == "update" {
        response = bincode::serialize(&pieces).unwrap();
    } else if request_str.len() == 3 {
        let mut request_bytes = request_str.bytes();
        let col = request_bytes.next().unwrap();
        let chr = request_bytes.next().unwrap();
        let y: i16 = (request_bytes.next().unwrap() as i16 - 48) * 2;
        let col = if col == b'r' {
            Color::Red
        } else {
            Color::Green
        };

        match chr {
            b'g' => {
                pieces.push(generate_giant(y as i16, col));
            }
            b'b' => {
                pieces.push(generate_barbarian(y as i16, col));
            }
            b'a' => {
                pieces.push(generate_archer(y as i16, col));
            }
            _ => (),
        }
    }

    if !response.is_empty() {
        stream.write_all(&response[..]).await.unwrap();
        match stream.flush().await {
            Ok(_) => (),
            Err(x) => {
                logging(format!("👂 Err Write {:?}", x)).await;
            }
        }
    }
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

    (x + y).sqrt()
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
                    movex = 1_i16;
                }
                if pieces[i].x > pieces[closest_enemy].x {
                    movex = -1_i16;
                }
                if pieces[i].y < pieces[closest_enemy].y {
                    movey = 1_i16;
                }
                if pieces[i].y > pieces[closest_enemy].y {
                    movey = -1_i16;
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

                        let attack_roll =
                            fastrand::i16(1..7) + fastrand::i16(1..7) + pieces[i].attack_skill;
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
