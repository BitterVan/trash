// 这个文件中定义了输入输出所需要的工具
use std::io::BufRead;
// use std::{fs::File, io::{self, Read, Write}, sync::mpsc::Receiver};
// use std::io::Read;
use std::io::Write;
// use std::sync::mpsc::Sender;
use std::io;
    #[allow(deprecated)]
use std::env::{current_dir, home_dir};
use users::*;
use ansi_term::Colour::Cyan;
use ansi_term::Colour::Red;
use ansi_term::Colour::White;
use ansi_term::Style;
// use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

// use std::sync::mpsc::Sender;
use super::bases::*;
// This function is used to input a commandline from stdin
// All the behaviors in trash is followed by an input from stdin

// 用于从标准输入中读取命令
pub fn input_cmd() -> io::Result<String> {
    let mut ret = String::new();
    // If fail, return the error from stdin
    // 需要上锁之后再进行读取，否则会导致缓冲区混乱
    io::stdin().lock().read_line(&mut ret)?;
    Ok(ret)
}

// 下面是一些非阻塞读入的函数，但是由于涉及系统死锁，所以最后没有加入
// pub fn in_unblock(unblock_src: InputSrc, tx: Sender<u8>) -> io::Result<()> {
//     match unblock_src {
//         InputSrc::Channel(rx) => {
//             loop {
//                 tx.send(rx.recv().unwrap()).unwrap();
//             }
//         },
//         InputSrc::File(file_name) => {
//             let mut file = File::open(file_name)?;
//             let mut buf = Vec::new();
//             file.read_to_end(&mut buf)?;
//             for i in buf {
//                 tx.send(i).unwrap();
//             }
//         },
//         InputSrc::Stdin => {
//             let mut input = io::stdin();
//             let mut buf = Vec::new();
//             loop {
//                 input.read(&mut buf)?;
//                 tx.send(buf[0]).unwrap();
//             }
//         },
//     }
//     Ok(())
// }

// pub fn input_std(tx: Sender<u8>) {
//     let mut stdin= io::stdin();
//     let mut buf = [0];
//     loop {
//         stdin.read_exact(&mut buf).unwrap();
//         tx.send(buf[0]).unwrap();
//     }
// }

// pub fn output_std(rx: Receiver<u8>) {
//     let mut stdout = io::stdout();
//     loop {
//         let recv = rx.recv().unwrap();
//         if recv == BREAK_IO {
//             break;
//         }
//         stdout.write(&[recv]).unwrap();
//     }
// }

// pub fn output_err(rx: Receiver<u8>) {
//     let mut stderr = io::stderr();
//     loop {
//         let recv = rx.recv().unwrap();
//         if recv == BREAK_IO {
//             break;
//         }
//         stderr.write(&[recv]).unwrap();
//     }
// }

// pub fn out_unblock(rx: Receiver<u8>, unblock_des: UnblockDes) -> io::Result<()> {
//     match unblock_des {
//         UnblockDes::Line(tx) => {
//             let buf: Vec<u8> = rx.recv().into_iter().collect();
//             let buf = String::from_utf8(buf).unwrap();
//             tx.send(buf).unwrap();
//         }
//     }
//     Ok(())
// }

// 将内错误内容输出到指定位置
pub fn write_error(mut info: String, error_dst: ErrorDst) {
    // 匹配输出方式
    match error_dst {
        // 如果是输出到stderr
        ErrorDst::Stderr => {
            info.push('\n');
            // 直接进行写入
            io::stderr().write(info.as_bytes()).unwrap();
        },
        // 输出到管道
        ErrorDst::Channel(tx) => {
            // 将info中的byte逐个输出到管道中，这里支持各种输入，所以使用byte而非char类型
            for i in info.bytes() {
                tx.send(i).unwrap();
            }
            // 发送终止信号0
            tx.send(0).unwrap();
        },
        // 输出到文件
        ErrorDst::File(file_name, append) => {
            // println!("file_name: {}", file_name);
            // 如果不是进行append，就首先创建新文件
            if !append {
                // 创建文件/覆盖文件
                std::fs::File::create(&file_name).unwrap();
            }
            info.push('\n');
            // 按照指定的文件名和写入方式打开文件
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(append)
                .open(file_name.trim())
                .unwrap();
            // 写入所有需要输出的内容
            file.write_all(info.as_bytes()).unwrap();
        },
    }
}

pub fn write_out(output: String, output_dst: OutputDst) {
    // 匹配输出方式
    match output_dst {
        // 如果是输出到stdout
        OutputDst::Stdout => {
            // 直接进行写入
            io::stdout().write(output.as_bytes()).unwrap();
        },
        // 输出到管道
        OutputDst::Channel(tx) => {
            // 将info中的byte逐个输出到管道中，这里支持各种输入，所以使用byte而非char类型
            for i in output.bytes() {
                tx.send(i).unwrap();
            }
            // 发送终止信号0
            tx.send(0).unwrap();
        },
        // 输出到文件
        OutputDst::File(file_name, append) => {
            // 如果不是进行append，就首先创建新文件
            if !append {
                // 创建文件/覆盖文件
                std::fs::File::create(&file_name).unwrap();
            }
            // 按照指定的文件名和写入方式打开文件
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(append)
                .open(file_name.trim())
                .unwrap();

            // 写入所有需要输出的内容
            file.write_all(output.as_bytes()).unwrap();
        }
    }
}

// 打印命令提示符
pub fn print_hint(error: bool) {
    // 获取当前目录
    let raw_wd = String::from(current_dir().unwrap().to_str().unwrap());
    let mut wd;
    #[allow(deprecated)]
    // if current_dir().unwrap() == home_dir().unwrap() {
    //     wd = String::from("~");
    // }
    // 获取home路径
    let home = home_dir().unwrap();
    // 判断当前路径前是否是home路径，如果是，进行替换
    if raw_wd.starts_with(home.to_str().unwrap()) {
        // raw_wd.remove(home.to_str().unwrap().len());
        // 将当前路径前的home替换为ie~
        wd = String::from("~");
        wd.push_str(&raw_wd[home.to_str().unwrap().len()..]);
    } else {
        wd = raw_wd;
    }
    // 获取用户的uid
    let uid = get_current_uid();
    // 打印获取用户名称
    let username = String::from(get_user_by_uid(uid).unwrap().name().to_str().unwrap());
    // 打印出命令提示符
    print!("{}@{} {} ", Cyan.paint(&username), Style::new().bold().paint(&wd),
    // 判断前一条命令是否正常结束，如果发生了错误，将提示符的箭头染成红色
        if error {
            Red.paint("->")
        } else {
            White.paint("->")
        }
    );
    // 将缓冲区中的内容flush出来
    io::stdout().flush().unwrap();
}

// 原本准备用于创建tui，但是如果用这种方式操作tui程序，如vim，会有一些打印上的问题
// 所以最后弃用了，当需要从stdin输入时，直接初始化stdin实例进行读入
// pub fn takeover_stdin(tx: Sender<u8>) {
//     let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work 
//     let mut termios = Termios::from_fd(stdin).unwrap();
//     tcsetattr(stdin, TCSANOW, &mut termios).unwrap();
//     termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
//     loop {
//                     // on /dev/stdin or /dev/tty
//         // let mut new_termios = termios.clone();  // make a mutable copy of termios 
//                                                 // that we will modify
//         let mut reader = io::stdin();
//         let mut buffer = [0;1];  // read exactly one byte
//         // print!("Hit a key! ");
//         // stdout.lock().flush().unwrap();
//         match reader.read_exact(&mut buffer) {
//             Ok(_) => (),
//             Err(_) => break,
//         }
//         // println!("You have hit: {:?}", buffer);
//         match tx.send(buffer[0]) {
//             Ok(_) => (),
//             Err(_) => break,
//         }
//     }
//     tcsetattr(stdin, TCSANOW, & termios).unwrap();
//     termios.c_lflag |= (ICANON | ECHO); // no echo and canonical mode
//     println!("input breaking");
// }