/*EBNF GRAMMAR 巴克斯范式
<Expr> ::= <AddSubExpr>
<AddSubExpr> ::= <MulDivExpr> {('+' | '-') <MulDivExpr>}
<MulDivExpr> ::= <PrimaryExpr> {('*' | '/') <PrimaryExpr>}
<PrimaryExpr> ::= NUM | '-'NUM | '(' <Expr> ')'
*/

use std::{process, io::{self, Write}};

// 定义所有可能的 Token 类型
#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenType {
    NUMBER,
    ADD, SUB, MUL, DIV,
    LEFTPAREN, RIGHTPAREN,
    END,
    UNKNOWN // 用于初始化或错误状态
}

// 包含所有解析器状态的结构体
struct Calculator {
    src_chars: Vec<char>, // 存储表达式的字符向量
    current_index: usize,
    current_token: TokenType,
    number_val: i32,
    debug_mode: bool,
}

impl Calculator {
    // 构造函数
    fn new(src: String, debug: bool) -> Self {
        Self {
            src_chars: src.chars().collect(),
            current_index: 0,
            current_token: TokenType::UNKNOWN,
            number_val: 0,
            debug_mode: debug,
        }
    }

    // 调试输出
    fn debug(&self, message: &str) {
        if self.debug_mode {
            println!("[调试] {}", message);
        }
    }

    // 错误处理，停止程序
    fn error(&self, message: &str) -> ! {
        eprintln!("错误: {}", message);
        process::exit(1);
    }

    // 词法分析器：获取下一个 Token
    fn get_token(&mut self) {
        // 跳过空白字符
        while self.current_index < self.src_chars.len() &&
            self.src_chars[self.current_index].is_whitespace()
        {
            self.current_index += 1;
        }

        if self.current_index >= self.src_chars.len() {
            self.current_token = TokenType::END;
            self.debug("Token: 结束");
            return;
        }

        let current_char = self.src_chars[self.current_index];

        self.current_token = match current_char {
            '+' => TokenType::ADD,
            '-' => TokenType::SUB,
            '*' => TokenType::MUL,
            '/' => TokenType::DIV,
            '(' => TokenType::LEFTPAREN,
            ')' => TokenType::RIGHTPAREN,
            '0'..='9' => {
                // 解析数字
                self.number_val = 0;
                let start_index = self.current_index;

                while self.current_index < self.src_chars.len() &&
                    self.src_chars[self.current_index].is_digit(10)
                {
                    // 将字符转换为数字并累加
                    let digit = self.src_chars[self.current_index].to_digit(10).unwrap();
                    self.number_val = self.number_val * 10 + (digit as i32);
                    self.current_index += 1;
                }

                // 重置索引以进行统一推进（实际上数字已经在上面移动了）
                self.current_index = start_index;
                TokenType::NUMBER
            },
            _ => self.error(&format!("未知 Token: {}", current_char)),
        };

        // 统一推进索引
        if self.current_token != TokenType::NUMBER {
            self.current_index += 1;
        } else {
            // 对于 NUMBER Token，需要移动到数字的末尾
            while self.current_index < self.src_chars.len() &&
                self.src_chars[self.current_index].is_digit(10)
            {
                self.current_index += 1;
            }
        }

        self.debug(&format!("Token: {:?}", self.current_token));
    }


    // <Expr> ::= <AddSubExpr>
    fn eval_expr(&mut self) -> i32 {
        self.debug("求值: 表达式");
        self.eval_add_sub_expr()
    }

    // <AddSubExpr> ::= <MulDivExpr> {('+' | '-') <MulDivExpr>}
    fn eval_add_sub_expr(&mut self) -> i32 {
        self.debug("求值: 加减表达式");

        let mut result = self.eval_mul_div_expr();

        while self.current_token == TokenType::ADD || self.current_token == TokenType::SUB {
            let op_token = self.current_token; // 记录操作符
            self.get_token();                  // 消耗操作符，获取下一个 Token
            let temp_val = self.eval_mul_div_expr(); // 计算右侧表达式

            match op_token {
                TokenType::ADD => result += temp_val,
                TokenType::SUB => result -= temp_val,
                _ => {},
            }
        }

        result
    }

    // <MulDivExpr> ::= <PrimaryExpr> {('*' | '/') <PrimaryExpr>}
    fn eval_mul_div_expr(&mut self) -> i32 {
        self.debug("求值: 乘除表达式");

        let mut result = self.eval_primary_expr();

        while self.current_token == TokenType::MUL || self.current_token == TokenType::DIV {
            let op_token = self.current_token; // 记录操作符
            self.get_token();                  // 消耗操作符，获取下一个 Token
            let temp_val = self.eval_primary_expr(); // 计算右侧表达式

            match op_token {
                TokenType::MUL => result *= temp_val,
                TokenType::DIV => {
                    if temp_val == 0 {
                        self.error("除零错误");
                    }
                    result /= temp_val;
                },
                _ => {},
            }
        }

        result
    }

    // <PrimaryExpr> ::= NUM | '-'NUM | '(' <Expr> ')'
    fn eval_primary_expr(&mut self) -> i32 {
        self.debug("求值: 基本表达式");

        let result = match self.current_token {
            TokenType::NUMBER => {
                let val = self.number_val;
                self.get_token(); // 消耗数字
                val
            },
            TokenType::SUB => { // 识别为一元负号
                self.get_token(); // 消耗 '-'
                if self.current_token == TokenType::NUMBER {
                    let val = -self.number_val;
                    self.get_token(); // 消耗数字
                    val
                } else if self.current_token == TokenType::LEFTPAREN {
                    // 支持 -(Expr) 格式
                    let val = self.eval_primary_expr();
                    -val
                } else {
                    self.error("一元负号后必须跟数字或带括号的表达式");
                }
            }
            TokenType::LEFTPAREN => {
                self.get_token(); // 消耗 '('
                let val = self.eval_expr();
                if self.current_token != TokenType::RIGHTPAREN {
                    self.error("缺少右括号 ')'");
                }
                self.get_token(); // 消耗 ')'
                val
            },
            _ => self.error("非法基本表达式起始 (期望数字、'-' 或 '(')"),
        };

        result
    }
}

fn main() {
    print!("写下你想计算的算式: ");
    // 确保提示立即显示
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let src = buf.trim().to_string();

    // 初始化解析器
    // 启用 调试 模式
    let mut calculator = Calculator::new(src, true);

    // 开始解析
    calculator.get_token(); // 获取第一个 Token
    let expr_val = calculator.eval_expr();

    if calculator.current_token != TokenType::END {
        calculator.error("表达式后存在多余字符");
    }

    println!("结果是: {}", expr_val);
}

