mod lattice;
mod payoff_matrix;
mod bone_lattice;

use std::fs::OpenOptions;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use std::io::{Write, BufWriter};

use image::{ImageBuffer, Rgb, ImageOutputFormat};
use lattice::LatticeIdx;
use payoff_matrix::PayoffMatrix;
use bone_lattice::{BoneLattice, State};

use rand::Rng;

fn main() {

    let ctrlc = {        
        let ctrlc = Arc::new(AtomicBool::new(false));
        let ctrlc_clone = ctrlc.clone();
        ctrlc::set_handler(move || {
            ctrlc.store(true, std::sync::atomic::Ordering::Relaxed)
        }).expect("Error setting ^C handler");
        ctrlc_clone
    };

    let mut lattice: Option<(BoneLattice, Vec<(LatticeIdx, State)>)> = None;

    println!("Will's research project: MATH 89S (Spring 2023)");
    println!("Type \"help\" for a list of commands");

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut input_buffer = String::new();

    loop {

        print!("> ");
        stdout.flush().unwrap();

        // Get console input
        input_buffer.clear();
        let bytes_read = stdin.read_line(&mut input_buffer).unwrap();
        input_buffer.remove(bytes_read - 1);

        let command = match UserCommand::new(&input_buffer) {
            Some(cmd) => cmd,
            None => continue,
        };

        if command.identifier == "exit" { break; }

        run_command(command, &mut lattice, ctrlc.clone());
    }
}

fn run_command(
    mut command: UserCommand,
    lattice: &mut Option<(BoneLattice, Vec<(LatticeIdx, State)>)>,
    ctrlc: Arc<AtomicBool>
) -> Option<()> {
    match command.identifier {
        "init" => {
            let size = command.get_int_arg("size")?;
            let matrix = PayoffMatrix::by_params(
                [
                    command.get_float_arg("alpha1")?,
                    command.get_float_arg("alpha2")?,
                    command.get_float_arg("alpha3")?,
                ],
                [
                    command.get_float_arg("beta1")?,
                    command.get_float_arg("beta2")?,
                    command.get_float_arg("beta3")?,
                ]
            );
            command.error_on_args()?;

            // Create the lattice
            let mut rng = rand::thread_rng();

            *lattice = Some(
                (
                    BoneLattice::new(size as i16, matrix, |_| {
                        rng.gen::<State>()
                    }),
                    Vec::new()
                )
            );
        },
        "step" => {

            let count = command.get_int_arg("num")?;
            command.error_on_args()?;

            // Ensure there's a lattice
            let (lattice, step_buf) = match lattice {
                Some(ref mut stuff) => stuff,
                None => {
                    println!("Use \"init\" or \"load\" to create a lattice");
                    return None;
                },
            };

            let real_pre_time = Instant::now();
            let sim_pre_time = lattice.time;

            step_buf.push(lattice.step());
            println!("First step completed in {}ms", real_pre_time.elapsed().as_millis());

            for i in 1..count {

                if ctrlc.load(std::sync::atomic::Ordering::Relaxed) {
                    println!("Aborted; {} steps completed and t = {}", i, lattice.time);
                    ctrlc.store(false, std::sync::atomic::Ordering::Relaxed);
                    return None;
                }

                step_buf.push(lattice.step());
            }

            let sim_post_time = lattice.time;

            println!("Done; stepped {:.5} to reach t = {:.5} in {}ms", 
                sim_post_time - sim_pre_time, 
                sim_post_time, 
                real_pre_time.elapsed().as_millis()
            );
        }
        "time" => {
            let (lattice, _step_buf) = match lattice {
                Some(ref mut stuff) => stuff,
                None => {
                    println!("Use \"init\" or \"load\" to create a lattice");
                    return None;
                },
            };

            println!("Simulation time is t = {}", lattice.time);
        }
        "sim" => {

            let time_step = command.get_float_arg("time_step")?;
            command.error_on_args()?;
            
            // Ensure there's a lattice
            let (lattice, step_buf) = match lattice {
                Some(ref mut stuff) => stuff,
                None => {
                    println!("Use \"init\" or \"load\" to create a lattice");
                    return None;
                },
            };
            
            let real_start = Instant::now();
            let init_time = lattice.time;
            let final_time = init_time + time_step;
            let mut steps: u32 = 1;

            // Perform one step to get time of first step
            step_buf.push(lattice.step());
            let first_step_time = real_start.elapsed();
            println!("First step completed in {}ms", first_step_time.as_millis());

            // Enter the simulation loop
            let mut last_log = Instant::now();
            while lattice.time < final_time {

                if ctrlc.load(std::sync::atomic::Ordering::Relaxed) {
                    println!("Aborted; {} steps completed and t = {}", steps, lattice.time);
                    ctrlc.store(false, std::sync::atomic::Ordering::Relaxed);
                    return None;
                }

                step_buf.push(lattice.step());
                steps += 1;

                if last_log.elapsed().as_secs() >= 10 {

                    // Reset the time since last log
                    last_log = Instant::now();

                    let elapsed = real_start.elapsed();
                    let progress = lattice.time - init_time;
                    let progress_percent = progress / time_step;
                    
                    // Estimate the time remaining
                    let sim_time_left = time_step - (lattice.time - init_time);
                    let estimate = elapsed.as_secs_f32() * sim_time_left / progress;

                    println!("{:.2}s elapsed: {} steps completed ({:.2}% progress; est. {:.2}s remaining)", 
                        elapsed.as_secs_f32(),
                        steps,
                        progress_percent * 100.0,
                        estimate
                    );
                }
            }

            println!("Done; {} steps completed", steps);
        }
        // "load" => {
        //     match &*command.get_string_arg("kind")? {
        //         "csv" => {
        //             let file = command.get_string_arg("file")?;
        //             let matrix = PayoffMatrix::by_params(
        //                 [
        //                     command.get_float_arg("alpha1")?,
        //                     command.get_float_arg("alpha2")?,
        //                     command.get_float_arg("alpha3")?,
        //                 ],
        //                 [
        //                     command.get_float_arg("beta1")?,
        //                     command.get_float_arg("beta2")?,
        //                     command.get_float_arg("beta3")?,
        //                 ]
        //             );
        //             command.error_on_args()?;

        //             // Open the CSV file
        //             let file_result = File::open(&file);
        //             let file = match file_result {
        //                 Ok(x) => x,
        //                 Err(err) => {
        //                     println!("Error opening file: {}", err);
        //                     return None;
        //                 },
        //             };
                    
        //             // Read the first line of the CSV file to generate the size
        //         }
        //         _ => {
        //             "Invalid load option";
        //         }
        //     }
        // }
        "count" => {
            command.error_on_args();
            // Ensure there's a lattice
            let (lattice, _step_buf) = match lattice {
                Some(ref mut stuff) => stuff,
                None => {
                    println!("Use \"init\" or \"load\" to create a lattice");
                    return None;
                },
            };

            let count = lattice.count();
            println!("resorption: {}", count.0);
            println!("formation: {}", count.1);
            println!("quiescence: {}", count.2);
        }
        "dump" => {
            // Ensure there's a lattice
            let (lattice, step_buf) = match lattice {
                Some(ref mut stuff) => stuff,
                None => {
                    println!("Use \"init\" or \"load\" to create a lattice");
                    return None;
                },
            };

            let kind = command.get_string_arg("type")?;
            let file = command.get_string_arg("path")?;

            let mut open_options = std::fs::OpenOptions::new();
            open_options
                .write(true)
                .create_new(true);

            match &*kind {
                "csv" => {
                    let file_result = open_options.open(&file);
                    let mut file = match file_result {
                        Ok(x) => x,
                        Err(err) => {
                            println!("Error opening file: {}", err);
                            return None;
                        },
                    };
                    for i in 0..lattice.size() {
                        for j in 0..lattice.size() {
                            for k in 0..lattice.size() {
                                let number = match lattice.state(LatticeIdx(i, j, k)) {
                                    State::Resorption => 0,
                                    State::Formation => 1,
                                    State::Quiescence => 2,
                                };
                                write!(file, "{}", number).unwrap();
                                write!(file, ",").unwrap();
                            }
                            write!(file, "\n").unwrap();
                        }
                        write!(file, "\n").unwrap();
                    }
                }
                "count" => {
                    let file_result = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(file);
                    let mut file = match file_result {
                        Ok(x) => x,
                        Err(err) => {
                            println!("Error opening file: {}", err);
                            return None;
                        },
                    };
                    let count = lattice.count();
                    writeln!(file, "{:.5},{},{},{}", lattice.time, count.0, count.1, count.2).unwrap();
                }
                "img" => {
                    let path = std::path::PathBuf::from(file);
                    for img_idx in 0..lattice.size() {
                        let mut img = ImageBuffer::<Rgb<u8>, _>::new(
                            lattice.size() as u32, lattice.size() as u32);
                        for (i, j, pixel) in img.enumerate_pixels_mut() {
                            *pixel = match lattice.state(LatticeIdx(img_idx, i as i16, j as i16)) {
                                State::Resorption => Rgb([0, 0, 255]),
                                State::Formation => Rgb([150, 200, 150]),
                                State::Quiescence => Rgb([255, 255, 0]),
                            }
                        }
                        let writer_result = open_options
                            .open(path.join(&format!("layer{}.png", img_idx)));
                        let writer = match writer_result {
                            Ok(x) => x,
                            Err(err) => {
                                println!("Error opening file: {}", err);
                                return None;
                            }
                        };
                        let mut writer = BufWriter::new(writer);
                        let write_result = img.write_to(&mut writer, ImageOutputFormat::Png);
                        match write_result {
                            Ok(_) => {},
                            Err(err) => {
                                println!("Error writing image: {}", err);
                                return None;
                            },
                        }
                    }
                },
                "steps" => {
                    let file_result = open_options.open(file);
                    let mut file = match file_result {
                        Ok(x) => x,
                        Err(err) => {
                            println!("Error opening file: {}", err);
                            return None;
                        },
                    };
                    for (idx, state) in step_buf {
                        let state_num = match state {
                            State::Resorption => 0,
                            State::Formation => 1,
                            State::Quiescence => 2,
                        };
                        writeln!(file, "{},{},{},{}", idx.0, idx.1, idx.2, state_num).unwrap();
                    }
                }
                _ => {
                    println!("Unknown file type");
                }
            }
        }
        "help" => {
            println!("lmao you thought. (todo)");
        }
        _ => {
            println!("That command doesn't exist (type \"help\")");
        }
    };

    Some(())
}

/// A processed command issued by the user
struct UserCommand<'a> {
    pub identifier: &'a str,
    arg_iter: std::str::Split<'a, &'static str>,
}

impl<'a> UserCommand<'a, > {
    pub fn new(command: &'a str) -> Option<Self> {
        let mut args = command.split(" ");
        args.next().map(|identifier| {
            UserCommand {
                identifier,
                arg_iter: args
            }
        })
    }
 
    /// Gets a string arg and prints an error message otherwise.
    pub fn get_string_arg(&mut self, name: &str) -> Option<String> {
        match self.arg_iter.next() {
            Some(str) => Some(str.to_owned()),
            None => {
                println!("Expected string argument: {}", name);
                None
            }
        }
    }

    /// Gets a string arg and prints an error message otherwise.
    pub fn get_int_arg(&mut self, name: &str) -> Option<i64> {
        let arg = match self.arg_iter.next() {
            Some(str) => Some(str),
            None => {
                println!("Expected int argument: {}", name);
                None
            }
        }?;

        match arg.parse() {
            Ok(int) => Some(int),
            Err(_err) => {
                println!("Expected int argument: {}", name);
                None
            }
        }
    }

    /// Gets a string arg and prints an error message otherwise.
    pub fn get_float_arg(&mut self, name: &str) -> Option<f32> {
        let arg = match self.arg_iter.next() {
            Some(str) => Some(str),
            None => {
                println!("Expected float argument: {}", name);
                None
            }
        }?;

        match arg.parse() {
            Ok(int) => Some(int),
            Err(_err) => {
                println!("Expected float argument: {}", name);
                None
            }
        }
    }

    // pub fn get_payoff_matrix_arg(&mut self, name: &str) -> Option<PayoffMatrix> {
    //     let a1 = self.get_float_arg("a1")?;
    //     let a2 = self.get_float_arg("a2")?;
    //     let a3 = self.get_float_arg("a3")?;
    //     match &*self.get_string_arg("/")? {
    //         "/" => Some(()),
    //         other => {
    //             println!("Expected divider \"/\", got {}", other);
    //             None
    //         }
    //     }?;
    //     let b1 = self.get_float_arg("b1")?;
    //     let b2 = self.get_float_arg("b2")?;
    //     let b3 = self.get_float_arg("b3")?;
    //     match &*self.get_string_arg("/")? {
    //         "/" => Some(()),
    //         other => {
    //             println!("Expected divider \"/\", got {}", other);
    //             None
    //         }
    //     }?;
    //     let c1 = self.get_float_arg("c1")?;
    //     let c2 = self.get_float_arg("c2")?;
    //     let c3 = self.get_float_arg("c3")?;
    //     Some(PayoffMatrix::new([a1, a2, a3], [b1, b2, b3], [c1, c2, c3]))
    // }

    /// Ensures that there are no more arguments, errors with [`None`] and an
    /// error message otherwise.
    pub fn error_on_args(&mut self) -> Option<()> {
        match self.arg_iter.next() {
            Some(_arg) => {
                println!("Too many arguments in command");
                None
            },
            None => Some(()),
        }
    }
}
