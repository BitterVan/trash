// 这个文件中是较为复杂的内置命令操作
use std::env::set_current_dir;
use std::io::stderr;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::fs::FileType;
use std::io;
use std::fs;
// use std::sync::mpsc::Receiver;
use std::thread;
use crate::bases::*;
use users::{get_group_by_gid, get_user_by_uid};
use std::os::linux::fs::MetadataExt;

extern crate ctrlc;

// cd命令
pub fn cd(dir: &Path) -> Result<(), String> {
    match set_current_dir(dir) {
        Ok(_) => Ok(()),
        // 如果发生错误，就返回错误信息
        Err(info) => Err(info.to_string()),
    }
}

// 如果是执行一个程序
pub fn prog(prog_name: &str, args: Vec<&str>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> bool {
    // 这个是程序是否执行错误，也就是程序的返回值
    let ret = false;
    // ctrlc::set_handler(user_handler)
    // 先建立Command，等待spwan
    let mut res = Command::new(prog_name);
    res.args(args);

    res.stdout(Stdio::piped());
    res.stderr(Stdio::piped());

    // 同样是为了解决所有权的问题，进行复制
    let output_bak = output_dst.clone();
    let error_bak = error_dst.clone();

    // 根据输入的方式判断是否需要将输入截获后重新输入
    match input_src {
        InputSrc::Channel(_) => {
            // 如果是，那么就需要截获输入
            res.stdin(Stdio::piped());
        },
        InputSrc::File(_) => {
            res.stdin(Stdio::piped());
        },
        InputSrc::Stdin => {

        }
    }

    // 查看所需要执行的外部命令是否存在
    let child_thread = match res.spawn() {
        Ok(child) => child,
        Err(_) => {
            stderr().write("Invalid program name\n".as_bytes()).unwrap();
            return true;
        }
    };

    // 创建一个子进程用于向控制台进行字符输出
    let output;
    let output_thread;
    // 同样地，需要根据输出的类型进行匹配
    match output_dst {
        // 如果是通过管道
        OutputDst::Channel(tx) => {
            output = BufReader::new(child_thread.stdout.unwrap());
            output_thread = thread::spawn(
                // 直接开启另一个线程进行捕获
                move | | {
                    output.bytes()
                       .filter_map(|byte| byte.ok())
                       // 这里的foreach，是自动在另一个线程上进行的
                       .for_each(|byte| tx.send(byte).unwrap())
                }
            )
        },
        // 如果是从文件输出
        OutputDst::File(file_name, append) => {
            output = BufReader::new(child_thread.stdout.unwrap());
            // 如果不是append，就新建文件
            if !append {
                std::fs::File::create(file_name.clone()).unwrap();
            }
            // 根据需求打开文件
            let file = std::fs::OpenOptions::new().write(true)
                                                                     .append(append)
                                                                     .open(file_name)
                                                                     .ok();

            // 检测是否成功打开濑文件
            let mut file = match file {
                Some(file) => file,
                None => {
                    println!("No such file or directory");
                    return true;
                }
            };
            // 打开新线程来写入内容
            output_thread = thread::spawn(
                move | | {
                    output.bytes()
                       .filter_map(|byte| byte.ok())
                       // 同样是直接新线程，一收到输出内容就直接写入文件
                       .for_each(
                           |byte| {
                            file.write(&[byte]).unwrap();
                            // 这里无需重新flush，输出本身就是无阻塞的
                           }
                        )
                }
            )
        },
        // 这里其实无需截获，知识为了保证在线程join的过程中，这个函数不会被终止
        OutputDst::Stdout => {
            output = BufReader::new(child_thread.stdout.unwrap());
            let mut temp_stdout = io::stdout();
            output_thread = thread::spawn(
                move | | {
                    // 和上面完全一致，不同的是是这里是输出到stdout
                    output.bytes()
                       .filter_map(|byte| byte.ok())
                       .for_each(|byte| {
                        temp_stdout.write(&[byte]).unwrap();
                        // stdout是需要进行flush的
                        temp_stdout.flush().unwrap();
                       })
                }
            )
        },
    }

    // 对error进行相同的操作
    let error;
    // 也需要新建线程
    let error_thread;
    match error_dst {
        // 根据需要输出错误的位置进行分类
        // 方式和out的部分是完全一致的，不在重复注释
        ErrorDst::Channel(tx) => {
            error = BufReader::new(child_thread.stderr.unwrap());
            error_thread = thread::spawn(
                move | | {
                    error.bytes()
                       .filter_map(|byte| byte.ok())
                       .for_each(|byte| tx.clone().send(byte).unwrap())
                }
            )
        },
        ErrorDst::File(file_name, append) => {
            error = BufReader::new(child_thread.stderr.unwrap());
            if !append {
                std::fs::File::create(file_name.clone()).unwrap();
            }
            let file = std::fs::OpenOptions::new().write(true)
                                                                     .append(append)
                                                                     .open(file_name)
                                                                     .ok();

            let mut file = match file {
                Some(file) => file,
                None => {
                    println!("No such file or directory");
                    return true;
                }
            };
            error_thread = thread::spawn(
                move | | {
                    error.bytes()
                       .filter_map(|byte| byte.ok())
                       .for_each(
                           |byte| {
                            file.write(&[byte]).unwrap();
                           }
                        )
                }
            )
        },
        ErrorDst::Stderr => {
            error = BufReader::new(child_thread.stderr.unwrap());
            let mut temp_stderr = io::stderr();
            error_thread = thread::spawn(move | | ({
                    error.bytes()
                       .filter_map(|byte| byte.ok())
                       .for_each(|byte| {
                        temp_stderr.write(&[byte]).unwrap();
                        temp_stderr.flush().unwrap();
                       });
                    })
                )
        }
    }

    // let mut a;
    // let _a;
    // 最后调整输入方式
    match input_src {
        // 如果是通过前一个管道进行
        InputSrc::Channel(rx) => {
            // 将stdin取出
            let mut input = child_thread.stdin.unwrap();
            // 取出前一个管道
            let mut buf = Vec::new();
            loop {
                let i = rx.recv().unwrap();
                if i == 0 {
                    break;
                }
                // println!("{}", i);
                buf.push(i);
            }
            // 将管道中的内容全部写入stdin中
            input.write(&buf).unwrap();
            
            // 原本准备做异步写入，但是需要调用更多的unsafe api，最后没有执行
            // _a = thread::spawn( move | |
            // loop {
            //     for i in rx.recv() {
            //         // input.flush().unwrap();
            //         buf.push(i);
            //         println!("{}", i);
            //     }
            //     input.write(&buf).unwrap();
            //     input.flush().unwrap();
            // } );
        },
        InputSrc::File(file_name) => {
            // let mut file = fs::OpenOptions::new()
            //                                 .open(file_name.trim())
            //                                 .unwrap();
            // 取出新线程的stdin
            let mut input = child_thread.stdin.unwrap();
            let buf = fs::read(file_name).unwrap();
            // let mut buf = Vec::new();
            // file.read(&mut buf).unwrap();
            // 将文件中的内容全部直接写入
            input.write_all(&buf).unwrap();
            // 需要flush，否则最后一行的内容无法进入输入
            input.flush().unwrap();
        },
        InputSrc::Stdin => {
            // 这种情况下就直接从stdin读取，以缉拿小系统开销
        }
    }
    output_thread.join().unwrap();
    error_thread.join().unwrap();
    match output_bak {
        OutputDst::Channel(tx) => {
            tx.send(0).unwrap();
        },
        _ => (),
    }
    match error_bak {
        ErrorDst::Channel(tx) => {
            tx.send(0).unwrap();
        },
        _ => (),
    }
    ret
}

// 引入给字符串进行染色的工具
use ansi_term::Colour::Cyan;
use ansi_term::Colour::Blue;
use ansi_term::Colour::White;

// 这里是两个dir的参数，可以选择-a打印隐藏文件，活着-l以长模式打印
pub enum DirOption {
    Long,
    All,
}

// 实现dir
pub fn dir(dir: &Path, options: &Vec<DirOption>) -> io::Result<Vec<String>> {
    // 首先直接读取所有路径下的entries
    let mut entries = fs::read_dir(dir)?
        .map(|res| {
            // 对entry进行map，获取到可以供打印的信息
            let res = res.unwrap();
            // 文件名，文件类型以及元数据组成元祖
            (res.file_name(), res.file_type().unwrap(), res.metadata().unwrap())     
        })
        .collect::<Vec<_>>();

    // 根据文件名对entry进行排序
    entries.sort_by(|a, b| {
        a.0.cmp(&b.0)
    });

    // 判断dir的选项
    let mut opt_long = false;
    let mut opt_all = false;
    for option in options {
        match option {
            // 如果包含了-a
            &DirOption::All => {
                opt_all = true;
            },
            // 如果包含了 -l
            &DirOption::Long => {
                opt_long = true;
            },
        }
    }

    if opt_all {
        // 如果是all，就什么都不用做了
        // do nothing
    } else {
        // 如果不需要打印隐藏文件，就需要删去以.开头的文件了
        entries = entries.into_iter()
                         .filter(|x| !String::from(x.0.to_str().unwrap()).starts_with("."))
                         .collect();
    }

    // let ret;
    // 需要返回的内容
    let ret: Vec<String>;

    // 如果是长打印
    if opt_long {
        ret = entries.into_iter().map(
            |x| {
                // 这里是获取user和group
                // 获取uid
                let uid = x.2.st_uid();
                // 通过uid获取user name
                let user = String::from(get_user_by_uid(uid).unwrap().name().to_str().unwrap());
                // 获取gui
                let gid = x.2.st_gid();
                // 通过gid获取group name
                let group = String::from(get_group_by_gid(gid).unwrap().name().to_str().unwrap());
                // 得到filetype
                let temp = x.1.clone();
                // 打印出一个文件的需打印的信息
                format!("{}{} {} {} {} {}\n", 
                    type_trans(x.1),    // 文件类型
                    mod_trans(x.2.st_mode()),// 文件权限
                    x.2.st_nlink(),
                    user, //user name
                    group, // group name
                    if temp.is_dir() {
                        // 如果是目录，染成蓝色
                        Blue.paint(x.0.to_str().unwrap())
                    } else if temp.is_symlink() {
                        // 如果是链接，染成红色
                        Cyan.paint(x.0.to_str().unwrap())
                    } else {
                        // 如果是普通文件，染成白色
                        White.paint(x.0.to_str().unwrap())
                    }
                )
            }
        ).collect(); 
    } else {
        ret = entries.into_iter().map(
            |x| {
                // 这里一样的染色方式，只要文件名
                if x.1.is_dir() {
                    format!("{} ", Blue.paint(x.0.to_str().unwrap()))
                } else if x.1.is_symlink() {
                    format!("{} ", Cyan.paint(x.0.to_str().unwrap()))
                } else {
                    format!("{} ", x.0.to_str().unwrap())
                }
                // match x.1 {
                    
                // }
            }
        ).collect();
    }
    // 将最后的内容返回即可
    let mut ret_string = String::new();
    for i in ret {
        ret_string.push_str(&i);
    }
    if opt_long {
        // 要去掉最后一个回车，否则会换行两次
        ret_string.pop();
    }
    let mut ret = Vec::new();
    ret.push(ret_string);
    ret.push(String::new());
    Ok(ret)
}

// 将filetype转换成char，也就是权限前的一位
fn type_trans(file_type: FileType) -> char {
    let file_type = if file_type.is_dir() {
        'd'
    } else if file_type.is_symlink() {
        'l'
    } else {
        '-'
    };
    file_type
}

// 将返回的32位unsign权限转化成可供打印的样子
fn mod_trans(val: u32) -> String {
    let mut ret = String::new();
    let mut val = val % 512;
    // 每次都取最后一位
    for _ in 0..3 {
        // 依次是xwr
        if val % 2 == 1 {
            ret.push('x')
        } else {
            ret.push('-')
        }
        val /= 2;
        if val % 2 == 1 {
            ret.push('w')
        } else {
            ret.push('-')
        }
        val /= 2;
        if val % 2 == 1 {
            ret.push('r')
        } else {
            ret.push('-')
        }
        val /= 2;
    }
    // 由于每次都把新加入的部分放在最后，需要倒序
    ret.chars().rev().collect()
}