
// 这个文件是用户文档
// 文档被定义成一个&str，方便后续操作
pub const USER_DOCUMENT: &str = 

"
This is trash, a shell written in Rust.
License: MIT/Apache-2.0

User Guide:
(In the declaration, ? means the number of args can be 0 or 1, * means it can be any number, + means it have to larger than 0, || means or)

Buildin tools:
    cd                              Change Directory
        cd <path>
        cd Codes/rust                   -- change current directory to Codes/
        cd /home/bittervan/Codes/rust
                                        -- using absolute path
    clr                             CLeaR screen
        clr                             -- make terminal emulator clean

    dir                             DIRectory
        dir <path>?
        dir                             -- print the files in current directory
        dir rust                        -- print the files in directory rust/
        dir -la rust                    -- print the files in directory rust/, using long format and show hidden files
        dir /home/bittervan/Codes/rust
                                        -- using absolute path

    echo
        echo (<dynamic_string> || <static_string>)*
        echo \'${hello} is\' \"${hello}\"    
                                        -- print \"${hello} is <value of hello>\"
        echo \"hello\"                  -- print \"hello\"

    help
        help                            -- using application more to show this document

    pwd                             Print Working Directory
        pwd                             -- print current directory

    set/environ                     show ENVIRONment variable SET
        set || environ                  -- print all environment variables

    test
        test <option> <dynamic_string>
                                        -- test if the dynamic string matched the option
        test -n \"\"                    -- \"\" have zero length, so print false
        test -z \"${PATH}\"             -- the path is quit long and not zero length, so print false
        test <dynamic_string> <=/!=> <dynamic_string>
                                        -- test if two strings are equal
        test \"abc\" = \"abc\"          -- print true
        test \"${abd}\" = \"abd\"       -- test if variable abd is set to abd

    time
        time                            -- print current time

    umask
        umask                           -- print current umask value
        umask -S <permission(octal)>    -- set umask to the input
        umask -S 777                    -- set the umask to 777

    umask                           UNSET enviroment variable
        unset <variable_name>
        unset parent                    -- unset variable parent

Task controls: ** These Commands Cannot Be Redireced **
    bg                              BackGroud execution
        bg <taskid_in_this_shell>       -- continue a hanged process backgroud


    fg                              ForeGround exection
        fg <taskid_in_this_shell>       -- continue a hanged process foreground

    exec                            EXECute
        exec <command>                  -- let current thread be the command rather than shell trash
        exec vim                        -- run vim, and make current shell no longer running

    jobs
        jobs                            -- print all process background and their taskids

    kill
        kill <taskid_in_this_shell>     -- kill a background or hanged process

    To make a program running at background, you can add & behind a program.
    like
        -> sleep 10 &, or
        -> ping baidu.com &

    both of them will conduct program properly, and flow the output to the correct position
    The difference is, the input will be blocked, and you can use other commands at the mean time.

    You can use all the command above to control background programs.
    like using \"jobs\" command, you will find sleep 10 & have task id 0.

    You can use 
        -> fg 0, to make sleep run fore groud, or use
        -> kill 0, to directly stop the execution.

    Or you can press ctrl-c to pause all the backgroud processes, and use bg to restore them
    like using 
        -> bg 0, to restore sleep 10 & in the backgroud.

Batch operations:
    trash
        trash <batch_file>              -- execute a batch_file

    shift
        shift <shift_number>            -- shift the parameters <shift_number> times

    
As for the dynamic_string, the ${variable} will be parsed, but for static_string, it won't be.

    -> echo \'${hello} is\' \"${hello}\"    
    ${hello} is <value_of_hello>

    The \", $ and { have to be explicitly written.

    -> echo \"hello\"                  
    hello

    -> echo 'hello'
    hello

Variables can be assigned using other variables.

    -> a=\"1\"
    -> 2=\"${a}\"
    then 2 will be 1 as well

    These are called variables, they can be quite helpful when programming using shell
    But the control flow of trash is not completed now, in such shot preiod of time.
    That's why it is called The Half Accomplished SHell, making TRASH.

Using redirections using 
    
    -> help > doc
    in this way, the output of command help will be put into file ./doc, and replace the original file ./doc
    or if you want to append, then
    -> help >> doc
    the help infomation will be put behind the file ./doc.


    You can also declare source explicitly. Inlcuding stdout and stderr.
    -> ./prog < input (1)> output (2)> error
    Here (1)> means the stdout will be put to ./output, and (2)> means the stderr will pe put to file ./error.

    The redirection can be in any order, like
    -> ./prog (1)> output (2)> error < input

    Append is also avaliable
    -> ./prog < input (1)>> output (2)>> error

    the parenthesis cannot be emitted, or it cannot be parsed

Using pipes

    Pipes can be seens as a transport on stdin and stderr, like

    -> command1 | command2 | command3

    In this way, the stdout of command 1 will be put into the stdin of command2.
    And the stdout of command2 will be put into command3.

    For example program ./double will receive a line, and print it twice
    then ./a | ./a | ./a will print the input for three times

    During the input/output passing process, all the stderr are not redirected if not declared explicitly.

    If you want to redirect, feel free to do like
    -> command1 (2)> error | command2 (2)> another | command3
    it will also work

";