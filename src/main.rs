#[macro_use]
extern crate pest_derive;
extern crate pest;
extern crate ctrlc;

mod bases;
mod mixios;
mod interprete;
mod buildins;
mod doc;
use std::{io::{self, Write, stderr}, process::Command};

use bases::*;
use mixios::*;
use interprete::*;
use pest::Parser;
// use doc::DOCMUENT;
// use buildins::*;
#[derive(Parser)]
#[grammar = "trash.pest"]
pub struct CmdParser;

use ansi_term::Colour::Red;
use std::env;


fn main() {
    // 获取所有命令行参数
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    // 用于存储所有的后台进程
    let mut children = Vec::new();
    // 用于保存上一条命令的执行结果
    let mut status = false;
    // 建立通信管道，给ctrlc发送运行结果的能力
    let (tx, rx) = std::sync::mpsc::channel();
    // 用于检测ctrlc并且挂起所有任务
    ctrlc::set_handler( move | | (tx.send(true).unwrap())).unwrap();
    let shell = String::from(std::env::current_exe().unwrap().to_str().unwrap());
    // 设置所需的两个环境变量
    std::env::set_var("shell", shell);
    std::env::set_var("parent", "trash");
    // 如果是直接执行，而非执行某个脚本
    if args.len() == 1 {
        loop {
            // 检测ctrlc的结果
            // 这里其实是多线程，不断接受ctrlc返回结果
            match rx.try_recv() {
                Ok(_) => {status = true;},
                Err(_) => (),
            }
            // 打印出提示符
            print_hint(status);
            status = false;
            // 将命令复制三分（这是因为rust的特性，进入函数后会失去数据所有权）
            let cmd = input_cmd().unwrap();
            let bak = cmd.clone();
            let bakk = bak.clone();
            // 获取执行后，返回的对main的控制信号
            match interprete(cmd) {
                Ok(ctrl) => match ctrl {
                    Some(ctrl) => match ctrl {
                        MainCtrl::Exit => break,
                        MainCtrl::Exec => break,
                        // 如果是需要进行后台运行，就要额外创建子线程
                        MainCtrl::Back => {
                            // 重新解释命令
                            let mut prog_line = CmdParser::parse(Rule::back_line, &bak).unwrap();
                            let prog_line = prog_line.next().unwrap().into_inner().next().unwrap();//.into_inner().next().unwrap();
                            // 获取程序名称
                            let mut prog_name = "";
                            // 获取所有参数
                            let mut args = Vec::new();
                            // 打印程序信息
                            let mut prog_info = prog_line.into_inner();
                            loop {
                                match prog_info.next() {
                                    Some(thing) => {
                                        match thing.as_rule() {
                                            Rule::prog_args => args.push(thing.as_str()),
                                            Rule::prog_name => prog_name = thing.as_str(),
                                            _ => (),
                                        }
                                    },
                                    None => break,
                                }
                            }
                            // 打印出新建的后台进程的提示信息
                            println!("{} -> command: {}", children.len(), bak.clone().trim());
                            // 添加到子进程列表
                            children.push((bakk, Command::new(prog_name).args(args).spawn().unwrap()));
                        },
                        // 如果有错误，那就将状态设置为true
                        MainCtrl::Error => status = true,
                        MainCtrl::Jobs => {
                            // 打印出子进程列表
                            for i in 0..children.len() {
                                println!("{} -> command: {}", i, children[i].0.trim());
                            }
                        },
                        // 如果要杀死进程i
                        MainCtrl::Kill(i) => {
                            if children.len() > i {
                                // 直接杀死
                                match children[i].1.kill() {
                                    // 打印出提示信息，并用红色进行警示
                                    Ok(_) => {
                                        println!("{} {} {} {}", i, Red.paint("->"),  Red.paint("killed: "), children[i].0.trim());
                                    },
                                    Err(_) => {
                                        println!("{} {} {} {}", i, Red.paint("Have exited"),  Red.paint("command:"), children[i].0.trim());
                                    }
                                }
                                children.remove(i);
                            }
                        },
                        // 如果要将后台进程切换到前台，其实只需要等待其结束
                        MainCtrl::Fg(i) => {
                            if children.len() > i {
                                // 切换为等待进程i运行到结束
                                children[i].1.wait().unwrap();
                                children.remove(i);
                            }
                        }
                        // 后台运行挂起的程序
                        MainCtrl::Bg(i) => {
                            if children.len() > i {
                                // 因为rust的特性，复制字符串
                                let bak = String::from(children[i].0.trim());
                                let bakk = bak.clone();
                                // 重新解释字符串
                                let mut prog_line = CmdParser::parse(Rule::back_line, &bak).unwrap();
                                let prog_line = prog_line.next().unwrap().into_inner().next().unwrap();//.into_inner().next().unwrap();
                                let mut prog_name = "";
                                let mut args = Vec::new();
                                // 重新获取程序的运行信息
                                let mut prog_info = prog_line.into_inner();
                                loop {
                                    // 传入参数
                                    match prog_info.next() {
                                        Some(thing) => {
                                            match thing.as_rule() {
                                                Rule::prog_args => args.push(thing.as_str()),
                                                Rule::prog_name => prog_name = thing.as_str(),
                                                _ => (),
                                            }
                                        },
                                        None => break,
                                    }
                                }
                                // 如果不在进程表中，就无视指令
                                match children[i].1.kill() {
                                    _=> (),
                                }
                                // 删去进程表中原先的进程
                                children.remove(i);
                                // 打印提示信息
                                println!("{} -> command: {}", children.len(), bak.clone().trim());
                                //将进程加入进程表
                                children.push((bakk, Command::new(prog_name).args(args).spawn().unwrap()));
                            }
                        }
                        // _ => (),
                    }
                    None => ()
                }
                Err(info) => {
                    // 打印出错误信息
                    io::stderr().write(info.as_bytes()).unwrap();
                    // 进行换行
                    io::stderr().write(&[10]).unwrap();
                }
            }
        }
    } else {
        // 这种情况下，是需要进行脚本执行的情况
        let file_name = args[1].clone();
        // 赋值环境变量
        for i in 0..args.len() {
            std::env::set_var(i.to_string(), args[i].clone());
        }
        let bak = file_name.clone();
        // 尝试读取文件
        let file = match std::fs::read(file_name) {
            Ok(file) => {
                String::from_utf8(file).unwrap()
            },
            // 如果文件不存在，就向stderr写入错误
            // 当然，被当作程序执行的话，stderr会被截获并且进行重定向
            Err(_) => {
                stderr().write(format!("No such file or directory {}\n", bak).as_bytes()).unwrap();       
                return;
            }
        };
        let file = file.trim().split("\n");
        // 依次解释执行所有文件中的命令
        for cmd in file {
            // 将命令复制三分（这是因为rust的特性，进入函数后会失去数据所有权）
            let cmd = String::from(cmd);
            let bak = cmd.clone();
            let bakk = bak.clone();
            // 接下来的控制部分和直接输入命令是一致的
            match interprete(cmd) {
                Ok(ctrl) => match ctrl {
                    Some(ctrl) => match ctrl {
                        MainCtrl::Exit => break,
                        MainCtrl::Exec => break,
                        // 如果是需要进行后台运行，就要额外创建子线程
                        MainCtrl::Back => {
                            // 重新解释命令
                            let mut prog_line = CmdParser::parse(Rule::back_line, &bak).unwrap();
                            let prog_line = prog_line.next().unwrap().into_inner().next().unwrap();//.into_inner().next().unwrap();
                            // 获取程序名称
                            let mut prog_name = "";
                            // 获取所有参数
                            let mut args = Vec::new();
                            // 打印程序信息
                            let mut prog_info = prog_line.into_inner();
                            loop {
                                match prog_info.next() {
                                    Some(thing) => {
                                        match thing.as_rule() {
                                            Rule::prog_args => args.push(thing.as_str()),
                                            Rule::prog_name => prog_name = thing.as_str(),
                                            _ => (),
                                        }
                                    },
                                    None => break,
                                }
                            }
                            // 打印出新建的后台进程的提示信息
                            println!("{} -> command: {}", children.len(), bak.clone().trim());
                            // 添加到子进程列表
                            children.push((bakk, Command::new(prog_name).args(args).spawn().unwrap()));
                        },
                        // 如果有错误，那就将状态设置为true
                        MainCtrl::Error => (),
                        MainCtrl::Jobs => {
                            // 打印出子进程列表
                            for i in 0..children.len() {
                                println!("{} -> command: {}", i, children[i].0.trim());
                            }
                        },
                        // 如果要杀死进程i
                        MainCtrl::Kill(i) => {
                            if children.len() > i {
                                // 直接杀死
                                match children[i].1.kill() {
                                    // 打印出提示信息，并用红色进行警示
                                    Ok(_) => {
                                        println!("{} {} {} {}", i, Red.paint("->"),  Red.paint("killed: "), children[i].0.trim());
                                    },
                                    Err(_) => {
                                        println!("{} {} {} {}", i, Red.paint("Have exited"),  Red.paint("command:"), children[i].0.trim());
                                    }
                                }
                                children.remove(i);
                            }
                        },
                        // 如果要将后台进程切换到前台，其实只需要等待其结束
                        MainCtrl::Fg(i) => {
                            if children.len() > i {
                                // 切换为等待进程i运行到结束
                                children[i].1.wait().unwrap();
                                children.remove(i);
                            }
                        }
                        // 后台运行挂起的程序
                        MainCtrl::Bg(i) => {
                            if children.len() > i {
                                // 因为rust的特性，复制字符串
                                let bak = String::from(children[i].0.trim());
                                let bakk = bak.clone();
                                // 重新解释字符串
                                let mut prog_line = CmdParser::parse(Rule::back_line, &bak).unwrap();
                                let prog_line = prog_line.next().unwrap().into_inner().next().unwrap();//.into_inner().next().unwrap();
                                let mut prog_name = "";
                                let mut args = Vec::new();
                                // 重新获取程序的运行信息
                                let mut prog_info = prog_line.into_inner();
                                loop {
                                    // 传入参数
                                    match prog_info.next() {
                                        Some(thing) => {
                                            match thing.as_rule() {
                                                Rule::prog_args => args.push(thing.as_str()),
                                                Rule::prog_name => prog_name = thing.as_str(),
                                                _ => (),
                                            }
                                        },
                                        None => break,
                                    }
                                }
                                // 如果不在进程表中，就无视指令
                                match children[i].1.kill() {
                                    _=> (),
                                }
                                // 删去进程表中原先的进程
                                children.remove(i);
                                // 打印提示信息
                                println!("{} -> command: {}", children.len(), bak.clone().trim());
                                //将进程加入进程表
                                children.push((bakk, Command::new(prog_name).args(args).spawn().unwrap()));
                            }
                        }
                        // _ => (),
                    }
                    None => ()
                }
                Err(info) => {
                    // 如果检测到了错误，就要输出错误信息
                    io::stderr().write(info.as_bytes()).unwrap();
                    // 补充输出空格
                    io::stderr().write(&[10]).unwrap();
                }
            }
        }
    }
}

// 这两个函数是由于没有看清需求，用这个可以获取当前trash shell的parent
// fn get_pid_name(pid: u32) -> String {
//     let ret = std::process::Command::new("ps")
//         .arg("-p")
//         .arg(format!("{}", pid))
//         .arg("-o")
//         .arg("comm=")
//         .output();

//     String::from_utf8_lossy(&ret.unwrap().stdout).to_string()
// }

// fn get_parent_pid(pid: u32) -> String {
//     let ret = std::process::Command::new("ps")
//         .arg("-o")
//         .arg(format!("ppid={}", pid))
//         .output();

//     let output = String::from_utf8_lossy(&ret.unwrap().stdout).to_string();
//     let output = output.trim().split("\n");
//     let mut output_vec = Vec::new();
//     for i in output {
//         output_vec.push(i);
//     }
//     get_pid_name(output_vec[output_vec.len()-2].trim().parse::<u32>().unwrap())
// }