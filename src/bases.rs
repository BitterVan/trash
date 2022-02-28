// 这个文件中包含了一些基本的定义
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
extern crate pest;

// 用于重定向时确认输入方式的枚举类
pub enum InputSrc {
    // 通过文件输入（提供文件名）
    File(String),
    // 通过管道输入
    Channel(Receiver<u8>),
    // 通过标准输入来进行输入
    Stdin,
}

// 用于重定向时确认输出方式的枚举类
#[derive(Debug)]
pub enum OutputDst {
    // 文件输出（提供文件名和是否覆盖）
    File(String, bool),
    // 通过管道输出
    Channel(Sender<u8>),
    // 通过标准输出进行输出
    Stdout,
}

// 手动实现OutputDst的Clone trait，类似cpp的拷贝构造
impl Clone for OutputDst {
    fn clone(&self) -> Self {
        // 按照输出类型，复制好内部内容
        match &self {
            &Self::File(file_name, append) => {
                OutputDst::File(file_name.clone(), append.clone())
            },
            &Self::Channel(rx) => {
                OutputDst::Channel(rx.clone())
            },
            &Self::Stdout => {
                OutputDst::Stdout
            },
        }
    }
}

// 和OutputDst的功能实现基本一致
pub enum ErrorDst {
    // 文件输出（提供文件名和是否覆盖）
    File(String, bool),
    // 通过管道输出
    Channel(Sender<u8>),
    // 通过标准输出进行输出
    Stderr,
}

// 实现Clone trait
impl Clone for ErrorDst {
    fn clone(&self) -> Self {
        match &self {
            &Self::File(file_name, append) => {
                ErrorDst::File(file_name.clone(), append.clone())
            },
            &Self::Channel(rx) => {
                ErrorDst::Channel(rx.clone())
            },
            &Self::Stderr => {
                ErrorDst::Stderr
            },
        }
    }
}

// 用于控制主循环的信号
pub enum MainCtrl {
    // 退出
    Exit,
    // 切换执行
    Exec,
    // 后台运行
    Back,
    // 程序出错
    Error,
    // 显示进程信息
    Jobs,
    // 杀死进程
    Kill(usize),
    // 将后台运行的程序切换至前台运行
    Fg(usize),
    // 将挂起的程序继续执行
    Bg(usize),
}