
mod draw;
use draw::*;

mod logic;
use logic::*;

mod consts;
use consts::*;

use std::{
    io::{stdin, stdout, Write},
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread
};
use termion::{
    cursor::Goto,
    event::{Event, Key},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
};
//clock rng
use rand::{thread_rng, rngs::ThreadRng };

fn main() {
    let _stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    let (tx, rx) = sync_channel(2);

    let input = thread::spawn(move || {
        input_loop(tx);
    });

    let game = thread::spawn(|| {
        game_loop(rx);
    });

    game.join().unwrap();
    input.join().unwrap();
}
fn game_loop(rx: Receiver<char>) {
    //setup
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    let mut print_buffer: String = String::new();
    print_buffer.push_str(&format!(
        "{}",
        Goto(1,1)
    ));
    //clear screen
    print_buffer.push_str("\x1b[2J");
    print_buffer.push_str(START_GUI);

    //hide cursor
    print_buffer.push_str("\x1b[?25l");

    stdout.write_all(print_buffer.as_bytes()).unwrap();

    let mut rng: ThreadRng =  thread_rng();

    let mut board: [[u16; 10]; 24] = [[0; 10]; 24];
    let mut old_board: [[u16; 10]; 24] = [[0; 10]; 24];

    let mut current_block_bag: [usize; 7] = create_shuffled_bag(&mut rng);
    let mut block_bag_index = 0;
    let mut next_block_bag: [usize; 7] = create_shuffled_bag(&mut rng);
    
    let mut current_block: Block = Block {
        x: 3,
        y: 0,
        rotation: 0,
        shape: current_block_bag[block_bag_index],
    };
    let mut old_block: Block = current_block;
    let mut has_swapped: bool = false;
    let mut swap_block: Block = Block {
        x: 0,
        y: 0,
        rotation: 0,
        shape: 7,
    };

    let mut frame_number: u32 = 0;
    let mut score: u32 = 0;
    let level: usize = 0;


    loop {
        frame_number += 1;

        let mut new_block = current_block;


        match match rx.try_recv() {
            Ok(key) => key,
            Err(_) => '.',
        } {
            'q' => {
                //show cursor
                print_buffer.push_str("\x1b[?25h");
                stdout.write_all(print_buffer.as_bytes()).unwrap();
                panic!("Quit");
            },
            'w' => {
                for i in 0..5 {
                    new_block.x = current_block.x + 
                        if current_block.shape == 0 {
                            I_KICKS
                        } else {
                            NORMAL_KICKS
                        }[current_block.rotation][i][0] as usize;

                    new_block.y = current_block.y + 
                        if current_block.shape == 0 {
                            I_KICKS
                        } else {
                            NORMAL_KICKS
                        }[current_block.rotation][i][1] as usize;

                        new_block.rotation = (current_block.rotation + 1) % 4;
                        if check_transform(board, new_block) {
                            break;
                        }
                }
                //new_block.rotation = (curr_block.rotation + 1) % 4;
            },
            's' => {
                frame_number = LEVEL_GRAVITY[level];
            },
            'a' => {
                new_block.x -= 1;
            },
            'd' => {
                new_block.x += 1;
            },
            'e' => {
                new_block.shape = (new_block.shape + 1) % 7;
            },
            'c' => {
                if !has_swapped {
                    current_block.x = 3;
                    current_block.y = 0;
                    current_block.rotation = 0;
                    has_swapped = true;
                    frame_number = LEVEL_GRAVITY[level];
                    new_block.shape = swap_block.shape;
                    swap_block.shape = current_block.shape;
                    current_block.shape = new_block.shape;
                    update_hold_block_graphics(swap_block.shape, &mut stdout)
                }
            },
            ' ' => {
                while check_transform(board, new_block) {
                    new_block.y += 1;
                }
                new_block.y -= 1;
                frame_number = LEVEL_GRAVITY[level];
            },
            _ => {

            },
        };


        if check_transform(board, new_block) {
            current_block = new_block;
        };

        if frame_number >= LEVEL_GRAVITY[level] {
            frame_number = 0;
            new_block = current_block;
            
            //lock block if it can't move down
            new_block.y += 1;
            if !check_transform(board, new_block) {
                if new_block.shape != 7 {
                    has_swapped = false;
                }
                board = lock_block(board, current_block);
                block_bag_index += 1;

                if block_bag_index == 7 {
                    current_block_bag = next_block_bag.clone();
                    next_block_bag = create_shuffled_bag(&mut rng);
                    block_bag_index = 0;
                };
                current_block = Block {
                    x: 3,
                    y: 0,
                    rotation: 0,
                    shape: current_block_bag[block_bag_index]
                };
                update_next_blocks_graphics(current_block_bag, next_block_bag, block_bag_index, &mut stdout);
            } else {
                current_block = new_block;
            }
        }
        score += clear_lines(&mut stdout, &mut board, current_block);

        if board != old_board || current_block != old_block {
            update_board_graphics(board, old_board, current_block, old_block, &mut stdout);
        };

        old_block = current_block;
        old_board = board;
        
        thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn input_loop(tx: SyncSender<char>) {
    let stdin = stdin();

    for c in stdin.events() {
        let evt = c.unwrap();
        let _: bool = match evt {
            Event::Key(ke) => match ke {
                Key::Up => tx.try_send('w').is_err(),
                Key::Down => tx.try_send('s').is_err(),
                Key::Left => tx.try_send('a').is_err(),
                Key::Right => tx.try_send('d').is_err(),
                Key::Char(k) => match k {
                    'q' => tx.try_send('q').is_err(),
                    x => {
                        let thread_tx = tx.clone();
                        thread_tx.try_send(x).is_err()
                    }
                },
                _ => false,
            },
            _ => false,
        };
    }
}
