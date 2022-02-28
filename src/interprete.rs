extern crate pest;
use libc::umask;
// use pest::error::InputLocation;
use crate::doc::USER_DOCUMENT;
use std::io::Write;
use std::sync::mpsc::channel;
use std::path::Path;
use pest::Parser;
use pest::iterators::Pair;
use ansi_term::Style;
use std::env::current_dir;

#[derive(Parser)]
#[grammar = "trash.pest"]
pub struct CmdParser;

use crate::bases::*;
use crate::buildins::*;
use crate::mixios::*;

// 第一层解释，将字符串转换成pair
pub fn interprete(cmd: String) -> Result<Option<MainCtrl>, String> {
    // 进行第一轮parse
    let res = match CmdParser::parse(Rule::line, &cmd) {
        Ok(mut res) => res.next().unwrap().into_inner().next().unwrap(),
        // 如果不符合定义的规则之一，直接返回错误
        Err(info) => return Err(info.to_string()),
    };
    // 如果成功了，就进行第二轮解释
    let res = single_interprete(res, InputSrc::Stdin, OutputDst::Stdout, ErrorDst::Stderr);
    Ok(res)
}

// 这里是用于解释子语句
// 比如重定向和管道中
fn sub_interprete(line: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> bool {
    let line = line.into_inner().next().unwrap();
    // 再次调用单条语句的解释
    match single_interprete(line, input_src, output_dst, error_dst) {
        Some(info) => {
            match info {
                // 如果错误，需要向main返回错误控制信息
                MainCtrl::Error => true,
                _ => false,
            }
        },
        None => false
    }
}

fn single_interprete(line: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> Option<MainCtrl> {
    // println!("{:#?}", line);
    // 对应的语句需要进行对应的解释，这里的命名方式是足够清楚的
    match line.as_rule() {
        Rule::assign_line => assign_interprete(line, input_src, output_dst, error_dst),
        Rule::dir_line => if dir_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        }
        Rule::pipe_line => if pipe_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        }
        Rule::pwd_line => pwd_interprete(line, input_src, output_dst, error_dst),
        Rule::cd_line => if cd_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        },
        Rule::clr_line => clr_interprete(line, input_src, output_dst, error_dst),
        Rule::prog_line => if prog_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        },
        Rule::job_line => {
            return Some(MainCtrl::Jobs);
        },
        Rule::kill_line => {
            return Some(MainCtrl::Kill(line.into_inner().next().unwrap().as_str().parse().unwrap()))
        },
        Rule::fg_line => {
            return Some(MainCtrl::Fg(line.into_inner().next().unwrap().as_str().parse().unwrap()))
        },
        Rule::bg_line => {
            return Some(MainCtrl::Bg(line.into_inner().next().unwrap().as_str().parse().unwrap()))
        },
        Rule::unset_line => {
            let param_name = line.into_inner().next().unwrap();
            std::env::remove_var(param_name.as_str().trim());
        }
        Rule::back_line => return Some(MainCtrl::Back),
        Rule::exit_line => return Some(MainCtrl::Exit),
        Rule::exec_line => {
            exec_interprete(line, input_src, output_dst, error_dst);
            return Some(MainCtrl::Exec);
        },
        Rule::time_line => {
            time_interprete(line, input_src, output_dst, error_dst);
        }
        Rule::redir_line => if redir_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        },
        Rule::echo_line => echo_interprete(line, input_src, output_dst, error_dst),
        Rule::set_line => set_interprete(line, input_src, output_dst, error_dst),
        Rule::shift_line => shift_interprete(line, input_src, output_dst, error_dst),
        Rule::umask_line => umask_interprete(line, input_src, output_dst, error_dst),
        Rule::help_line => help_interprete(line, input_src, output_dst, error_dst),
        Rule::test_line => test_interprete(line, input_src, output_dst, error_dst),
        Rule::umask_set_line => if umask_set_interprete(line, input_src, output_dst, error_dst) {
            return Some(MainCtrl::Error);
        }
        _ => (),
        // _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "undefined command")),
    };
    None
}


// 对test语句进行解释
fn test_interprete(line: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst)  {
    match line.into_inner().next() {
        Some(line) => {
            match line.as_rule() {
                // 如果是test字符长度是不是为0的语句
                Rule::test_zero => {
                    match test_zero(line) {
                        // 按照是否为真进行输出
                        true => write_out(String::from("true\n"), output_dst),
                        false => write_out(String::from("false\n"), output_dst)
                    }
                },
                // 如果是判断两个字符串是否相等
                Rule::test_equal => {
                    match test_equal(line) {
                        // 按照是否为真进行输出
                        true => write_out(String::from("true\n"), output_dst),
                        false => write_out(String::from("false\n"), output_dst)
                    }
                },
                _ => (),
            }
        },
        None => (),
    }
}

// 判断字符串长度是不是0
fn test_zero(line: Pair<Rule>) -> bool {
    let mut line = line.into_inner();
    let arg = line.next().unwrap().as_str().trim();
    let cand = line.next().unwrap();
    // 由于需要比较的部分可能含有变量，是需要先取值再比较的
    let cand = tran_cand(cand);
    match arg {
        "-z" => return cand.len() == 0,
        "-n" => return cand.len() != 0,
        _ => (),
    }
    false
}

fn test_equal(line: Pair<Rule>) -> bool {
    let mut line = line.into_inner();
    // 需要对两个字符串中的变量都首先进行求值
    let cand1 = line.next().unwrap();
    let arg = line.next().unwrap();
    let cand2 = line.next().unwrap();
    // 进行转换
    let cand1 = tran_cand(cand1);
    let cand2 = tran_cand(cand2);
    match arg.as_str() {
        // 判断是需要判断相等还是不等
        "=" => cand1.trim() == cand2.trim(),
        "!=" => cand1.trim() != cand2,
        _ => false,
    } 
}

// 这个函数的将变量名称转化为变量的值
fn tran_cand(cand: Pair<Rule>) -> String {
    let mut whole = String::new();
    // 将一个字符串中的所有部分提取出来逐个转换
    let i = cand.into_inner();
    for i in i {
        match i.as_rule() {
            Rule::parameter_wrapped => {
                // 如果是变量
                let i = i.into_inner().next().unwrap();
                // 检测环境变量中是否包含了该变量
                match std::env::var(i.as_str().trim()) {
                    Ok(val) => {
                        // 如果有，那就加入值
                        whole.push_str(&val);
                    },
                    Err(_) => {
                        // 没有的话，视为空字符串
                    }
                }
            },
            // 如果是普通的字符串，那直接加入就行
            Rule::dynamic_unwrapped => {
                whole.push_str(i.as_str());
            },
            _ => (),
        }
    }

    whole
}

// 解释shift命令
fn shift_interprete(line: Pair<Rule>, _: InputSrc, _: OutputDst, _: ErrorDst) {
    let mut i = 0;
    // 获取shift的参数
    let total: u32 = line.as_str()[5..].trim().parse().unwrap();
    loop {
        // 将所有参数逐个移动
        match std::env::var((i+total).to_string()) {
            Ok(val) => {
                std::env::set_var(i.to_string(), val);
            },
            // 如果遇到了一个最大的，没有对应位置参数的参数，就跳出循环
            Err(_) => {
                break;
            },
        }
        // 每次增加1
        i += 1;
    }
}

// 解释help
fn help_interprete(_: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) {
    // 将文档写入临时文件
    std::fs::write(".trash_doc", USER_DOCUMENT.as_bytes()).unwrap();
    // 解释一个more语句
    // 但是由于我截获了输出，导致more输出的不是终端，所以这里不起作用
    let line = CmdParser::parse(Rule::prog_line, "more .trash_doc").unwrap().next().unwrap();
    // 解释more命令
    prog_interprete(line, input_src, output_dst, error_dst);
    // 删除临时文件
    std::fs::remove_file(".trash_doc").unwrap();
}

// 解释赋值命令
fn assign_interprete(line: Pair<Rule>, _:InputSrc, _: OutputDst, _: ErrorDst) {
    let mut line = line.into_inner();
    // 取出变量名称
    let var_name = line.next().unwrap().as_str().trim();
    let mut whole = String::new();
    for i in line {
        match i.as_rule() {
            Rule::dynamic_word => {
                // 变量是可以被另一个变量的值赋值的，这里需要现行解释赋值的内容
                let i = i.into_inner();
                for i in i {
                    match i.as_rule() {
                        // 如果是变量
                        Rule::parameter_wrapped => {
                            let i = i.into_inner().next().unwrap();
                            // 尝试获取环境变量，这里和前面解释变量的过程是一致的
                            match std::env::var(i.as_str().trim()) {
                                Ok(val) => {
                                    whole.push_str(&val);
                                },
                                Err(_) => {

                                }
                            }
                        },
                        // 如果是普通的变量
                        Rule::dynamic_unwrapped => {
                            whole.push_str(i.as_str());
                        },
                        _ => (),
                    }
                }
                // 补充空格
                whole.push(' ');
           },
           // 也可能是单引号的静态字符串
            Rule::static_word => {
                let to_print = i.as_str().trim();
                whole.push_str(&to_print[1..to_print.len()-1]);
                whole.push_str(" ");
            },
            _ => (),
        }
    }
    // 设置变量
    std::env::set_var(var_name, whole)
}

// 打印出所有环境变量
fn set_interprete(_: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst) {
    let mut out = String::new();
    // 直接对迭代器进行迭代
    for (key, value) in std::env::vars() {
        // 进行语句拼接
        out.push_str(&format!("{} => {}\n", Style::new().bold().paint(&key), value));
    }
    write_out(out, output_dst);
}

// 解释echo命令
fn echo_interprete(line: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst) {
    let line = line.into_inner();
    let mut whole = String::new();
    for i in line {
        // 这部分和assign和test部分的解释变量部分是一致的
        match i.as_rule() {
            Rule::dynamic_word => {
                let i = i.into_inner();
                for i in i {
                    match i.as_rule() {
                        Rule::parameter_wrapped => {
                            let i = i.into_inner().next().unwrap();
                            // println!("{:#?}", i);
                            match std::env::var(i.as_str().trim()) {
                                Ok(val) => {
                                    whole.push_str(&val);
                                },
                                Err(_) => {
                                }
                            }
                        },
                        Rule::dynamic_unwrapped => {
                            whole.push_str(i.as_str());
                        },
                        _ => (),
                    }
                }
                whole.push(' ');
           },
            Rule::static_word => {
                let to_print = i.as_str().trim();
                whole.push_str(&to_print[1..to_print.len()-1]);
                whole.push_str(" ");
            },
            _ => (),
        }
    }
    whole.push('\n');
    write_out(whole, output_dst);
}


// 解释umask -S 命令
fn umask_set_interprete(line: Pair<Rule>, _: InputSrc, _: OutputDst, error_dst: ErrorDst) -> bool {
    let num = line.into_inner().next().unwrap().as_str();
    let mut octal_num = String::new();
    // 需要使用八进制前缀进行解释
    octal_num.push_str("0o");
    octal_num.push_str(num.trim());
    use parse_int::parse;
    // 将输入转换成八进制
    let num: u32 = match parse(&octal_num) {
        Ok(num) => num,
        Err(_) => {
            // 如果输入的不是八进制数字
            write_error(String::from("Only octal digit can be put"), error_dst);
            return true;
        }
    };
    // umask被视为不安全的函数
    unsafe {
        umask(num);
    };
    false
}

// 解释umask命令
fn umask_interprete(_: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst) {
    unsafe {
        let origin = umask(0);
        // 获取到当前的mask值
        write_out(String::from(format!("{:03o}\n", origin)), output_dst);
        // 写回原本的mask值
        umask(origin);
    }
}

// 输出时间
fn time_interprete(_: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst) {
    let time = chrono::Local::now();
    // 将其格式化为字符串
    let time = String::from(format!("{}\n", time));
    write_out(time, output_dst)
}

// 如果需要进行重定向
fn redir_interprete(line: Pair<Rule>, mut input_src: InputSrc, mut output_dst: OutputDst, mut error_dst: ErrorDst) -> bool {
    let mut fail = false;
    let mut line = line.into_inner();
    // 获取程序名称
    let prog_line = line.next().unwrap();
    let mut redir_options = Vec::new();
    loop {
        match line.next() {
            Some(opt) => redir_options.push(opt.as_str()),
            None => break,
        }
    }

    // 这里会分别对所有重定向选项进行判断
    for opt in redir_options {
        // 如果是>>
        if opt.starts_with(">>") {
            match std::fs::File::open(opt[3..].trim()) {
                Err(_) => {
                    std::io::stderr().write("No file for stdout redirection\n".as_bytes()).unwrap();
                    return true;
                },
                _ => (),
            };
        
            output_dst = OutputDst::File(String::from(opt[3..].trim()), true);
        // 如果是(1)>>
        } else if opt.starts_with("(1)>>"){
            match std::fs::File::open(opt[5..].trim()) {
                Err(_) => {
                    std::io::stderr().write("No file for stdout redirection\n".as_bytes()).unwrap();
                    return true;
                },
                _ => (),
            };
            output_dst = OutputDst::File(String::from(opt[5..].trim()), true);
        // 如果是>
        }else if opt.starts_with(">") {
            // match std::fs::File::create(opt[2..].trim()) { _ => () };
            output_dst = OutputDst::File(String::from(opt[2..].trim()), false);
        // 如果是(2)>>
        } else if opt.starts_with("(2)>>") {
            match std::fs::File::open(opt[5..].trim()) {
                Err(_) => {
                    std::io::stderr().write("No file for stderr redirection\n".as_bytes()).unwrap();
                    return true;
                },
                _ => (),
            };
            error_dst = ErrorDst::File(String::from(opt[5..].trim()), true);
        // 如果是(1)>
        } else if opt.starts_with("(1)>") {
            output_dst = OutputDst::File(String::from(opt[4..].trim()), false);
        // 如果是(2)>
        } else if opt.starts_with("(2)>") {
            error_dst = ErrorDst::File(String::from(opt[4..].trim()), false);
        // 如果是<
        } else if opt.starts_with("<") {
            match std::fs::File::open(opt[1..].trim()) {
                Err(_) => {
                    std::io::stderr().write("No file for stdin redirection\n".as_bytes()).unwrap();
                    return true;
                },
                _ => (),
            };
            input_src = InputSrc::File(String::from(opt[1..].trim()));
        // 否则就是解释不成功
        } else {
            fail = true;
        }
    }

    // println!("{:?}", output_dst.clone());
    let prog_line = prog_line.as_str();
    // 执行程序
    let prog_line = CmdParser::parse(Rule::line, prog_line).unwrap().next().unwrap();
    // 递归地解释前半个语句
    fail |= sub_interprete(prog_line, input_src, output_dst, error_dst);
    fail
}

// 这里调用buildins里的方法
fn exec_interprete(line: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) {
    let line = line.into_inner().next().unwrap();
    prog_interprete(line, input_src, output_dst, error_dst);
}

// 解释带有管道的命令
fn pipe_interprete(line: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> bool {
            let mut line = line.into_inner();
            let ret = false;
            let mut all = Vec::new();
            // 将所有命令都加入序列
            loop {
                match line.next() {
                    Some(line) => {
                        all.push(line.as_str());
                    },
                    None => {
                        break;
                    },
                }
            }

            // 建立起管道
            let (tx, rx) = channel();
            // 这个函数中只执行依次，其余的被递归地实行
            let last = all[all.len()-1];
            let last = CmdParser::parse(Rule::line, last);
            let last = last.unwrap().next().unwrap();

            let mut origin = String::new();
            for i in 0..all.len()-1 {
                // 这里是将前面的语句拼接起来，重新解释
                origin.push_str(all[i]);
                origin.push_str(" | ");
            }
            let origin = CmdParser::parse(Rule::line, &origin);
            let origin = origin.unwrap().next().unwrap();
            // 对前面的语句进行递归，用来解决多层管道
            sub_interprete(origin, input_src, OutputDst::Channel(tx.clone()), error_dst.clone());
            // 单独执行最后一层的管道
            sub_interprete(last, InputSrc::Channel(rx), output_dst, error_dst);
            // println!("done");
            ret
}

// 预处理parse之后的内容
fn dir_interprete(dir_line: Pair<Rule>, _: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> bool {
    let mut options = Vec::new();
    // 默认的目录就是当前目录
    let mut dir_name = Path::new(".");
    let mut info = dir_line.into_inner();
    loop {
        match info.next() {
            Some(thing) => {
                match thing.as_rule() {
                    // 同样地，这里再获取dir的参数属性
                    Rule::dir_args => {
                        let args = &thing.as_str()[1..];
                        for i in args.chars() {
                            match i {
                                'l' => options.push(DirOption::Long),
                                'a' => options.push(DirOption::All),
                                _ => println!("Undefined arg {}", i), //write_error(String::from(format!("Undefine option {}", i)), error_dst)
                            }
                        }
                    },
                    // 如果最后是目录名称，就要将目标目录改成该目录
                    Rule::dir_name => {
                        dir_name = Path::new(thing.as_str());
                    },
                    _ => {

                    },
                }
            },
            None => {
                break;
            }
        }
    }
    let ret;
    match dir(&dir_name, &options) {
        Ok(outputs) => {
            // 输出返回的内容
            // 这是正常输出的字符串
            let mut all = String::new();
            all.push_str(outputs[0].as_str());
            let mut error = String::new();
            // 这是错误信息
            error.push_str(outputs[1].as_str());
            all.push('\n');
            // 如果有错误信息需要打印
            if error.len() > 0 {
                // 返回结果改为true
                ret = true;
                write_error(error, error_dst);
            } else {
                ret = false;
            }
            // 将输出写出
            write_out(all, output_dst);
            return ret;
        },
        Err(e) => {
            // 有错误的话也许要写出错误信息
            write_error(e.to_string(), error_dst);
            // 将返回结果改为有错误
            return true;
        }
    }

}

fn cd_interprete(line: Pair<Rule>, _: InputSrc, _: OutputDst, error_dst: ErrorDst) -> bool {
    let dir_name = line.into_inner().next().unwrap().as_str();
    match cd(&Path::new(dir_name)) {
        // 如果成功，就不需要返回内容
        Ok(_) => (),
        // 否则写出错误信息
        Err(info) => { 
            write_error(info, error_dst);
            return true;
        }
    }
    false
}

// 打印出当前目录
fn pwd_interprete(_: Pair<Rule>, _: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) {
    // 调用env中的函数
    match current_dir() {
        Ok(path) => {
            // 打印内容
            let mut path_name = String::from(path.to_str().unwrap());
            path_name.push('\n');
            write_out(path_name, output_dst);
        }
        Err(info) => write_error(info.to_string(), error_dst),
    }
}

// 解释clr命令
fn clr_interprete(_: Pair<Rule>, _: InputSrc, output_dst: OutputDst, _: ErrorDst) {
    write_out( String::from("\x1B[2J\x1B[1;1H"), output_dst);
}

// 这里是预处理parse过后的外部程序语句
fn prog_interprete(prog_line: Pair<Rule>, input_src: InputSrc, output_dst: OutputDst, error_dst: ErrorDst) -> bool {
    // 获取外部命令名称
    let mut prog_name = "";
    let mut args = Vec::new();
    let mut prog_info = prog_line.into_inner();
    loop {
        // 获取外部命令参数
        match prog_info.next() {
            Some(thing) => {
                match thing.as_rule() {
                    // 如果是参数
                    Rule::prog_args => args.push(thing.as_str()),
                    // 如果是目录的话，也添加到参数中
                    // rust不区分选项和最后的对象
                    Rule::prog_name => prog_name = thing.as_str(),
                    _ => (),
                }
            },
            None => break,
        }
    }
    prog(prog_name, args, input_src, output_dst, error_dst)
}